//! Cart artwork the card supplies, in place of the outline SVG the build carries.
//!
//! `System/Carts/cart_gb.png`, `cart_gbc.png`, `cart_gba.png` — one per machine. Absent, the
//! built-in silhouette and flat shell colour are used and nothing changes; present, the card's
//! own art is the cart, and the shell colour is applied to it rather than to a mask.
//!
//! Two conventions the art has to keep, both of them discovered rather than invented:
//!
//! * **Alpha is the shape.** A transparent pixel is not cart, which is how the notch, the
//!   bulge and the rounded corners are cut.
//! * **Magenta is the label.** `#ff00ff` marks where the game's own label goes. The code cuts
//!   that region out, fills it with the colour just outside it — so the corners of a rounded
//!   label show the recess rather than a hole — and pastes the label into it, clipped to the
//!   same shape. Nothing else has to be written down: the art says where its label goes and how
//!   round its corners are.
//!
//! The artwork is loaded once at boot and read from the face-building thread afterwards, which
//! is why it lives in a `OnceLock` rather than being passed down.

use std::path::Path;


use slot_store::System;

use crate::art;
use crate::cart::{CartFace, CartSize};
use crate::shell::{Finish, Shell};

/// Where the card keeps it.
const DIR: &str = "System/Carts";

/// The magenta the label slot is marked with. The tolerance is generous because the art comes
/// back through a paint program: `#ff00ff` with green held well below the other two is what
/// every magenta that is meant as a marker looks like and no shell colour does.
const MARK_R: u8 = 200;
const MARK_G: u8 = 60;
const MARK_B: u8 = 200;

/// How bright the plastic is taken to be, as a percentile of its own pixels. Applied as
/// `colour * (luma / this)`, so anything at or below the reference keeps the shell colour's
/// value and only the true highlights may exceed it. The *median* was the first choice and it
/// was wrong: half the art then scaled up, and a shell's brightest highlight — which the built
/// in pipeline caps at 1.45x — ran to 255 and blew out to white.
const REF_PERCENTILE: f32 = 0.90;

/// The label slot, in the art's own pixels.
pub struct Label {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    /// Coverage of the slot, so a label with rounded corners keeps them.
    mask: Vec<u8>,
}

pub struct Art {
    rgba: Vec<u8>,
    w: u32,
    h: u32,
    label: Option<Label>,
    /// Median-ish brightness of the plastic, for the shell tint.
    luma: f32,
    /// The colour just outside the label slot, so the slot can be filled before the label is
    /// pasted and its corners read as recess rather than as a hole.
    recess: [u8; 3],
    /// Whether the shell colour is painted through this art at all.
    ///
    /// A drawing in `System/Carts` is a shell waiting to be coloured — it is grey, and the colour
    /// a cart's plastic happens to be is what makes two carts of one machine two carts. A border
    /// in `System/Borders` is a picture of a particular cart, sticker and all: it brought its own
    /// colour, and painting a shell colour over it would paint over the thing it is a picture of.
    tint: bool,
}

#[derive(Default)]
pub struct CartArt {
    gb: Option<Art>,
    gbc: Option<Art>,
    gba: Option<Art>,
    /// A Super Nintendo cart of the card's own. Absent, that machine is drawn with the built-in
    /// shell it shares with the Advance — same landscape shape — so a card that starts filing
    /// SFC games looks finished before anybody draws anything for it.
    sfc: Option<Art>,
    /// A Family Computer cart of the card's own. Absent, that machine wears the built-in
    /// yellow shell and its blue label panel, which is the shape a Famicom cart has anyway.
    fc: Option<Art>,
}

/// What the card offered, once somebody has offered it.
///
/// Re-settable rather than a `OnceLock`: it is installed once at boot and never again in the
/// program, but a test that cannot install a second card is a test that can only ever ask about
/// the first one, and the whole of what is worth testing here is the difference one card makes
/// from another.
static ART: std::sync::RwLock<Option<&'static CartArt>> = std::sync::RwLock::new(None);

