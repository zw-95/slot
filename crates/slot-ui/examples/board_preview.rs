//! Writes the drawn board, chip and core name out as PNGs so they can be looked at.
//!
//! The board and the chip are drawn in code now rather than rastered from art, and the only way
//! to tell whether a drawing is any good is to look at it.
//!
//!     cargo run -p slot-ui --example board_preview -- <a directory to write into>

use slot_store::{Cart, Core, System};
use slot_ui::{
    board_at, board_face, chip_face, core_dot_face, core_name_face, CORE_DOT, CORE_DOT_GAP,
    CORE_DOT_HALO, CORE_DOT_LIFT, OUT_H, OUT_W,
};

fn cart(stem: &str, code: &str, ext: &str) -> Cart {
    Cart {
        initial: slot_store::initial(&slot_store::clean_label(stem)),
        stem: stem.into(),
        rom: format!("Games/{stem}.{ext}").into(),
        label: None,
        code: code.into(),
        title: stem.to_uppercase(),
    }
}

fn write(rgba: &[u8], w: u32, h: u32, path: &str) {
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc
        .write_header()
        .expect("header")
        .write_image_data(rgba)
        .expect("data");
    println!("{path}  {w}x{h}");
}

/// `src` (with alpha) onto opaque `dst`, at a position that may fall partly outside it.
fn paste(
    dst: &mut [u8],
    dw: u32,
    dh: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    x: i32,
    y: i32,
) {
    for sy in 0..sh {
        for sx in 0..sw {
            let (dx, dy) = (x + sx as i32, y + sy as i32);
            if dx < 0 || dy < 0 || dx >= dw as i32 || dy >= dh as i32 {
                continue;
            }
            let s = ((sy * sw + sx) * 4) as usize;
            let a = src[s + 3] as f32 / 255.0;
            if a == 0.0 {
                continue;
            }
            let d = ((dy as u32 * dw + dx as u32) * 4) as usize;
            for c in 0..3 {
                dst[d + c] = (src[s + c] as f32 * a + dst[d + c] as f32 * (1.0 - a)).round() as u8;
            }
        }
    }
}

/// `src` onto `dst` at `alpha` on top of its own: how the picker draws a dot that is partly
/// lit, which is one dim draw and one lit one over it.
fn paste_a(
    dst: &mut [u8],
    dw: u32,
    dh: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    x: i32,
    y: i32,
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    for sy in 0..sh {
        for sx in 0..sw {
            let (dx, dy) = (x + sx as i32, y + sy as i32);
            if dx < 0 || dy < 0 || dx >= dw as i32 || dy >= dh as i32 {
                continue;
            }
            let s = ((sy * sw + sx) * 4) as usize;
            let a = (src[s + 3] as f32 / 255.0) * alpha;
            if a == 0.0 {
                continue;
            }
            let d = ((dy as u32 * dw + dx as u32) * 4) as usize;
            for c in 0..3 {
                dst[d + c] = (src[s + c] as f32 * a + dst[d + c] as f32 * (1.0 - a)).round() as u8;
            }
        }
    }
}

/// The panel's own ground: a board is looked at against it, because where the board sits on the
/// panel is half of what the drawing is.
const PANEL_GROUND: [u8; 3] = [0x14, 0x14, 0x18];

