//! A platform's own art, read off the card, overriding the built-in assets.
//!
//! The card may keep a grey template and say where its label prints:
//!
//! ```text
//! Carts/NES/cart.png     a neutral grey shell, its alpha the cart's outline
//! Carts/NES/cart.txt     label = 25 20 253 208
//! ```
//!
//! Anything wrong with either is reported on stderr and the built-in asset is kept, so a card
//! with no `Carts` folder behaves exactly as it did before.
//!
//! One a platform, not a mould: `Carts/GB/` and `Carts/GBC/` are the same shape but may still
//! carry different art, and neither should be dropped in favour of the other.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use slot_store::Platform;

use crate::art;
use crate::bitmaps::{Mould, GB_CART_H, GB_CART_W};
use crate::cart::{CART_H, CART_W};

/// The folder under the card's root holding a platform's own art, beside `Labels` and `Games`.
pub const CARTS: &str = "Carts";

/// The template's picture inside `Carts/<机种>`, and its configuration beside it.
const PNG: &str = "cart.png";
const TXT: &str = "cart.txt";

/// The plastics a cart may be drawn in, in the order they are offered: grey first, the colour a
/// shell is before anyone picks another. **Only ever added to**: a pick is kept as this list's
/// position, so putting a colour anywhere but the end moves every pick after it.
pub const PLASTICS: [[u8; 3]; PLASTIC_COUNT] = [
    [156, 156, 156], // 0 gray
    [32, 32, 34],    // 1 black
    [236, 236, 232], // 2 white
    [44, 52, 148],   // 3 blue
    [232, 122, 162], // 4 pink
    [168, 44, 44],   // 5 red
    [44, 130, 66],   // 6 green
    [226, 186, 40],  // 7 yellow
];

pub const PLASTIC_COUNT: usize = 8;

/// One platform's own art, as read off the card.
struct Own {
    /// The shape it stands in for, which is what fixes the size the picture has to be.
    mould: Mould,
    png: PathBuf,
    label: (u32, u32, u32, u32),
    /// How far the label box's corners round off, when the card names it.
    corner: Option<u32>,
    /// Which of `PLASTICS` this platform offers, when the card narrows it down. None is all of
    /// them, in order.
    plastics: Option<Vec<usize>>,
    /// Decoded and checked on first use, not at start-up: there are tens of these and a card
    /// draws only the few it is asked for.
    rgba: OnceLock<Option<(Vec<u8>, u32, u32)>>,
    /// Read off the picture's alpha, and the rounded corners, both on first use. They belong to
    /// the art rather than to the platform: reading the card again throws the old ones away with
    /// the art they were measured from.
    mask: OnceLock<Vec<u8>>,
    round: OnceLock<Vec<u8>>,
}

impl Own {
    /// The picture, once it has been checked. None when it does not hold up, which leaves the
    /// built-in asset in place.
    fn decoded(own: &'static Own) -> Option<&'static (Vec<u8>, u32, u32)> {
        own.rgba.get_or_init(|| own.check()).as_ref()
    }

    fn check(&self) -> Option<(Vec<u8>, u32, u32)> {
        let at = self.png.display();
        let (rgba, w, h) = art::decode(&self.png).or_else(|| {
            eprintln!("slot: carts: {at}: cart.png does not decode");
            None
        })?;
        let (want_w, want_h) = built_in_size(self.mould);
        if (w, h) != (want_w, want_h) {
            eprintln!("slot: carts: {at}: cart.png is {w}x{h}, not {want_w}x{want_h}");
            return None;
        }
        let (x, y, lw, lh) = self.label;
        if x + lw > w || y + lh > h {
            eprintln!("slot: carts: {at}: the label box {x} {y} {lw} {lh} leaves the picture");
            return None;
        }
        Some((rgba, w, h))
    }
}

/// The size this shape's built-in art is drawn at, which a card's own picture has to match: the
/// Game Boy family is measured in absolute pixels all the way through, so a picture of another
/// size would be quietly cut short rather than scaled.
fn built_in_size(mould: Mould) -> (u32, u32) {
    match mould {
        Mould::Gba => (CART_W, CART_H),
        Mould::Notched | Mould::Rounded => (GB_CART_W, GB_CART_H),
        Mould::Media(m) => m.size(),
    }
}

/// One a platform, at its index, empty until `load_card_art` runs. Read without holding the lock:
/// what it hands out lives as long as the program does.
static OWN: Mutex<Vec<Option<&'static Own>>> = Mutex::new(Vec::new());

fn own(platform: Platform) -> Option<&'static Own> {
    OWN.lock().ok()?.get(platform.index()).copied().flatten()
}

/// Reads the card's own art. A platform with none, or with art that does not hold up, keeps the
/// built-in one and says why on stderr. Safe to run again: it replaces what it found before.
pub fn load_card_art(root: &Path) {
    let mut found: Vec<Option<Own>> = (0..Platform::COUNT).map(|_| None).collect();
    for platform in Platform::ALL {
        let dir = root.join(CARTS).join(platform.dir_name());
        if !dir.join(PNG).is_file() {
            continue; // No art is the usual case; say nothing.
        }
        match read(&dir, Mould::of(platform)) {
            Ok(art) => found[platform.index()] = Some(art),
            Err(why) => eprintln!("slot: carts: {}: {why}", dir.display()),
        }
    }
    let found = found
        .into_iter()
        .map(|one| one.map(|art| &*Box::leak(Box::new(art))))
        .collect();
    if let Ok(mut held) = OWN.lock() {
        *held = found;
    }
}

