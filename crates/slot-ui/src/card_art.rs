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

/// How far each of a label box's four corners rounds off, clockwise from the top left — the order
/// a stylesheet reads `border-radius` in, so a card copied from one needs no thought.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corner {
    pub tl: u32,
    pub tr: u32,
    pub br: u32,
    pub bl: u32,
}

impl Corner {
    /// All four the same, which is what a card naming one number means.
    pub fn all(r: u32) -> Self {
        Self {
            tl: r,
            tr: r,
            br: r,
            bl: r,
        }
    }

    /// Clockwise from the top left, which is the order `rounded` reads them in.
    pub fn as_array(self) -> [u32; 4] {
        [self.tl, self.tr, self.br, self.bl]
    }

    /// Whether nothing rounds: the mask is full coverage everywhere and need not be built.
    pub fn is_square(self) -> bool {
        self.tl == 0 && self.tr == 0 && self.br == 0 && self.bl == 0
    }

    /// The same four corners moved by one scaling. A box squeezed sideways and upwards at
    /// different rates has no single scale, so the two are averaged: a corner is a shape of its
    /// own, and the eye reads it as too round or too square either way.
    pub fn scaled(self, by: f32) -> Self {
        let at = |v: u32| (v as f32 * by).round() as u32;
        Self {
            tl: at(self.tl),
            tr: at(self.tr),
            br: at(self.br),
            bl: at(self.bl),
        }
    }
}

/// A picture's label box and corners after the picture was scaled to the shape's size. Written
/// once, the first time the picture is decoded.
#[derive(Clone, Copy)]
struct Fit {
    label: (u32, u32, u32, u32),
    corner: Option<Corner>,
}

impl Fit {
    fn of(
        label: (u32, u32, u32, u32),
        corner: Option<Corner>,
        w: u32,
        h: u32,
        want_w: u32,
        want_h: u32,
    ) -> Self {
        let (sx, sy) = (want_w as f32 / w as f32, want_h as f32 / h as f32);
        let at = |v: u32, s: f32| (v as f32 * s).round() as u32;
        Self {
            label: (
                at(label.0, sx),
                at(label.1, sy),
                at(label.2, sx),
                at(label.3, sy),
            ),
            corner: corner.map(|c| c.scaled((sx + sy) / 2.0)),
        }
    }
}

/// One of a platform's pictures: the grey shell a cart of it is drawn with, and where that
/// picture takes its label.
struct Template {
    png: PathBuf,
    label: (u32, u32, u32, u32),
    /// How far the label box's corners round off, each its own, when the card names them.
    corner: Option<Corner>,
    /// The size this shape's built-in art is drawn at, which a picture standing in for it is
    /// scaled to. The shapes measured in absolute pixels all the way through have to match it, or
    /// the three bitmaps a pak is stacked from stop lining up. Media are free: a shell is drawn at
    /// whatever size its picture is, and nothing is scaled.
    must_be: Option<(u32, u32)>,
    /// Decoded and checked on first use, not at start-up: there are tens of these and a card
    /// draws only the few it is asked for.
    rgba: OnceLock<Option<(Vec<u8>, u32, u32)>>,
    /// Read off the picture's alpha, and the rounded corners, both on first use. They belong to
    /// the art rather than to the platform: reading the card again throws the old ones away with
    /// the art they were measured from.
    mask: OnceLock<Vec<u8>>,
    round: OnceLock<Vec<u8>>,
    /// Where the label box and corners ended up, when the picture had to be scaled to the shape's
    /// size to be used at all. Empty while it did not.
    fit: OnceLock<Fit>,
}

impl Template {
    /// The label box, in the picture's own pixels: the one the card wrote, unless the picture was
    /// scaled, in which case the one that scaling moved it to.
    fn label(&self) -> (u32, u32, u32, u32) {
        self.fit.get().map_or(self.label, |f| f.label)
    }

