//! Writes the drawn board, chip and core name out as PNGs so they can be looked at.
//!
//! The board and the chip are drawn in code now rather than rastered from art, and the only way
//! to tell whether a drawing is any good is to look at it.
//!
//!     cargo run -p slot-ui --example board_preview -- <a directory to write into>

use slot_store::{Cart, Core, System};
use slot_ui::{board_face, chip_face, core_name_face};

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

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
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
    }
    let chip = chip_face(Core::Mgba);
    write(&chip.rgba, chip.w, chip.h, &format!("{out}/chip.png"));
    let name = core_name_face(Core::Mgba);
    write(&name.rgba, name.w, name.h, &format!("{out}/name.png"));
}
