//! The picture a state keeps of itself: **the game as the screen was showing it**, not the core's
//! whole frame with its black bars still on and its shape still to be decided.
//!
//! A state's thumbnail used to be the core's frame straight from the core. What the screen showed
//! was a crop of that frame, stretched to the screen's shape, so the polaroid never held the
//! picture that had been on the screen — and a crop of a small frame, blown up to the size a
//! polaroid is drawn at, is a blur.

use slot_gfx::{OUT_H, OUT_W};

/// What the screen was showing when the state was taken.
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    /// The part of the frame that is picture, as fractions of it: left, top, width, height.
    pub src: [f32; 4],
    /// Whether the picture was being stretched to the screen's shape.
    pub stretch: bool,
}

/// The picture the screen showed, as a png. The core's own frame when the part of it to take
/// cannot be read, which is no worse than what was kept before.
pub fn png(frame: &[u8], size: (u32, u32), shot: &Shot) -> Option<Vec<u8>> {
    let (x, y, w, h) = crop(size, shot.src);
    let Some(cut) = sub(frame, size, x, y, w, h) else {
        return crate::thumb::png(frame, size);
    };
    let (rgba, size) = match shot.stretch {
        true => stretched(&cut, (w, h)),
        false => (cut, (w, h)),
    };
    crate::thumb::png(&rgba, size)
}

/// The part of a `w` by `h` frame that `src` names, in whole pixels inside it.
fn crop((w, h): (u32, u32), src: [f32; 4]) -> (u32, u32, u32, u32) {
    let whole = |at: f32, len: f32| (at.max(0.0).min(1.0) * len).round() as u32;
    let (x, y) = (whole(src[0], w as f32), whole(src[1], h as f32));
    let (cw, ch) = (whole(src[2], w as f32), whole(src[3], h as f32));
    (
        x,
        y,
        cw.min(w.saturating_sub(x)).max(1),
        ch.min(h.saturating_sub(y)).max(1),
    )
}

/// `cw` by `ch` pixels of the frame, starting at `x`, `y`.
fn sub(frame: &[u8], (w, _): (u32, u32), x: u32, y: u32, cw: u32, ch: u32) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    for row in y..y + ch {
        let at = (
            ((row * w + x) * 4) as usize,
            (((row * w + x) + cw) * 4) as usize,
        );
        out.extend_from_slice(frame.get(at.0..at.1)?);
    }
    Some(out)
}

/// The picture squeezed to the screen's shape, **at its own pixel count**: what stretching does to
/// how it looks, without making the file any bigger than the picture already is. A card is written
/// to once per state, and a state's picture is not worth a megabyte.
fn stretched(rgba: &[u8], (w, h): (u32, u32)) -> (Vec<u8>, (u32, u32)) {
    let area = (w.max(1) as f32 * h.max(1) as f32).max(1.0);
    let shape = OUT_W as f32 / OUT_H as f32;
    let ow = (area * shape).sqrt().round().max(1.0) as u32;
    let oh = ((ow as f32 / shape).round()).max(1.0) as u32;
    (resample(rgba, (w, h), (ow, oh)), (ow, oh))
}

/// Nearest neighbour: a screenshot is not a photograph, and a soft one reads as a broken one.
fn resample(rgba: &[u8], (w, h): (u32, u32), (ow, oh): (u32, u32)) -> Vec<u8> {
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    for y in 0..oh {
        let sy = y as u64 * h as u64 / oh.max(1) as u64;
        for x in 0..ow {
            let sx = x as u64 * w as u64 / ow.max(1) as u64;
            let from = ((sy * w as u64 + sx) * 4) as usize;
            let to = ((y * ow + x) * 4) as usize;
            out[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
        }
    }
    out
}
