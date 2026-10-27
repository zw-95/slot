//! A card may carry a platform's own art: a grey template and where its label prints. Anything
//! wrong with either leaves the built-in shape whole.

use std::path::Path;
use std::sync::Mutex;

use slot_store::{scan, Cart, Platform};
use slot_ui::{cart_box, cart_face, label_size, load_card_art, Media, CARTS, GB_LABEL_H, GB_LABEL_W};
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

/// Puts a picture and a configuration under `Carts/<dir>`, and reads the card.
fn card_with(dir: &str, png_at: Option<(u32, u32)>, txt: Option<&str>) -> TempDir {
    let d = root();
    let art = d.path().join(CARTS).join(dir);
    if let Some((w, h)) = png_at {
        write_png(&art.join("cart.png"), w, h, [90, 90, 94, 255]);
    }
    if let Some(text) = txt {
        std::fs::create_dir_all(&art).expect("create dir");
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
    let d = card_with("NES", Some(cart_box(Platform::Nes)), Some(LABEL));
    assert_eq!(label_size(&nes(&d)), LABEL_PX);
}

#[test]
fn a_picture_of_the_wrong_size_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let (w, h) = cart_box(Platform::Nes);
    let d = card_with("NES", Some((w + 8, h)), Some(LABEL));
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_label_box_that_leaves_the_picture_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = card_with("NES", Some(cart_box(Platform::Nes)), Some("label = 999 5 30 30"));
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_label_that_is_not_four_numbers_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    for bad in ["label = 5 5", "label = 5 5 30 -30", "label = a b c d"] {
        let d = card_with("NES", Some(cart_box(Platform::Nes)), Some(bad));
        assert_eq!(label_size(&nes(&d)), want, "for {bad:?}");
    }
}

#[test]
fn a_configuration_with_no_label_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = card_with("NES", Some(cart_box(Platform::Nes)), Some("corner = 6"));
    assert_eq!(label_size(&nes(&d)), want);
}

#[test]
fn a_picture_with_no_configuration_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Nes);
    let d = card_with("NES", Some(cart_box(Platform::Nes)), None);
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

#[test]
fn a_game_boy_pak_takes_the_cards_art() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = card_with("GB", Some(cart_box(Platform::Gb)), Some(LABEL));
    assert_eq!(label_size(&gb(&d)), LABEL_PX);
}

#[test]
fn a_game_boy_picture_of_the_wrong_size_is_ignored() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = root();
    load_card_art(d.path());
    let want = built_in(Platform::Gb);
    let d = card_with("GB", Some((312 + 8, 337)), Some(LABEL));
    assert_eq!(label_size(&gb(&d)), want);
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
        Some(cart_box(Platform::Nes)),
        Some("label = 5 5 30 30\ncorner = 8"),
    );
    let face = cart_face(&nes(&d));
    assert!(red(&face, 20, 20), "the middle of the label box lost its label");
    assert!(!red(&face, 5, 5), "the label printed into the rounded corner");
}

#[test]
fn without_a_corner_the_label_reaches_the_corner() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let d = card_with("NES", Some(cart_box(Platform::Nes)), Some(LABEL));
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
    let d = card_with("NES", Some(cart_box(Platform::Nes)), Some(LABEL));
    let own = cart_face(&nes(&d));
    assert_eq!((own.w, own.h), (plain.w, plain.h));
    // Our picture is opaque all over, where the built-in shell is not at its corners.
    let corner = |f: &slot_ui::CartFace| f.rgba[3] as u32;
    assert_eq!(corner(&own), 255);
    assert!(corner(&plain) < 255, "the built-in shell is cut at a corner");
}