    /// The corners, moved by the same scaling the label box was, when there was one.
    fn corner(&self) -> Option<Corner> {
        match self.fit.get() {
            Some(f) => f.corner,
            None => self.corner,
        }
    }

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
        // A pak's shell is laid over its board and back plastic, three bitmaps at one size, so a
        // picture standing in for the shell is scaled to that size rather than turned away: it is
        // the art its owner drew, and being a few pixels out is not worth losing it over. The box
        // and the corners are moved with it, or the cover would land somewhere else entirely.
        if let Some((want_w, want_h)) = self.must_be {
            if (w, h) != (want_w, want_h) {
                let _ = self
                    .fit
                    .set(Fit::of(self.label, self.corner, w, h, want_w, want_h));
                eprintln!(
                    "slot: carts: {at}: {w}x{h} scaled to {want_w}x{want_h} to fit this shape, \
                     its label box moved with it"
                );
                return self.keep(resample(&rgba, w, h, want_w, want_h), want_w, want_h);
            }
        }
        self.keep(rgba, w, h)
    }

    /// The picture, once its label box has been seen to sit inside it.
    fn keep(&self, rgba: Vec<u8>, w: u32, h: u32) -> Option<(Vec<u8>, u32, u32)> {
        let (x, y, lw, lh) = self.label();
        if x + lw > w || y + lh > h {
            eprintln!(
                "slot: carts: {}: the label box {x} {y} {lw} {lh} leaves the picture",
                self.png.display()
            );
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
            eprintln!(
                "slot: carts: {}: {name} is not a picture's name",
                dir.display()
            );
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
    let at = |key: &str| {
        fields
            .get(&format!("{key}.{name}"))
            .or_else(|| fields.get(key))
    };
    let Some(value) = at("label") else {
        return Err("no label".into());
    };
    let label = rect(value).ok_or("label is not four whole numbers")?;
    if label.2 == 0 || label.3 == 0 {
        return Err("label has no width or height".into());
    }
    // A corner that does not read is not worth losing the picture over: it is noted and squared
    // off, which is what a card that never named one gets.
    let corner = at("corner").and_then(|v| corner_of(&v));
    Ok(Template {
        png: dir.join(name),
        label,
        corner,
        must_be: built_in_size(mould),
        rgba: OnceLock::new(),
        mask: OnceLock::new(),
        round: OnceLock::new(),
        fit: OnceLock::new(),
    })
}

/// "6" rounds all four corners; "12 12 2 2" rounds the top left, top right, bottom right and
/// bottom left, in that order. Anything else — two numbers, a word, a minus sign — is noted and
/// read as no corner at all, which is what a card that never named one gets.
fn corner_of(v: &str) -> Option<Corner> {
    let nums: Vec<u32> = v
        .split_whitespace()
        .map(|s| s.parse::<u32>())
        .collect::<Result<_, _>>()
        .unwrap_or_default();
    match nums.len() {
        1 => Some(Corner::all(nums[0])),
        4 => Some(Corner {
            tl: nums[0],
            tr: nums[1],
            br: nums[2],
            bl: nums[3],
        }),
        n => {
            eprintln!(
                "slot: carts: corner 需要 1 个或 4 个数（左上 右上 右下 左下），收到 {n} 个: {v:?}"
            );
            None
        }
    }
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
    let (x, y, w, h) = (
        words.next()??,
        words.next()??,
        words.next()??,
        words.next()??,
    );
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
            Some(list) if !list.is_empty() => {
                list.get(want.plastic).copied().unwrap_or_else(|| list[0])
            }
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
    Some(t.label())
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

/// Which plastic this cart is drawn in, when the card brought the pictures the plastics go
/// beside. None when it did not: a cart modelled from the built-in asset is left the plastic
/// that asset was drawn in, rather than recoloured by a list the card never offered.
pub fn plastic_of(cart: &Cart) -> Option<usize> {
    own(cart.platform)?;
    let pick = pick_of(cart);
    plastics(cart.platform).get(pick.plastic).copied()
}

/// Where this cart's plastic sits **in the list its platform offers**, which is the row the picker
/// draws it on and the number a pick is written as. `pick_of` answers in `PLASTICS` instead, and
/// the two are the same number only while the card offers every plastic in order.
pub fn plastic_slot(cart: &Cart) -> usize {
    let pick = pick_of(cart);
    plastics(cart.platform)
        .iter()
        .position(|&offered| offered == pick.plastic)
        .unwrap_or(0)
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
    let (_, _, w, h) = t.label();
    let corner = t.corner()?;
    Some(t.round.get_or_init(|| {
        // Square corners are full coverage everywhere, which is what a label box with no mask
        // gets: nothing to cut, so nothing to build a mask for.
        if corner.is_square() {
            return vec![255; (w * h) as usize];
        }
        let (r, cut) = clamp_corner(w, h, corner.as_array());
        if cut {
            eprintln!(
                "slot: carts: corner 被钳制到不相交（{w}x{h} 的角最多到 {}）",
                (w / 2).min(h / 2)
            );
        }
        rounded(w, h, r)
    }))
}

/// RGBA resampled to another size, one pixel read from the four it covers. `bitmaps::scale` picks
/// the nearest pixel alone, which is fine for the built-in art it was written for and leaves a
/// picture shrunk by a third with jagged edges.
fn resample(src: &[u8], sw: u32, sh: u32, w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let fy = (y as f32 + 0.5) * sh as f32 / h as f32 - 0.5;
        let y0 = fy.floor().clamp(0.0, sh as f32 - 1.0) as u32;
        let y1 = (y0 + 1).min(sh - 1);
        let wy = (fy - y0 as f32).clamp(0.0, 1.0);
        for x in 0..w {
            let fx = (x as f32 + 0.5) * sw as f32 / w as f32 - 0.5;
            let x0 = fx.floor().clamp(0.0, sw as f32 - 1.0) as u32;
            let x1 = (x0 + 1).min(sw - 1);
            let wx = (fx - x0 as f32).clamp(0.0, 1.0);
            for c in 0..4usize {
                let at = |x: u32, y: u32| src[((y * sw + x) * 4 + c as u32) as usize] as f32;
                let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * wx;
                let bottom = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * wx;
                let v = top + (bottom - top) * wy;
                out[((y * w + x) * 4 + c as u32) as usize] = v.round() as u8;
            }
        }
    }
    out
}

/// Keeps a corner within half of the shorter side, and keeps two corners sharing a side from
/// meeting: those two are scaled together rather than cut, which keeps the shape's proportions.
/// Returns whether anything had to give, so the card's author is told once.
fn clamp_corner(w: u32, h: u32, mut r: [u32; 4]) -> ([u32; 4], bool) {
    let mut cut = false;
    let half = (w / 2).min(h / 2);
    for v in r.iter_mut() {
        if *v > half {
            *v = half;
            cut = true;
        }
    }
    // (one corner, the other, the side they share): 0 = tl, 1 = tr, 2 = br, 3 = bl.
    for (a, b, lim) in [(0usize, 1usize, w), (3, 2, w), (0, 3, h), (1, 2, h)] {
        let sum = r[a] + r[b];
        if sum > lim && sum > 0 {
            r[a] = r[a] * lim / sum;
            r[b] = r[b] * lim / sum;
            cut = true;
        }
    }
    (r, cut)
}

/// A rounded rectangle's coverage over a w by h box, each corner cut by a circle of its own radius
/// centred that far in from the two edges it rounds, the cut antialiased across a pixel. `r` is
/// clockwise from the top left, the order `Corner::as_array` gives.
fn rounded(w: u32, h: u32, r: [u32; 4]) -> Vec<u8> {
    let half = (w as f32 / 2.0).min(h as f32 / 2.0);
    let r: [f32; 4] = r.map(|v| (v as f32).min(half));
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // Which corner's box this pixel falls in, and the centre of the circle that cuts it.
            // Outside all four the pixel is the flat of the box and stays whole.
            let (cx, cy, rad) = if px < r[0] && py < r[0] {
                (r[0], r[0], r[0])
            } else if px > w as f32 - r[1] && py < r[1] {
                (w as f32 - r[1], r[1], r[1])
            } else if px > w as f32 - r[2] && py > h as f32 - r[2] {
                (w as f32 - r[2], h as f32 - r[2], r[2])
            } else if px < r[3] && py > h as f32 - r[3] {
                (r[3], h as f32 - r[3], r[3])
            } else {
                out.push(255);
                continue;
            };
            let cover = (rad - (px - cx).hypot(py - cy) + 0.5).clamp(0.0, 1.0);
            out.push((cover * 255.0).round() as u8);
        }
    }
    out
}

