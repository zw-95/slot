//! What a border on the card does to the carts drawn from it.
//!
//! Two claims, both of them about a picture the user dropped on the card rather than about a
//! drawing that ships in the binary: **a border is the machine's cart**, ahead of the drawing in
//! `Carts`, and **a border is the machine's outline**, ahead of the one in code. Neither is a
//! preference expressed in a setting; they are what the picture is for.

use std::path::Path;

use slot_store::System;
use slot_ui::{board_face, install_cart_art, load_cart_art, shell_for, CartFace};

/// A card carrying a PNG: `w` × `h`, `colour` where `inside` says, transparent everywhere else.
fn write_png(path: &Path, w: u32, h: u32, colour: [u8; 4], inside: impl Fn(u32, u32) -> bool) {
    let mut raw = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let p = if inside(x, y) {
                colour
            } else {
                [0, 0, 0, 0]
            };
            raw.extend_from_slice(&p);
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(&raw).unwrap();
    }
    std::fs::write(path, out).unwrap();
}

fn cart() -> slot_store::Cart {
    slot_store::Cart {
        initial: slot_store::initial("Tetris"),
        stem: "Tetris".to_string(),
        rom: std::path::PathBuf::from("Games/Tetris.gba"),
        label: None,
        code: String::new(),
        title: "TETRIS".to_string(),
    }
}

fn card_with_borders() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("System/Carts")).unwrap();
    std::fs::create_dir_all(d.path().join("System/Borders")).unwrap();
    // The drawing the binary's own card carries: red, and a full rectangle.
    write_png(
        &d.path().join("System/Carts/cart_gba.png"),
        64,
        64,
        [220, 30, 30, 255],
        |_, _| true,
    );
    // And the border dropped on the card: blue, and only its left half is cart — the shape a
    // board cut from it cannot help but show.
    write_png(
        &d.path().join("System/Borders/gba1.png"),
        64,
        64,
        [40, 60, 220, 255],
        |x, _| x < 32,
    );
    d
}

/// One test and not three, because what is installed is a **global**: three tests running at once
/// would each be asking about a card while another was putting its own on the table. What is worth
/// saying here is the difference one card makes from another, and that has to be said in order.
#[test]
fn what_a_border_on_the_card_does() {
    // A border is a picture of a particular cart, and it brought its own colour. The shell colour
    // belongs to a shell — grey plastic waiting to be coloured — and painting it over a
    // photograph would paint over the thing the photograph is of.
    let d = card_with_borders();
    install_cart_art(load_cart_art(d.path()));
    let art = slot_ui::cart_art_for(System::Gba).expect("the border was on the card");
    let shell = shell_for("");
    let face = art.face(&shell, &[], 0, 0, 64, 64, None);
    let at = |x: u32, y: u32| {
        let i = ((y * face.w + x) * 4) as usize;
        [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2], face.rgba[i + 3]]
    };
    assert_eq!(at(8, 32), [40, 60, 220, 255], "the border, in its own colour");
    assert_eq!(at(48, 32)[3], 0, "and nothing outside its shape");

    // The board a cart opens onto is cut with the border's outline, not with the shape written
    // down for that machine in code: a cart that comes apart in the picker comes apart into the
    // shape it was on the shelf.
    let face = board_face(&cart(), System::Gba);
    let alpha_at = |x: u32, y: u32| face.rgba[((y * face.w + x) * 4 + 3) as usize];
    let right_of_the_border = face.w - face.w / 4;
    assert!(alpha_at(face.w / 4, face.h / 2) > 200, "the border's left half is cart");
    assert_eq!(
        alpha_at(right_of_the_border, face.h / 2),
        0,
        "its right half is not, and a board cut from a rectangle would have been"
    );

    // Without a border nothing moves: the drawing in `Carts` is still the cart. A card that has
    // never been given the directory is every card there is.
    let plain = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(plain.path().join("System/Carts")).unwrap();
    write_png(
        &plain.path().join("System/Carts/cart_gba.png"),
        64,
        64,
        [220, 30, 30, 255],
        |_, _| true,
    );
    install_cart_art(load_cart_art(plain.path()));
    let bare: CartFace = board_face(&cart(), System::Gba);
    let bare_alpha = |x: u32, y: u32| bare.rgba[((y * bare.w + x) * 4 + 3) as usize];
    assert!(
        bare_alpha(right_of_the_border, bare.h / 2) > 200,
        "the same point is cart when the card carries a whole drawing instead"
    );
}