/// `src` onto `dst`, scaled to `w` by `h` — the board's own face is one size and the panel
/// draws it at another.
fn paste_scaled(
    dst: &mut [u8],
    dw: u32,
    dh: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    for dy in 0..h as i32 {
        for dx in 0..w as i32 {
            let (px, py) = (x as i32 + dx, y as i32 + dy);
            if px < 0 || py < 0 || px >= dw as i32 || py >= dh as i32 {
                continue;
            }
            let sx = ((dx as f32 * sw as f32 / w) as u32).min(sw - 1);
            let sy = ((dy as f32 * sh as f32 / h) as u32).min(sh - 1);
            let s = ((sy * sw + sx) * 4) as usize;
            let a = src[s + 3] as f32 / 255.0;
            if a == 0.0 {
                continue;
            }
            let d = ((py as u32 * dw + px as u32) * 4) as usize;
            for c in 0..3 {
                dst[d + c] = (src[s + c] as f32 * a + dst[d + c] as f32 * (1.0 - a)).round() as u8;
            }
        }
    }
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
    let chip = chip_face(Core::Mgba);
    let name = core_name_face(Core::Mgba);
    // The row above the chip is a row of lamps: every seat wears one that is out, and the one
    // in hand is the one burning.
    let lamp_off = core_dot_face(false);
    let lamp_on = core_dot_face(true);
    write(&chip.rgba, chip.w, chip.h, &format!("{out}/chip.png"));
    write(&name.rgba, name.w, name.h, &format!("{out}/name.png"));
    write(&lamp_off.rgba, lamp_off.w, lamp_off.h, &format!("{out}/lamp_off.png"));
    write(&lamp_on.rgba, lamp_on.w, lamp_on.h, &format!("{out}/lamp_on.png"));
    for (system, stem, ext, code) in [
        (
            System::Gba,
            "Pokemon - Emerald Version (USA, Europe)",
            "gba",
            "BPEE",
        ),
        (System::Gb, "Tetris (World)", "gb", "DMG-TR"),
        (System::Sfc, "Super Mario World (USA)", "sfc", "SNS-MW"),
    ] {
        let face = board_face(&cart(stem, code, ext), system);
        write(
            &face.rgba,
            face.w,
            face.h,
            &format!("{out}/board_{system:?}.png"),
        );
        // And what the picker draws over it: the chip in the middle of the board, and the name
        // in the middle of the chip. Which is the whole of this drawing's job, so it is what the
        // picture has to show.
        let mut rgba = face.rgba.clone();
        paste(
            &mut rgba,
            face.w,
            face.h,
            &chip.rgba,
            chip.w,
            chip.h,
            (face.w - chip.w) as i32 / 2,
            (face.h - chip.h) as i32 / 2,
        );
        paste(
            &mut rgba,
            face.w,
            face.h,
            &name.rgba,
            name.w,
            name.h,
            (face.w - name.w) as i32 / 2,
            (face.h - name.h) as i32 / 2,
        );
        // And the row above the chip: one dot per core this machine offers, the lit one the
        // core in hand — thirteen on the Super Nintendo and the middle one lit, which is what
        // that machine's list actually looks like.
        let count = match system {
            System::Sfc => 13,
            System::Gb | System::Gbc => 5,
            System::Gba => 6,
            System::Fc => 1,
        };
        if count > 1 {
            // A lamp is drawn a little wider than it is: the face carries the lit one's halo out
            // past the disc, and a lamp clipped to its own circle throws no light.
            let size = CORE_DOT + CORE_DOT_HALO * 2.0;
            let row = count as f32 * size + (count - 1) as f32 * CORE_DOT_GAP;
            let x0 = (face.w as f32 - row) / 2.0;
            let y = (face.h - chip.h) as i32 / 2 - CORE_DOT_LIFT as i32 - size as i32;
            for i in 0..count {
                let x = (x0 + i as f32 * (size + CORE_DOT_GAP)) as i32;
                paste_a(&mut rgba, face.w, face.h, &lamp_off.rgba, lamp_off.w, lamp_off.h, x, y, 1.0);
                if i == count / 2 {
                    paste_a(
                        &mut rgba, face.w, face.h, &lamp_on.rgba, lamp_on.w, lamp_on.h, x, y, 1.0,
                    );
                }
            }
        }
        write(&rgba, face.w, face.h, &format!("{out}/opened_{system:?}.png"));
        // And the whole panel, with the open cart on it where the panel puts it. A board is one
        // shape per machine and the panel is one shape for all of them, so this is the drawing
        // that shows whether a board sits in the middle of the panel or off in a corner of it.
        let mut panel = vec![0u8; (OUT_W * OUT_H * 4) as usize];
        for px in panel.chunks_exact_mut(4) {
            px[..3].copy_from_slice(&PANEL_GROUND);
            px[3] = 255;
        }
        let at = board_at(1.0, system);
        paste_scaled(
            &mut panel, OUT_W, OUT_H, &rgba, face.w, face.h, at.x, at.y, at.w, at.h,
        );
        write(&panel, OUT_W, OUT_H, &format!("{out}/panel_{system:?}.png"));
    }
}