/// Called once, at boot, before the face builder starts.
pub fn install(cart_art: CartArt) {
    if let Ok(mut slot) = ART.write() {
        *slot = Some(Box::leak(Box::new(cart_art)));
    }
}

pub fn art_for(system: System) -> Option<&'static Art> {
    let guard = ART.read().ok()?;
    let all = (*guard)?;
    match system {
        System::Gb => all.gb.as_ref(),
        System::Gbc => all.gbc.as_ref(),
        System::Gba => all.gba.as_ref(),
        System::Sfc => all.sfc.as_ref(),
        System::Fc => all.fc.as_ref(),
    }
}

/// One border, by number, read off the card there and then.
///
/// The shelf keeps one picture per machine and the picker shows every one the card has, so the
/// rest are read when they are asked for rather than held for the whole of a boot: a machine
/// with thirty-two borders is thirty-two photographs, and holding all of them for a shelf that
/// draws one is the wrong bargain. A border nobody has looked at costs nothing at all.
pub fn border_art(root: &Path, system: System, number: u32) -> Option<Art> {
    let border = slot_store::borders::borders(root, system)
        .into_iter()
        .find(|b| b.number == number)?;
    read(&border.path, false)
}

/// Read whatever the card offers. A missing, unreadable or index-coloured file is simply not
/// there: the built-in cart is a perfectly good cart.
pub fn load(root: &Path) -> CartArt {
    let dir = root.join(DIR);
    CartArt {
        gb: pick(root, System::Gb, &dir.join("cart_gb.png")),
        gbc: pick(root, System::Gbc, &dir.join("cart_gbc.png")),
        gba: pick(root, System::Gba, &dir.join("cart_gba.png")),
        sfc: pick(root, System::Sfc, &dir.join("cart_sfc.png")),
        fc: pick(root, System::Fc, &dir.join("cart_fc.png")),
    }
}

/// What one machine's carts are drawn as: **a border off the card, and only then the drawing in
/// `System/Carts`.**
///
/// A border is a picture of a real cart, so it is both the better shell and the truer outline —
/// and the outline is the part that used to be drawn in code from a description, which is how a
/// Family Computer cart came out as the shape somebody wrote down for it rather than the shape
/// the thing has. This way one picture on the card is the shell, the label slot and the outline
/// all at once, and a machine with no border at all is exactly what it was before.
fn pick(root: &Path, system: System, drawn: &Path) -> Option<Art> {
    let border = slot_store::borders::borders(root, system)
        .into_iter()
        .find_map(|b| read(&b.path, false));
    border.or_else(|| read(drawn, true))
}

/// What the card has, for the startup log: a card with art that was refused has nothing else to
/// say so.
pub fn summary() -> String {
    let guard = match ART.read() {
        Ok(g) => g,
        Err(_) => return "none".to_string(),
    };
    match *guard {
        None => "none".to_string(),
        Some(all) => [
            ("gb", &all.gb),
            ("gbc", &all.gbc),
            ("gba", &all.gba),
            ("sfc", &all.sfc),
            ("fc", &all.fc),
        ]
            .iter()
            .map(|(name, art)| match art {
                Some(a) if a.label.is_some() => format!("{name} {}x{}+label", a.w, a.h),
                Some(a) => format!("{name} {}x{}", a.w, a.h),
                None => format!("{name} -"),
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// Colour bleed at the alpha edge.
///
/// A cut-out carried over from a paint program keeps the colour it was cut from in its
/// semi-transparent pixels — on the cart art that is a near-black keyline, invisible while the
/// shell is dark and a dark halo the moment it is painted light grey. Three passes of a 3×3
/// average taken from the opaque pixels inward re-colours the edge with the cart's own colour,
/// which is what it should have been. Alpha is untouched: only the colour under it changes.
fn bleed_edge(rgba: &mut [u8], w: u32, h: u32) {
    for _ in 0..3 {
        let src = rgba.to_vec();
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let a = src[i + 3];
                if a == 0 || a == 255 {
                    continue;
                }
                let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let j = ((ny as u32 * w + nx as u32) * 4) as usize;
                        if src[j + 3] == 255 {
                            r += src[j] as u32;
                            g += src[j + 1] as u32;
                            b += src[j + 2] as u32;
                            n += 1;
                        }
                    }
                }
                if n > 0 {
                    rgba[i] = (r / n) as u8;
                    rgba[i + 1] = (g / n) as u8;
                    rgba[i + 2] = (b / n) as u8;
                }
            }
        }
    }
}

