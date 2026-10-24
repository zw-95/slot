//! A cell that is a **picture of the panel** with the game that is up on it.
//!
//! The board of screen fits is the one table whose entries are not colours. A fit is not
//! something a strip of shades can say — what tells two of them apart is how much of the panel
//! the picture takes and which way the black runs, which is a picture, and the only honest picture
//! of it is the game itself, drawn where that fit would draw it.
//!
//! So a cell is the frame the core last produced, put on the panel at the fit's rect and scaled
//! down to the cell, with the panel's own black round it. Nothing here is a preview of a
//! preference: it *is* the preference, at the size a cell can carry.

use crate::CartFace;
use slot_gfx::{OUT_H, OUT_W};

/// What the panel looks like with the machine's picture on it at one fit.
///
/// `shot` is the frame that is up — RGBA at its own size, the size the core produced — and `rect`
/// is where this fit puts it on the panel, in panel pixels. The panel is black wherever the
/// picture is not, which is the whole point: on a Game Boy the integer fit leaves a wide frame of
/// black down both sides and the fill fit leaves almost none, and that difference is the choice.
///
/// **No frame is a fully transparent face**, which is the caller's cue to draw the shape without
/// the picture rather than a cell with a hole in it: a board opened in the moment before the core
/// had published anything still has to say which fit is which.
///
/// Nearest neighbour, both ways. A 160x144 frame scaled to a cell is already an act of violence,
/// and a bilinear one would hand back a soft grey blur — which is not what any of the three fits
/// looks like, and would make the one that is supposed to be crisp indistinguishable from the one
/// that is not.
/// The game that is up with **one of the apertures laid over it** — the picture a cell of the
/// board of looks is a picture of.
///
/// `table` is the mask: three by three multipliers per source pixel, which is what an aperture
/// *is*. The cell is scaled down from the frame, so a triad is smaller than a pixel by the time it
/// is drawn — which is exactly the truth about it: at a cell's size an LCD mask reads as fine
/// diagonal colour and a scanline reads as darker rows, and that is also how they read on the
/// screen when the game is small. A cell that lied about that by drawing the table at 1:1 would
/// be a cell of nine colours, which is the one picture every look shares.
///
/// `flat` — every multiplier 255 — hands back the picture untouched, which is what "no aperture"
/// looks like and why the board's own `OFF` cell is drawn black instead: an untouched picture is
/// not a look, it is the absence of one, and a cell showing the game as it is would be a cell
/// saying nothing.
pub fn mask_thumb(
    shot: Option<(&[u8], u32, u32)>,
    table: [[[u8; 3]; 3]; 3],
    w: u32,
    h: u32,
) -> CartFace {
    let mut face = CartFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    let Some((src, sw, sh)) = shot else {
        return face;
    };
    if sw == 0 || sh == 0 {
        return face;
    }
    for y in 0..h {
        for x in 0..w {
            // The middle of this cell pixel, in the frame's own pixels — and where it falls
            // *inside* the source pixel it lands in, which is the third of it the mask is for.
            let fx = (x as f32 + 0.5) * sw as f32 / w as f32;
            let fy = (y as f32 + 0.5) * sh as f32 / h as f32;
            let sx = (fx as u32).min(sw - 1);
            let sy = (fy as u32).min(sh - 1);
            let cx = ((fx - fx.floor()) * 3.0) as usize;
            let cy = ((fy - fy.floor()) * 3.0) as usize;
            let m = table[cy.min(2)][cx.min(2)];
            let s = ((sy * sw + sx) * 4) as usize;
            let i = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                face.rgba[i + c] = ((src[s + c] as u32 * m[c] as u32) / 255) as u8;
            }
            face.rgba[i + 3] = 255;
        }
    }
    face
}