/// What stopped the card's art from being used.
fn read(dir: &Path, mould: Mould) -> Result<Own, String> {
    let text = std::fs::read_to_string(dir.join(TXT)).map_err(|_| "no cart.txt beside cart.png")?;
    let fields = slot_store::ini::parse(&text);
    let Some(value) = fields.get("label") else {
        return Err("cart.txt has no label".into());
    };
    let label = rect(value).ok_or("label is not four whole numbers")?;
    if label.2 == 0 || label.3 == 0 {
        return Err("label has no width or height".into());
    }
    let corner = match fields.get("corner").map(|v| v.parse::<u32>()) {
        Some(Ok(r)) => Some(r),
        Some(Err(_)) => {
            eprintln!("slot: carts: {}: corner is not a whole number", dir.display());
            None
        }
        None => None,
    };
    let plastics = fields.get("colour").map(|v| plastic_list(v, dir));
    Ok(Own {
        mould,
        png: dir.join(PNG),
        label,
        corner,
        plastics,
        rgba: OnceLock::new(),
        mask: OnceLock::new(),
        round: OnceLock::new(),
    })
}

/// Which of `PLASTICS` the card names, in the order it names them. A number past the end of the
/// table is dropped with a word on stderr, and one named twice is kept at its first place: the
/// list is narrowed, not the whole configuration thrown out with it.
fn plastic_list(value: &str, dir: &Path) -> Vec<usize> {
    let mut out = Vec::new();
    for word in value.split_whitespace() {
        match word.parse::<usize>() {
            Ok(i) if i < PLASTIC_COUNT => {
                if !out.contains(&i) {
                    out.push(i);
                }
            }
            _ => eprintln!(
                "slot: carts: {}: {word} is not one of the plastics 0..{}",
                dir.display(),
                PLASTIC_COUNT - 1
            ),
        }
    }
    out
}

/// Four whole numbers, `x y w h`, and nothing else.
fn rect(value: &str) -> Option<(u32, u32, u32, u32)> {
    let mut words = value.split_whitespace().map(|w| w.parse::<u32>().ok());
    let (x, y, w, h) = (words.next()??, words.next()??, words.next()??, words.next()??);
    words.next().is_none().then_some((x, y, w, h))
}

/// The label box the card asks for in place of the built-in one. Its picture has to check out
/// first: a shell that fell back to the built-in one is drawn with the built-in label box too,
/// rather than one measured for a different picture.
pub fn label(platform: Platform) -> Option<(u32, u32, u32, u32)> {
    let art = own(platform)?;
    Own::decoded(art)?;
    Some(art.label)
}

/// The plastics this platform offers, as positions in `PLASTICS`: every one of them in order
/// unless the card narrowed the list down, and every one of them too when it named none that
/// count, which is the same as not having written the line at all.
pub fn plastics(platform: Platform) -> &'static [usize] {
    static ALL: [usize; PLASTIC_COUNT] = [0, 1, 2, 3, 4, 5, 6, 7];
    match own(platform).and_then(|art| art.plastics.as_deref()) {
        Some(list) if !list.is_empty() => list,
        _ => &ALL,
    }
}

/// The card's own neutral shell, decoded once, when it has one that holds up.
pub fn neutral(platform: Platform) -> Option<&'static [u8]> {
    let (rgba, _, _) = Own::decoded(own(platform)?)?;
    Some(rgba)
}

/// Where on the label box the label may print when the card rounds its corners off: full
/// coverage but for the four corners, each cut by its own quarter circle. `paste_label` already
/// reads its coverage a pixel at a time, so this is all it takes.
pub fn label_mask(platform: Platform) -> Option<&'static [u8]> {
    let art = own(platform)?;
    Own::decoded(art)?;
    let (_, _, w, h) = art.label;
    let r = art.corner?;
    Some(art.round.get_or_init(|| rounded(w, h, r)))
}

/// A rounded rectangle's coverage over a w by h box, its corners cut by circles of radius r
/// centred r in from each of the two edges they round, the cut antialiased across a pixel.
fn rounded(w: u32, h: u32, r: u32) -> Vec<u8> {
    let r = (r as f32).min(w as f32 / 2.0).min(h as f32 / 2.0);
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let near = (px < r || px > w as f32 - r) && (py < r || py > h as f32 - r);
            let cover = match near {
                false => 1.0,
                true => {
                    let cx = if px < r { r } else { w as f32 - r };
                    let cy = if py < r { r } else { h as f32 - r };
                    (r - (px - cx).hypot(py - cy) + 0.5).clamp(0.0, 1.0)
                }
            };
            out.push((cover * 255.0).round() as u8);
        }
    }
    out
}

/// The shell's coverage, read off the card's own picture's alpha.
pub fn mask(platform: Platform) -> Option<&'static [u8]> {
    let art = own(platform)?;
    let (rgba, _, _) = Own::decoded(art)?;
    Some(art.mask.get_or_init(|| rgba.iter().skip(3).step_by(4).copied().collect()))
}