fn read(path: &Path, tint: bool) -> Option<Art> {
    let (rgba, w, h) = art::decode_rgba(path)?;
    if w < 8 || h < 8 || rgba.len() < (w * h * 4) as usize {
        return None;
    }
    let mut rgba = rgba;
    bleed_edge(&mut rgba, w, h);
    // **Nothing is done about magenta here.** The slot is the magenta, read on the one test below
    // and no other: a mark is `#ff00ff` and a pixel that is half magenta is not one. That is the
    // card's side of the bargain — the art is drawn with a hard-edged, pure magenta slot — and the
    // code's side is to take it at its word rather than to guess at what was meant. A softened
    // edge is therefore the art's to fix and not the code's: the alternative, deciding that a
    // pixel which is *nearly* magenta is close enough to be the slot, moves the slot's edge off
    // the edge the art drew and washes colours out of shells that were never marked at all.
    // Note: hardening this alpha was tried and made things worse. A cut-out's antialiased edge is
    // partly transparent, and a side slot draws its cart at 0.7 over a black silhouette, so the
    // silhouette shows through that band as a dark rim on a light theme. Pushing the edge to 0 or
    // 255 turns the soft band into a hard transparent one and lets *more* black through. The rim
    // is the silhouette's, not the edge's: see `Shelf::draw_row`, which does not draw it under a
    // cart that came with its own art.
    let label = find_label(&rgba, w, h);
    let luma = reference_luma(&rgba, w, h, label.as_ref());
    let recess = label
        .as_ref()
        .map(|l| ring_colour(&rgba, w, h, l))
        .unwrap_or([80, 80, 80]);
    Some(Art {
        rgba,
        w,
        h,
        label,
        luma,
        recess,
        tint,
    })
}