/// A cell of nothing: the screen with the aperture switched off, which is a black rectangle and
/// not a picture of the game.
///
/// That is a deliberate lie about the game and the truth about the setting: `OFF` is the one
/// entry on the board that is not a look but the absence of one, and a cell showing the game
/// unmasked would be indistinguishable from the cell next to it at LCD 25 on a dark scene.
pub fn off_thumb(w: u32, h: u32) -> CartFace {
    let mut face = CartFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    for px in face.rgba.chunks_exact_mut(4) {
        px[3] = 255;
    }
    face
}

/// The game that is up **inside one of the screen frames** — the picture a cell of the board of
/// frames is a picture of.
///
/// `art` is the frame: a full-panel sheet with a window cut out of the middle, which is what a
/// screen overlay *is*. It is laid over the panel the same way it is laid over the screen — the
/// picture showing through the window, the machine's own case everywhere else — so a cell is not
/// a swatch of the frame's colours but the frame *on the game*, which is the only thing a player
/// choosing between frames is choosing between.
///
/// Nearest neighbour again, and for the same reason: a cell is a hundred and eighty pixels wide
/// and a frame is a sheet of fine bevels and shadows, and softening them would take away the only
/// detail that tells two machines' cases apart at this size.
pub fn overlay_thumb(
    shot: Option<(&[u8], u32, u32)>,
    rect: (f32, f32, f32, f32),
    art: Option<(&[u8], u32, u32)>,
    w: u32,
    h: u32,
) -> CartFace {
    let mut face = panel_thumb(shot, rect, w, h);
    let Some((src, sw, sh)) = art else {
        return face;
    };
    if sw == 0 || sh == 0 {
        return face;
    }
    for y in 0..h {
        for x in 0..w {
            let sx = ((x as f32 + 0.5) * sw as f32 / w as f32) as u32;
            let sy = ((y as f32 + 0.5) * sh as f32 / h as f32) as u32;
            let s = ((sy.min(sh - 1) * sw + sx.min(sw - 1)) * 4) as usize;
            if s + 3 >= src.len() {
                continue;
            }
            let a = src[s + 3] as f32 / 255.0;
            if a <= 0.0 {
                continue; // The window: the picture the panel already drew shows through.
            }
            let i = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                let over = src[s + c] as f32 * a;
                let under = face.rgba[i + c] as f32 * (1.0 - a);
                face.rgba[i + c] = (over + under).round() as u8;
            }
            face.rgba[i + 3] = 255;
        }
    }
    face
}

pub fn panel_thumb(
    shot: Option<(&[u8], u32, u32)>,
    rect: (f32, f32, f32, f32),
    w: u32,
    h: u32,
) -> CartFace {
    let mut face = CartFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    let Some((src, sw, sh)) = shot else {
        return face;
    };
    if sw == 0 || sh == 0 {
        return face;
    }
    let (rx, ry, rw, rh) = rect;
    if rw <= 0.0 || rh <= 0.0 {
        return face;
    }
    for y in 0..h {
        for x in 0..w {
            // The middle of this cell pixel, in panel pixels — where on the panel it stands.
            let px = (x as f32 + 0.5) * OUT_W as f32 / w as f32;
            let py = (y as f32 + 0.5) * OUT_H as f32 / h as f32;
            let i = ((y * w + x) * 4) as usize;
            face.rgba[i + 3] = 255;
            if px < rx || py < ry || px >= rx + rw || py >= ry + rh {
                continue; // The panel's own black: the letterbox, which is half of the choice.
            }
            // And where that lands in the frame the core produced.
            let sx = ((px - rx) / rw * sw as f32) as u32;
            let sy = ((py - ry) / rh * sh as f32) as u32;
            let s = ((sy.min(sh - 1) * sw + sx.min(sw - 1)) * 4) as usize;
            if s + 3 >= src.len() {
                continue;
            }
            let a = src[s + 3];
            for c in 0..3 {
                // Over the panel's black, so a frame with transparency in it — a core that has
                // published nothing but zeroes — reads as the black it is.
                let v = src[s + c] as f32 * (a as f32 / 255.0);
                face.rgba[i + c] = v.round() as u8;
            }
        }
    }
    face
}
