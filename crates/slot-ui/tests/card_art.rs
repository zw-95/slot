//! A card may carry a platform's own art: any number of grey templates, and where each one takes
//! its label. Anything wrong with one of them leaves that one out; anything wrong with all of
//! them leaves the built-in shape whole.

use std::path::Path;
use std::sync::Mutex;

use slot_store::{scan, Cart, Platform};
use slot_ui::{
    cart_box, cart_face, label_size, load_card_art, pick_of, Media, CARTS, GB_CART_H, GB_CART_W,
    GB_LABEL_H, GB_LABEL_W,
};
use tempfile::TempDir;

/// The art is one global, so the tests here take turns.
static TURN: Mutex<()> = Mutex::new(());

const LABEL: &str = "label = 5 5 30 30";
const LABEL_PX: (u32, u32) = (30, 30);

fn root() -> TempDir {
    let d = tempfile::tempdir().expect("tempdir");
    for sub in ["Games", "Labels", "Saves", "States", "System", "Config"] {
        std::fs::create_dir_all(d.path().join(sub)).expect("create dir");
    }
    d
}

fn write_png(path: &Path, w: u32, h: u32, rgba: [u8; 4]) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("create dir");
    }
    let px: Vec<u8> = (0..w * h).flat_map(|_| rgba).collect();
    let f = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .expect("png header")
        .write_image_data(&px)
        .expect("png data");
}

/// The size the built-in shape is drawn at, which is the size a picture has to be when the shape
/// is measured in absolute pixels all the way through.
fn box_of(platform: Platform) -> (u32, u32) {
    match Media::of(platform) {
        Some(m) => m.size(),
        None => (GB_CART_W, GB_CART_H),
    }
}

/// Puts pictures and a configuration under `Carts/<dir>`, and reads the card. Every picture is
/// `at` big, apart from the ones named in `sizes`.
fn card_with(dir: &str, at: (u32, u32), txt: Option<&str>, pics: &[&str]) -> TempDir {
    let d = root();
    let art = d.path().join(CARTS).join(dir);
    for name in pics {
        write_png(&art.join(name), at.0, at.1, [90, 90, 94, 255]);
    }
    if let Some(text) = txt {
        std::fs::write(art.join("cart.txt"), text).expect("write cart.txt");
    }
    load_card_art(d.path());
    d
}

/// The one cart in the card: a ROM of `dir` with a red label beside it.
fn cart(d: &TempDir, dir: &str, name: &str) -> Cart {
    std::fs::create_dir_all(d.path().join("Games").join(dir)).expect("create games dir");
    std::fs::write(d.path().join("Games").join(dir).join(name), vec![0u8; 0x200])
        .expect("write rom");
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    write_png(
        &d.path().join("Labels").join(dir).join(format!("{stem}.png")),
        64,
        64,
        [220, 30, 30, 255],
    );
    scan(d.path()).expect("scan").into_iter().next().expect("a cart")
}

fn nes(d: &TempDir) -> Cart {
    cart(d, "NES", "Game.nes")
}

fn gb(d: &TempDir) -> Cart {
    cart(d, "GB", "Game.gb")
}

/// What each game is drawn with, written beside its label.
fn pick(d: &TempDir, dir: &str, text: &str) {
    let labels = d.path().join("Labels").join(dir);
    std::fs::create_dir_all(&labels).expect("create dir");
    std::fs::write(labels.join("cart_pick.txt"), text).expect("write cart_pick.txt");
}

/// The size the built-in label box is, which is what a turned-down card falls back to.
fn built_in(platform: Platform) -> (u32, u32) {
    match Media::of(platform) {
        Some(m) => {
            let (_, _, w, h) = m.label();
            (w, h)
        }
        None => (GB_LABEL_W, GB_LABEL_H),
    }
}

#[test]
fn a_card_with_its_own_art_uses_it() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = card_with("NES", box_of(Platform::Nes), Some(LABEL), &["cart.png"]);
    assert_eq!(label_size(&nes(&d)), LABEL_PX);
}

#[test]
fn a_medium_may_be_drawn_at_a_size_of_its_own() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let (w, h) = box_of(Platform::Nes);
    let d = card_with("NES", (w + 40, h - 20), Some(LABEL), &["cart.png"]);
    let cart = nes(&d);
    assert_eq!(label_size(&cart), LABEL_PX, "the label box still applies");
    assert_eq!(
        cart_box(&cart),
        (w + 40, h - 20),
        "the cart is drawn at the size its picture is"
    );
}

#[test]
fn a_game_boy_picture_of_the_wrong_size_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Gb);
    let d = card_with("GB", (GB_CART_W + 8, GB_CART_H), Some(LABEL), &["cart.png"]);
    assert_eq!(label_size(&gb(&d)), want);
}