fn find_label(rgba: &[u8], w: u32, h: u32) -> Option<Label> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    let mut any = false;
    for y in 0..h {
        for x in 0..w {
            if is_mark(&rgba[((y * w + x) * 4) as usize..][..4]) {
                any = true;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if !any {
        return None;
    }
    let mut mask = vec![0u8; (w * h) as usize];
    for y in y0..=y1 {
        for x in x0..=x1 {
            if is_mark(&rgba[((y * w + x) * 4) as usize..][..4]) {
                mask[(y * w + x) as usize] = 255;
            }
        }
    }
    Some(Label { x0, y0, x1, y1, mask })
}

/// The magenta the card paints where the label slot is. Board art had marks in this colour too,
/// for the lamps the picker used to fill; that is gone, and this is the label's alone.
fn is_mark(p: &[u8]) -> bool {
    p[3] > 0 && p[0] > MARK_R && p[1] < MARK_G && p[2] > MARK_B
}

/// The shell's own brightness, ignoring the label slot.
fn reference_luma(rgba: &[u8], w: u32, h: u32, label: Option<&Label>) -> f32 {
    let mut lums = Vec::new();
    for y in (0..h).step_by(3) {
        for x in (0..w).step_by(3) {
            let inside = label.is_some_and(|l| {
                x >= l.x0 && x <= l.x1 && y >= l.y0 && y <= l.y1
            });
            let p = &rgba[((y * w + x) * 4) as usize..][..4];
            if p[3] > 200 && !inside {
                lums.push(0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32);
            }
        }
    }
    if lums.is_empty() {
        return 128.0;
    }
    lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let i = ((lums.len() - 1) as f32 * REF_PERCENTILE) as usize;
    lums[i].max(1.0)
}

/// Just outside the slot all the way round, median per channel.
fn ring_colour(rgba: &[u8], w: u32, h: u32, l: &Label) -> [u8; 3] {
    let mut samples: Vec<[u8; 3]> = Vec::new();
    let mut take = |x: u32, y: u32| {
        if x < w && y < h {
            let p = &rgba[((y * w + x) * 4) as usize..][..4];
            if p[3] > 0 {
                samples.push([p[0], p[1], p[2]]);
            }
        }
    };
    for x in l.x0..=l.x1 {
        take(x, l.y0.saturating_sub(3));
        take(x, (l.y1 + 3).min(h - 1));
    }
    for y in l.y0..=l.y1 {
        take(l.x0.saturating_sub(3), y);
        take((l.x1 + 3).min(w - 1), y);
    }
    if samples.is_empty() {
        return [80, 80, 80];
    }
    let mut out = [0u8; 3];
    for c in 0..3 {
        let mut v: Vec<u8> = samples.iter().map(|s| s[c]).collect();
        v.sort_unstable();
        out[c] = v[v.len() / 2];
    }
    out
}

/// Where the card's own label slot sits, as a share of the cart art: `x0, y0, x1, y1`. This is
/// the magenta region, which is where the label actually goes on this cart — so anything that
/// has to line up with the label has to line up with this rather than with the built-in panel.
pub fn label_share(system: System) -> Option<(f32, f32, f32, f32)> {
    art_for(system)?.label_share()
}

impl Art {
    /// The cart's own outline, at whatever size the caller is drawing: **the alpha of this art**,
    /// which is the shape a real cart has — the notch, the bulge, the corners — rather than the
    /// shape written down for it in code. The picker's board is cut with it, so a cart opened to
    /// change its core is the same shape as the cart on the shelf.
    pub fn outline(&self, w: u32, h: u32) -> Vec<u8> {
        let mut out = vec![0u8; (w * h) as usize];
        let sx = self.w as f32 / w as f32;
        let sy = self.h as f32 / h as f32;
        for y in 0..h {
            // Four samples rather than one: an outline taken at one point per pixel steps, and a
            // stepped edge reads as a cut edge rather than as a moulded one.
            let fy = (y as f32 + 0.5) * sy - 0.5;
            let y0 = fy.floor().clamp(0.0, self.h as f32 - 1.0) as u32;
            let y1 = (y0 + 1).min(self.h - 1);
            let wy = fy - y0 as f32;
            for x in 0..w {
                let fx = (x as f32 + 0.5) * sx - 0.5;
                let x0 = fx.floor().clamp(0.0, self.w as f32 - 1.0) as u32;
                let x1 = (x0 + 1).min(self.w - 1);
                let wx = fx - x0 as f32;
                let a = |x: u32, y: u32| {
                    self.rgba[((y * self.w + x) * 4 + 3) as usize] as f32
                };
                let top = a(x0, y0) * (1.0 - wx) + a(x1, y0) * wx;
                let bot = a(x0, y1) * (1.0 - wx) + a(x1, y1) * wx;
                let v = top * (1.0 - wy) + bot * wy;
                out[(y * w + x) as usize] = v.clamp(0.0, 255.0) as u8;
            }
        }
        out
    }

    /// The label slot as a share of the art, for a caller aligning something to it — the shelf's
    /// favourites star sits on its top right corner.
    pub fn label_share(&self) -> Option<(f32, f32, f32, f32)> {
        let l = self.label.as_ref()?;
        Some((
            l.x0 as f32 / self.w as f32,
            l.y0 as f32 / self.h as f32,
            (l.x1 + 1) as f32 / self.w as f32,
            (l.y1 + 1) as f32 / self.h as f32,
        ))
    }

    /// The picture's own proportions, so a caller about to build a face can build one **the shape
    /// of the art it is drawing** rather than the shape the machine is.
    ///
    /// A machine's borders are not all one shape: the Family Computer's are three of them, and
    /// the widest is two thirds again as wide as the squarest. See `cart::border_face`, which is
    /// why this exists.
    pub fn extent(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// The label slot in face pixels, so the caller can mint a label of the right size. The
    /// target face is the machine's own, which is why this cannot be a constant: the same art
    /// drawn on a taller cart has a taller slot.
    pub fn label_size(&self, size: CartSize) -> Option<(u32, u32)> {
        let l = self.label.as_ref()?;
        let w = ((l.x1 - l.x0 + 1) as f32 * size.face_w() as f32 / self.w as f32).round() as u32;
        let h = ((l.y1 - l.y0 + 1) as f32 * size.face_h() as f32 / self.h as f32).round() as u32;
        (w > 0 && h > 0).then_some((w, h))
    }

    /// The face the shelf draws: this art, in the cart's shell colour, with `label` pasted into
    /// the slot the art marks out.
    ///
    /// The colour is applied as a multiplier on the art's own brightness — `shell * luma/luma`
    /// for the shell's own value at the reference brightness — which is what the built-in path
    /// does too, where the shell colour is painted through the silhouette and the moulding is
    /// the same colour turned away from the light. So a card that paints a dark shell with a lit
    /// edge keeps the edge when the shell colour changes, which is the whole point of tinting
    /// rather than recolouring.
    /// `light` is what this machine's cart becomes on the light theme: `Some(grey)` to repaint
    /// it flat, `None` to use the art's own colour untouched. The dark theme always uses the
    /// shell colour, which is what the art was drawn to sit under.
    pub fn face(
        &self,
        shell: &Shell,
        label: &[u8],
        label_w: u32,
        label_h: u32,
        face_w: u32,
        face_h: u32,
        light: Option<[u8; 3]>,
    ) -> CartFace {
        let mut rgba = vec![0u8; (face_w * face_h * 4) as usize];
        let sx = self.w as f32 / face_w as f32;
        let sy = self.h as f32 / face_h as f32;
        let l = self.label.as_ref();
        // The slot in face pixels, so the fill and the paste land on the same rectangle the
        // label was minted for.
        let (lx, ly) = match l {
            Some(l) => (
                (l.x0 as f32 / sx).round() as i64,
                (l.y0 as f32 / sy).round() as i64,
            ),
            None => (0, 0),
        };
        for y in 0..face_h {
            let sy0 = y as f32 * sy;
            for x in 0..face_w {
                let sx0 = x as f32 * sx;
                let p = &self.rgba[(((sy0 as u32).min(self.h - 1) * self.w
                    + (sx0 as u32).min(self.w - 1))
                    * 4) as usize..][..4];
                if p[3] == 0 {
                    continue;
                }
                let in_slot = label_w > 0
                    && l.is_some()
                    && (x as i64) >= lx
                    && (x as i64) < lx + label_w as i64
                    && (y as i64) >= ly
                    && (y as i64) < ly + label_h as i64;
                let src = if in_slot {
                    // The recess the label sits in: the card's own colour there, held back a
                    // little, so a rounded label's corners read as the well it is stuck into.
                    [
                        (self.recess[0] as f32 * 0.78) as u8,
                        (self.recess[1] as f32 * 0.78) as u8,
                        (self.recess[2] as f32 * 0.78) as u8,
                        255,
                    ]
                } else if !self.tint {
                    // A picture of a cart keeps the colour it came in: the shell colour belongs to
                    // the machine's plastic, and this is not plastic, it is a photograph of some
                    // particular cart somebody owned.
                    [p[0], p[1], p[2], p[3]]
                } else {
                    let luma = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
                    // The dark theme paints the shell colour through the art's own shading, which
                    // is what the art is drawn for. Under the light theme that came out too dark
                    // against paler furniture, so a cart keeps its own colour there and the
                    // Advance cart — whose art is a dark grey drawing — is repainted a light one.
                    let (base, k) = match crate::palette::mode() {
                        crate::palette::Mode::Light => match light {
                            None => ([p[0], p[1], p[2]], 1.0),
                            Some(flat) => (flat, (luma / self.luma).clamp(0.0, 1.6)),
                        },
                        crate::palette::Mode::Dark => {
                            let c = match shell.finish {
                                Finish::Solid => shell.colour,
                                Finish::Translucent => shell.colour,
                            };
                            (c, (luma / self.luma).clamp(0.0, 1.6))
                        }
                    };
                    [
                        (base[0] as f32 * k).min(255.0) as u8,
                        (base[1] as f32 * k).min(255.0) as u8,
                        (base[2] as f32 * k).min(255.0) as u8,
                        p[3],
                    ]
                };
                let d = ((y * face_w + x) * 4) as usize;
                rgba[d..d + 4].copy_from_slice(&src);
            }
        }
        // The label itself, clipped to whatever shape the card drew its slot with.
        if let (Some(l), true) = (l, label_w > 0 && label_h > 0) {
            let mx = l.x1 - l.x0 + 1;
            let my = l.y1 - l.y0 + 1;
            for y in 0..label_h {
                for x in 0..label_w {
                    // Where this face pixel lands in the art's own slot mask.
                    let ax = l.x0 + (x as f32 * mx as f32 / label_w as f32) as u32;
                    let ay = l.y0 + (y as f32 * my as f32 / label_h as f32) as u32;
                    if l.mask[(ay.min(self.h - 1) * self.w + ax.min(self.w - 1)) as usize] == 0 {
                        continue;
                    }
                    let s = ((y * label_w + x) * 4) as usize;
                    let a = label[s + 3] as u32;
                    if a == 0 {
                        continue;
                    }
                    let d = (((y as i64 + ly).max(0) as u32 * face_w
                        + (x as i64 + lx).max(0) as u32)
                        * 4) as usize;
                    if d + 4 > rgba.len() {
                        continue;
                    }
                    for c in 0..3 {
                        let dst = rgba[d + c] as u32;
                        let src = label[s + c] as u32;
                        rgba[d + c] = ((src * a + dst * (255 - a) + 127) / 255) as u8;
                    }
                }
            }
        }
        CartFace {
            rgba,
            w: face_w,
            h: face_h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot is the magenta and **only** the magenta: a mark is `#ff00ff`, and a pixel that is
    /// half magenta and half shell is not one.
    ///
    /// Which is the one thing the art has to keep — see the note in `read` — and the reason this
    /// test exists is that it is also the one thing that is tempting to "fix" in code. A reader
    /// who finds a pink rim round a cover and relaxes the test until it goes away has moved the
    /// slot's edge off the edge the art drew: the slot is now a pixel wider, the label is masked
    /// to a shape nobody drew, and every pale shell on the card is one step from being called a
    /// mark. The rim belongs to the art, and the art is fixed in a paint program, not here.
    #[test]
    fn the_slot_is_the_magenta_and_only_the_magenta() {
        // Pure, and enough of each: the mark.
        assert!(is_mark(&[255, 0, 255, 255]));
        assert!(is_mark(&[220, 40, 210, 255]));
        // Half of each, which is what an antialiased edge is: not the slot.
        assert!(!is_mark(&[220, 130, 220, 255]));
        // A grey shell and a red shell, neither of which is magenta at all.
        assert!(!is_mark(&[200, 200, 200, 255]));
        assert!(!is_mark(&[220, 40, 40, 255]));
        // Transparent, which is not cart and so not the slot either.
        assert!(!is_mark(&[255, 0, 255, 0]));
    }
}
