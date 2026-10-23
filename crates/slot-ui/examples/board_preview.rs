//! Writes the drawn board, chip and core name out as PNGs so they can be looked at.
//!
//! The board and the chip are drawn in code now rather than rastered from art, and the only way
//! to tell whether a drawing is any good is to look at it.
//!
//!     cargo run -p slot-ui --example board_preview -- <a directory to write into>

use slot_store::{Cart, Core, System};
use slot_ui::{
    board_face, chip_face, core_dot_face, core_name_face, CORE_DOT, CORE_DOT_DIM, CORE_DOT_GAP,
    CORE_DOT_LIFT,
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

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
    let chip = chip_face(Core::Mgba);
    let name = core_name_face(Core::Mgba);
    let dot = core_dot_face();
    write(&chip.rgba, chip.w, chip.h, &format!("{out}/chip.png"));
    write(&name.rgba, name.w, name.h, &format!("{out}/name.png"));
    write(&dot.rgba, dot.w, dot.h, &format!("{out}/dot.png"));
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
            let row = count as f32 * CORE_DOT + (count - 1) as f32 * CORE_DOT_GAP;
            let x0 = (face.w as f32 - row) / 2.0;
            let y = (face.h - chip.h) as i32 / 2 - CORE_DOT_LIFT as i32 - CORE_DOT as i32;
            for i in 0..count {
                let x = (x0 + i as f32 * (CORE_DOT + CORE_DOT_GAP)) as i32;
                paste_a(
                    &mut rgba, face.w, face.h, &dot.rgba, dot.w, dot.h, x, y, CORE_DOT_DIM,
                );
                paste_a(
                    &mut rgba,
                    face.w,
                    face.h,
                    &dot.rgba,
                    dot.w,
                    dot.h,
                    x,
                    y,
                    if i == count / 2 { 1.0 } else { 0.0 },
                );
            }
        }
        write(&rgba, face.w, face.h, &format!("{out}/opened_{system:?}.png"));
    }
}
