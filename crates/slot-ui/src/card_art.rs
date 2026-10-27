//! A platform's own art, read off the card, overriding the built-in assets.
//!
//! The card may keep any number of grey templates and say where each one takes its label:
//!
//! ```text
//! Carts/NES/cart.txt                  label = 25 20 253 208
//! Carts/NES/cart.png                  the one template, when no others are named
//!
//! Carts/NES/cart.txt                  template = 1.png 2.png 3.png
//!                                     label = 25 20 253 208
//!                                     label.2.png = 30 30 200 180
//! Carts/NES/1.png 2.png 3.png
//! ```
//!
//! A key written as `<key>.<file>` overrides that key for one picture alone; a key on its own
//! is the one every picture that does not name its own falls back to.
//!
//! Anything wrong with any of them is reported on stderr and that picture is dropped. A platform
//! left with no picture at all keeps the built-in asset, so a card with no `Carts` folder behaves
//! exactly as it did before.
//!
//! One a platform, not a mould: `Carts/GB/` and `Carts/GBC/` are the same shape but may still
//! carry different art, and neither should be dropped in favour of the other.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use slot_store::{Cart, Platform};

use crate::art;
use crate::bitmaps::{Mould, GB_CART_H, GB_CART_W};
use crate::cart::{CART_H, CART_W};

/// The folder under the card's root holding a platform's own art, beside `Labels` and `Games`.
pub const CARTS: &str = "Carts";

/// The template's picture inside `Carts/<机种>`, and its configuration beside it.
const PNG: &str = "cart.png";
const TXT: &str = "cart.txt";

/// Where a cart's choice of template and plastic is kept, one a game, beside its label.
const PICKS: &str = "cart_pick.txt";

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

/// Which of a platform's templates and which plastic a cart is drawn with. Both are positions:
/// the template in the order the card named them, the plastic in `PLASTICS`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pick {
    pub template: usize,
    pub plastic: usize,
}

/// One of a platform's pictures: the grey shell a cart of it is drawn with, and where that
/// picture takes its label.
struct Template {
    png: PathBuf,
    label: (u32, u32, u32, u32),
    /// How far the label box's corners round off, when the card names it.
    corner: Option<u32>,
    /// The size this shape's built-in art is drawn at, for the shapes measured in absolute
    /// pixels all the way through. A picture of another size would be quietly cut short rather
    /// than scaled, so it is turned down instead. Media are free: a shell is drawn at whatever
    /// size its picture is.
    must_be: Option<(u32, u32)>,
    /// Decoded and checked on first use, not at start-up: there are tens of these and a card
    /// draws only the few it is asked for.
    rgba: OnceLock<Option<(Vec<u8>, u32, u32)>>,
    /// Read off the picture's alpha, and the rounded corners, both on first use. They belong to
    /// the art rather than to the platform: reading the card again throws the old ones away with
    /// the art they were measured from.
    mask: OnceLock<Vec<u8>>,
    round: OnceLock<Vec<u8>>,
}

impl Template {
    /// The picture, once it has been checked. None when it does not hold up, which leaves the
    /// built-in asset in place.
    fn decoded(t: &'static Template) -> Option<&'static (Vec<u8>, u32, u32)> {
        t.rgba.get_or_init(|| t.check()).as_ref()
    }

    fn check(&self) -> Option<(Vec<u8>, u32, u32)> {
        let at = self.png.display();
        let (rgba, w, h) = art::decode(&self.png).or_else(|| {
            eprintln!("slot: carts: {at}: does not decode");
            None
        })?;
        if let Some((want_w, want_h)) = self.must_be {
            if (w, h) != (want_w, want_h) {
                eprintln!("slot: carts: {at}: it is {w}x{h}, which has to be {want_w}x{want_h}");
                return None;
            }
        }
        let (x, y, lw, lh) = self.label;
        if x + lw > w || y + lh > h {
            eprintln!("slot: carts: {at}: the label box {x} {y} {lw} {lh} leaves the picture");
            return None;
        }
        Some((rgba, w, h))
    }
}

/// One platform's own art, as read off the card.
struct Own {
    /// The pictures, in the order the card named them.
    templates: Vec<Template>,
    /// Which of `PLASTICS` this platform offers, when the card narrows it down. None is all of
    /// them, in order.
    plastics: Option<Vec<usize>>,
    /// What each game is drawn with, read from `cart_pick.txt` beside its label.
    picks: HashMap<String, Pick>,
}

/// One a platform, at its index, empty until `load_card_art` runs. Read without holding the lock:
/// what it hands out lives as long as the program does.
static OWN: Mutex<Vec<Option<&'static Own>>> = Mutex::new(Vec::new());

fn own(platform: Platform) -> Option<&'static Own> {
    OWN.lock().ok()?.get(platform.index()).copied().flatten()
}