#[test]
fn a_label_box_that_leaves_the_picture_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = card_with(
        "NES",
        box_of(Platform::Nes),
        Some("label = 999 5 30 30"),
        &["cart.png"],
    );
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_label_that_is_not_four_numbers_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    for bad in ["label = 5 5", "label = 5 5 30 -30", "label = a b c d"] {
        let d = card_with("NES", box_of(Platform::Nes), Some(bad), &["cart.png"]);
        assert_eq!(label_size(&nes(&d)), want, "for {bad:?}");
    }
}

#[test]
fn a_configuration_with_no_label_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = card_with("NES", box_of(Platform::Nes), Some("corner = 6"), &["cart.png"]);
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_picture_with_no_configuration_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = card_with("NES", box_of(Platform::Nes), None, &["cart.png"]);
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_picture_that_does_not_decode_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = root();
    let art = d.path().join(CARTS).join("NES");
    std::fs::create_dir_all(&art).expect("create dir");
    std::fs::write(art.join("cart.png"), b"not a png").expect("write png");
    std::fs::write(art.join("cart.txt"), LABEL).expect("write cart.txt");
    load_card_art(d.path());
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_card_with_no_art_changes_nothing() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    assert_eq!(label_size(&nes(&d)), want);
}

/// More than one picture, each with the label box that applies to it.
#[test]
fn each_template_takes_its_own_label_box() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let txt = "template = 1.png 2.png\nlabel = 5 5 30 30\nlabel.2.png = 5 5 40 40";
    let d = card_with("NES", box_of(Platform::Nes), Some(txt), &["1.png", "2.png"]);
    assert_eq!(label_size(&nes(&d)), LABEL_PX, "the first is the one used");
    pick(&d, "NES", "Game = 0 0\n");
    load_card_art(d.path());
    assert_eq!(label_size(&nes(&d)), LABEL_PX);
    pick(&d, "NES", "Game = 1 0\n");
    load_card_art(d.path());
    assert_eq!(label_size(&nes(&d)), (40, 40), "the card picked the second");
}

/// A picture that does not hold up is dropped, and the ones either side of it are kept.
#[test]
fn a_template_that_does_not_hold_up_is_dropped_and_the_rest_are_kept() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let txt = "template = 1.png 2.png\nlabel = 5 5 30 30\nlabel.2.png = 9999 5 30 30";
    let d = card_with("NES", box_of(Platform::Nes), Some(txt), &["1.png", "2.png"]);
    assert_eq!(label_size(&nes(&d)), LABEL_PX, "the good one is still there");
    pick(&d, "NES", "Game = 1 0\n");
    load_card_art(d.path());
    assert_eq!(
        label_size(&nes(&d)),
        LABEL_PX,
        "the one dropped is not offered, so the pick falls back to the first"
    );
}

/// A pick past the end of what the card offers is the first, not nothing at all.
#[test]
fn a_pick_past_the_end_falls_back_to_the_first() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let txt = "template = 1.png 2.png\nlabel = 5 5 30 30\nlabel.2.png = 5 5 40 40";
    let d = card_with("NES", box_of(Platform::Nes), Some(txt), &["1.png", "2.png"]);
    pick(&d, "NES", "Game = 9 0\n");
    load_card_art(d.path());
    let cart = nes(&d);
    assert_eq!(pick_of(&cart).template, 0, "{:?}", pick_of(&cart));
    assert_eq!(label_size(&cart), LABEL_PX);
}

/// Red, as the label beside every cart here is painted.
fn red(face: &slot_ui::CartFace, x: u32, y: u32) -> bool {
    let i = ((y * face.w + x) * 4) as usize;
    face.rgba[i] > face.rgba[i + 1] + 40 && face.rgba[i] > face.rgba[i + 2] + 40
}

/// A corner the card rounds off is a corner the label does not print in.
#[test]
fn a_rounded_corner_keeps_the_label_out_of_it() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = card_with(
        "NES",
        box_of(Platform::Nes),
        Some("label = 5 5 30 30\ncorner = 8"),
        &["cart.png"],
    );
    let face = cart_face(&nes(&d));
    assert!(red(&face, 20, 20), "the middle of the label box lost its label");
    assert!(!red(&face, 5, 5), "the label printed into the rounded corner");
}

#[test]
fn without_a_corner_the_label_reaches_the_corner() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = card_with("NES", box_of(Platform::Nes), Some(LABEL), &["cart.png"]);
    let face = cart_face(&nes(&d));
    assert!(red(&face, 5, 5), "the corner was rounded without being asked");
}

/// The card's own picture is the outline: its alpha is what the face is clipped to.
#[test]
fn the_cards_picture_clips_the_face() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let plain = cart_face(&nes(&d));
    let d = card_with("NES", box_of(Platform::Nes), Some(LABEL), &["cart.png"]);
    let own = cart_face(&nes(&d));
    assert_eq!((own.w, own.h), (plain.w, plain.h));
    // Our picture is opaque all over, where the built-in shell is not at its corners.
    let corner = |f: &slot_ui::CartFace| f.rgba[3] as u32;
    assert_eq!(corner(&own), 255);
    assert!(corner(&plain) < 255, "the built-in shell is cut at a corner");
}
