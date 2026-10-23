//! Writes cart faces — the fronts the shelf draws — out as PNGs so they can be looked at.
//!
//! A shell is a drawing like any other, and the only way to tell whether a drawing is any good
//! is to look at it. One cart per machine, plus that machine's blank, because the blank is the
//! face a jump across the alphabet spends most of its time showing.
//!
//! Given `--art <a card root>`, the cart art on that card is loaded first and drawn instead of
//! the built-in shell, which is how a cart you drew yourself gets looked at without a device:
//! what comes back is the card's own art with the game's label pasted into the slot it marked.
//!
//!     cargo run -p slot-ui --example cart_preview -- <a directory to write into>
//!     cargo run -p slot-ui --example cart_preview -- <dir> --art <card root>

use slot_store::{Cart, System};
use slot_ui::{cart_face, cart_placeholder, size_for};

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
    enc.write_header()
        .expect("header")
        .write_image_data(rgba)
        .expect("data");
    println!("{path}  {w}x{h}");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
    // The card's own art, if it has any. What `summary` prints is the check that it was read at
    // all: a machine with no art and a machine whose art was refused look identical on the
    // shelf, and only the summary tells them apart.
    if let Some(root) = args
        .iter()
        .position(|a| a == "--art")
        .and_then(|i| args.get(i + 1))
    {
        slot_ui::install_cart_art(slot_ui::load_cart_art(std::path::Path::new(root)));
        println!("cart art: {}", slot_ui::cart_art_summary());
    }
    for (system, stem, ext, code) in [
        (
            System::Gba,
            "Pokemon - Emerald Version (USA, Europe)",
            "gba",
            "BPEE",
        ),
        (System::Gb, "Tetris (World)", "gb", "DMG-TR"),
        (System::Gbc, "Pokemon - Crystal Version (USA)", "gbc", "CGB-BP"),
        (System::Sfc, "Super Mario World (USA)", "sfc", "SNS-MW"),
        (System::Fc, "Super Mario Bros.", "nes", "NES-MB"),
    ] {
        let face = cart_face(&cart(stem, code, ext));
        write(
            &face.rgba,
            face.w,
            face.h,
            &format!("{out}/cart_{system:?}.png"),
        );
        let blank = cart_placeholder(size_for(system), system);
        write(
            &blank.rgba,
            blank.w,
            blank.h,
            &format!("{out}/blank_{system:?}.png"),
        );
    }
}