fn template(platform: Platform, at: usize) -> Option<&'static Template> {
    own(platform)?.templates.get(at)
}

/// Which template a position means: that one when it holds up, the first that does when it does
/// not. A pick is kept from one run to the next, so a card that has since lost a picture is not
/// left pointing at the gap it left - that would draw the cart with no picture and no label box
/// either, half of the built-in shape and half of the card's.
fn resolve(art: &'static Own, at: usize) -> usize {
    let holds = |at: usize| {
        art.templates
            .get(at)
            .is_some_and(|t| Template::decoded(t).is_some())
    };
    if holds(at) {
        return at;
    }
    (0..art.templates.len()).find(|&at| holds(at)).unwrap_or(0)
}

/// Reads the card's own art. A platform with none, or with art that does not hold up, keeps the
/// built-in one and says why on stderr. Safe to run again: it replaces what it found before.
pub fn load_card_art(root: &Path) {
    let mut found: Vec<Option<Own>> = (0..Platform::COUNT).map(|_| None).collect();
    for platform in Platform::ALL {
        let dir = root.join(CARTS).join(platform.dir_name());
        if !dir.join(TXT).is_file() {
            continue; // No art is the usual case; say nothing.
        }
        match read(&dir, Mould::of(platform)) {
            Ok(mut art) => {
                art.picks = read_picks(&root.join("Labels").join(platform.dir_name()));
                found[platform.index()] = Some(art);
            }
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

/// What each game of this platform is drawn with: `<stem> = <template> <plastic>`, one a line.
fn read_picks(dir: &Path) -> HashMap<String, Pick> {
    let Ok(text) = std::fs::read_to_string(dir.join(PICKS)) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (stem, value) in slot_store::ini::parse(&text) {
        let mut words = value.split_whitespace();
        let (Some(t), Some(p)) = (words.next(), words.next()) else {
            eprintln!("slot: carts: {stem}: a pick is a template and a plastic");
            continue;
        };
        match (t.parse(), p.parse()) {
            (Ok(template), Ok(plastic)) => {
                out.insert(stem, Pick { template, plastic });
            }
            _ => eprintln!("slot: carts: {stem}: a pick is two whole numbers"),
        }
    }
    out
}

/// What stopped the card's art from being used.
fn read(dir: &Path, mould: Mould) -> Result<Own, String> {
    let text = std::fs::read_to_string(dir.join(TXT)).map_err(|_| "cart.txt does not read")?;
    let fields = slot_store::ini::parse(&text);
    let plastics = fields.get("colour").map(|v| plastic_list(v, dir));
    let templates = templates(&fields, dir, mould)?;
    Ok(Own {
        templates,
        plastics,
        picks: HashMap::new(),
    })
}

/// The pictures the card named, each with the label box and corner that apply to it. A picture
/// that does not hold up is dropped; none of them holding up is what turns the platform's art
/// down as a whole.
fn templates(
    fields: &HashMap<String, String>,
    dir: &Path,
    mould: Mould,
) -> Result<Vec<Template>, String> {
    let named = match fields.get("template") {
        Some(value) => value.split_whitespace().map(str::to_string).collect(),
        None => vec![PNG.to_string()],
    };
    if named.is_empty() {
        return Err("template names no pictures".into());
    }
    let mut out = Vec::new();
    for name in named {
        if name.contains(['/', '\\']) || name.starts_with("..") {
            eprintln!("slot: carts: {}: {name} is not a picture's name", dir.display());
            continue;
        }
        match template_of(fields, dir, &name, mould) {
            Ok(t) => out.push(t),
            Err(why) => eprintln!("slot: carts: {}: {why}", dir.join(&name).display()),
        }
    }
    if out.is_empty() {
        return Err("no picture holds up".into());
    }
    Ok(out)
}

/// The size this shape's built-in art is drawn at, which a picture standing in for it has to
/// match. Media are free: nothing downstream of a medium's shell is measured in absolute pixels,
/// so a card may draw one at whatever size its picture is. A Game Boy pak is not free - its
/// board and back plastic are bitmaps of a fixed size the shell is laid over.
fn built_in_size(mould: Mould) -> Option<(u32, u32)> {
    match mould {
        Mould::Gba => Some((CART_W, CART_H)),
        Mould::Notched | Mould::Rounded => Some((GB_CART_W, GB_CART_H)),
        Mould::Media(_) => None,
    }
}

fn template_of(
    fields: &HashMap<String, String>,
    dir: &Path,
    name: &str,
    mould: Mould,
) -> Result<Template, String> {
    let at = |key: &str| fields.get(&format!("{key}.{name}")).or_else(|| fields.get(key));
    let Some(value) = at("label") else {
        return Err("no label".into());
    };
    let label = rect(value).ok_or("label is not four whole numbers")?;
    if label.2 == 0 || label.3 == 0 {
        return Err("label has no width or height".into());
    }
    let corner = match at("corner").map(|v| v.parse::<u32>()) {
        Some(Ok(r)) => Some(r),
        Some(Err(_)) => return Err("corner is not a whole number".into()),
        None => None,
    };
    Ok(Template {
        png: dir.join(name),
        label,
        corner,
        must_be: built_in_size(mould),
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

/// How many pictures this platform offers. None of them, when the card brought none that held
/// up, is the built-in asset being kept.
pub fn count(platform: Platform) -> usize {
    own(platform).map_or(0, |art| art.templates.len())
}

/// What this cart is drawn with: the template and plastic the card picked for it, or the first
/// of each when it did not pick one, or picked one this platform does not offer.
pub fn pick_of(cart: &Cart) -> Pick {
    let Some(art) = own(cart.platform) else {
        return Pick::default();
    };
    let want = art.picks.get(&cart.stem).copied().unwrap_or_default();
    Pick {
        template: resolve(art, want.template),
        plastic: match art.plastics.as_deref() {
            // A pick is a position in the list the card offers, not in `PLASTICS` itself.
            Some(list) if !list.is_empty() => list
                .get(want.plastic)
                .copied()
                .unwrap_or_else(|| list[0]),
            _ => want.plastic.min(PLASTIC_COUNT - 1),
        },
    }
}

/// The label box the card asks for in place of the built-in one. Its picture has to check out
/// first: a shell that fell back to the built-in one is drawn with the built-in label box too,
/// rather than one measured for a different picture.
pub fn label(platform: Platform, pick: Pick) -> Option<(u32, u32, u32, u32)> {
    let t = template(platform, pick.template)?;
    Template::decoded(t)?;
    Some(t.label)
}

/// The size the picture is, which is the size the cart is drawn at.
pub fn size(platform: Platform, pick: Pick) -> Option<(u32, u32)> {
    let (_, w, h) = Template::decoded(template(platform, pick.template)?)?;
    Some((*w, *h))
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
pub fn neutral(platform: Platform, pick: Pick) -> Option<&'static [u8]> {
    let (rgba, _, _) = Template::decoded(template(platform, pick.template)?)?;
    Some(rgba)
}

/// Where on the label box the label may print when the card rounds its corners off: full
/// coverage but for the four corners, each cut by its own quarter circle. `paste_label` already
/// reads its coverage a pixel at a time, so this is all it takes.
pub fn label_mask(platform: Platform, pick: Pick) -> Option<&'static [u8]> {
    let t = template(platform, pick.template)?;
    Template::decoded(t)?;
    let (_, _, w, h) = t.label;
    let r = t.corner?;
    Some(t.round.get_or_init(|| rounded(w, h, r)))
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
pub fn mask(platform: Platform, pick: Pick) -> Option<&'static [u8]> {
    let t = template(platform, pick.template)?;
    let (rgba, _, _) = Template::decoded(t)?;
    Some(t.mask.get_or_init(|| rgba.iter().skip(3).step_by(4).copied().collect()))
}