/// The shell's coverage, read off the card's own picture's alpha.
pub fn mask(platform: Platform, pick: Pick) -> Option<&'static [u8]> {
    let t = template(platform, pick.template)?;
    let (rgba, _, _) = Template::decoded(t)?;
    Some(
        t.mask
            .get_or_init(|| rgba.iter().skip(3).step_by(4).copied().collect()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn mask_of(w: u32, h: u32, c: Corner) -> Vec<u8> {
        let (r, _) = clamp_corner(w, h, c.as_array());
        rounded(w, h, r)
    }

    #[test]
    fn one_number_rounds_every_corner_the_same() {
        let c = Corner::all(10);
        let m = mask_of(80, 80, c);
        let at = |x: u32, y: u32| m[(y * 80 + x) as usize];
        assert_eq!(corner_of("10"), Some(c));
        assert_eq!(at(0, 0), 0, "the corner was not cut at all");
        for (x, y) in [(79, 0), (0, 79), (79, 79)] {
            assert_eq!(at(x, y), at(0, 0), "the corners differ at ({x}, {y})");
        }
    }

    #[test]
    fn each_corner_rounds_by_its_own_number() {
        let c = Corner {
            tl: 10,
            tr: 0,
            br: 0,
            bl: 2,
        };
        let m = mask_of(80, 80, c);
        let at = |x: u32, y: u32| m[(y * 80 + x) as usize];
        assert_eq!(corner_of("10 0 0 2"), Some(c));
        assert_eq!(at(0, 0), 0, "the top left was not cut");
        assert_eq!(at(79, 0), 255, "the square top right lost its corner");
        assert_eq!(at(79, 79), 255, "the square bottom right lost its corner");
        assert!(
            at(0, 79) < 255,
            "the bottom left kept a square corner at {}",
            at(0, 79)
        );
    }

    #[test]
    fn counts_other_than_one_or_four_are_no_corner() {
        assert_eq!(corner_of("1 2 3"), None);
        assert_eq!(corner_of("wide"), None);
        assert_eq!(corner_of("10 -1 0 0"), None);
        assert_eq!(corner_of("0"), Some(Corner::all(0)));
        assert!(Corner::all(0).is_square());
        assert!(!Corner::all(1).is_square());
    }

    #[test]
    fn corners_too_big_for_the_box_are_brought_back() {
        let (r, cut) = clamp_corner(80, 80, [999, 999, 0, 0]);
        assert!(cut, "nothing was clamped");
        assert_eq!(r, [40, 40, 0, 0]);
        let c = Corner {
            tl: 999,
            tr: 999,
            br: 0,
            bl: 0,
        };
        assert_eq!(mask_of(80, 80, c).len(), 80 * 80, "the mask is not the box");
    }

    #[test]
    fn a_picture_scaled_to_the_shape_takes_its_box_and_corners_with_it() {
        // Halved both ways: the box halves, and so does every corner.
        let fit = Fit::of((10, 20, 100, 50), Some(Corner::all(8)), 400, 200, 200, 100);
        assert_eq!(fit.label, (5, 10, 50, 25));
        assert_eq!(fit.corner, Some(Corner::all(4)));
        // Narrowed but not shortened: only the numbers across move.
        let fit = Fit::of((10, 20, 100, 50), Some(Corner::all(8)), 400, 200, 200, 200);
        assert_eq!(fit.label, (5, 20, 50, 50));
    }

    #[test]
    fn resampling_gives_back_the_size_it_was_asked_for() {
        let src = vec![7u8; 8 * 4 * 4];
        let out = resample(&src, 8, 4, 5, 3);
        assert_eq!(out.len(), 5 * 3 * 4, "not the size asked for");
        assert!(
            out.iter().all(|&v| v == 7),
            "a flat picture resampled to a varied one"
        );
        // The corners of a quartered picture keep their own colours.
        let mut src = vec![0u8; 2 * 2 * 4];
        for (i, v) in [10u8, 20, 30, 40].iter().enumerate() {
            src[i * 4] = *v;
        }
        let out = resample(&src, 2, 2, 2, 2);
        assert_eq!([out[0], out[4], out[8], out[12]], [10, 20, 30, 40]);
    }

    #[test]
    fn a_picture_may_round_its_own_corners() {
        let f = fields(&[
            ("template", "1.png 2.png"),
            ("label", "0 0 40 40"),
            ("corner", "6"),
            ("corner.2.png", "10 10 0 0"),
        ]);
        let dir = Path::new("nowhere");
        let one = template_of(&f, dir, "1.png", Mould::Gba).expect("1.png");
        let two = template_of(&f, dir, "2.png", Mould::Gba).expect("2.png");
        assert_eq!(one.corner, Some(Corner::all(6)), "the global did not apply");
        assert_eq!(
            two.corner,
            Some(Corner {
                tl: 10,
                tr: 10,
                br: 0,
                bl: 0
            }),
            "the picture's own corner did not apply"
        );
    }
}
