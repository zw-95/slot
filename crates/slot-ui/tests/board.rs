use slot_store::{Cart, Core, System};
use slot_ui::{
    board_at, board_face, chip_face, chip_shadow_face, lid_at, lift_of, on_board, padded,
    shelf_cart, shell_for, slide_of, socket_face, CartFace, Placed, BOARD_H, BOARD_W, CART_H,
    DEFAULT_SHELL, LID_TURN, SHADOW_H, SHADOW_W, SLIDE_SHARE, SLIDE_UP, SOCKET_H, SOCKET_W,
    TURN_PAD,
};

fn cart(stem: &str, code: &str) -> Cart {
    Cart {
        initial: bucket(stem),
        stem: stem.into(),
        rom: format!("Games/{stem}.gba").into(),
        label: None,
        code: code.into(),
        title: stem.to_uppercase(),
    }
}

/// The letter the ring files a cart under, derived the way `scan` derives it: from the name the
/// shelf shows. A fixture that made one up would pass while the real derivation was broken.
fn bucket(stem: &str) -> char {
    slot_store::initial(&slot_store::clean_label(stem))
}

fn rgb(face: &CartFace, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * face.w + x) * 4) as usize;
    [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
}

fn near(a: [u8; 3], b: [u8; 3]) -> bool {
    (0..3).all(|k| (a[k] as i32 - b[k] as i32).abs() <= 3)
}

const EMERALD: &str = "Pokemon - Emerald Version (USA, Europe)";

/// The open cart shows **one** chip, in the middle, carrying the core's name. It does not repeat
/// the game's title in a corner as well: which cart this is was settled on the shelf, and a second
/// thing to read on the board is a second thing to read past.
#[test]
fn the_board_carries_no_second_chip() {
    let face = board_face(&cart(EMERALD, "BPEE"), System::Gba);
    // The board above the socket, where a mask ROM used to sit with the game's name on it: the
    // Advance cart's own outline cuts its top-right corner away, so the region asked for is the
    // board **inside** the socket's silkscreen and above it — substrate, wall shade and nothing
    // else. What would give a second package away is its body, a flat grey-black with no green in
    // it: the substrate is green enough that its red channel alone is under any threshold a black
    // package would pass, so the test asks for grey — red and green within a step of each other.
    let dark = (36..58)
        .flat_map(|y| (150..260).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let [r, g, _] = rgb(&face, x, y);
            r < 60 && (g as i32 - r as i32).abs() < 25
        })
        .count();
    assert_eq!(
        dark, 0,
        "{dark} dark pixels above the socket — a second package is still drawn there"
    );
}

/// Rasterised at the size it is shown at, since that is the only size a face is sharp at.
#[test]
fn the_board_is_the_size_it_is_shown_at() {
    let face = board_face(&cart(EMERALD, "BPEE"), System::Gba);
    assert_eq!((face.w, face.h), (BOARD_W, BOARD_H));
    assert_eq!((BOARD_W, BOARD_H), (372, 209));
}

/// The board fills the shell now, so the shell's plastic does not show anywhere as plastic — but
/// the substrate is pulled a sixteenth of the way to it, so a board out of a green cart is not the
/// same green as one out of a blue one, and a cart can still be told by its own board. Board
/// pixel (126, 37) of a Game Boy cart: on the board, clear of the wall and of everything drawn.
#[test]
fn the_board_takes_the_shells_own_tint() {
    let substrate = |shell: [u8; 3]| -> [u8; 3] {
        [0x1e, 0x53, 0x36]
            .iter()
            .zip(shell)
            .map(|(&x, y)| (x as f32 * 0.84 + y as f32 * 0.16).round() as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    };
    let emerald = board_face(&cart(EMERALD, "BPEE"), System::Gb);
    assert!(
        near(rgb(&emerald, 126, 37), substrate(shell_for("BPEE").colour)),
        "Emerald's board is {:?}",
        rgb(&emerald, 126, 37)
    );
    let unknown = board_face(&cart("Homebrew", ""), System::Gb);
    assert!(
        near(rgb(&unknown, 126, 37), substrate(DEFAULT_SHELL.colour)),
        "an unknown cart's board is {:?}",
        rgb(&unknown, 126, 37)
    );
}

/// A board that failed to draw would come back transparent and every other test here would pass
/// on nothing. Board pixel (250, 180) is bare substrate — right of where the traces stop, above
/// the edge connector, below the socket and clear of the ROM.
#[test]
fn the_board_itself_is_drawn() {
    let face = board_face(&cart(EMERALD, "BPEE"), System::Gba);
    // The substrate, pulled a sixteenth of the way to the cart's own plastic.
    let shell = shell_for("BPEE").colour;
    let base: [u8; 3] = [0x1e, 0x53, 0x36]
        .iter()
        .zip(shell)
        .map(|(&x, y)| (x as f32 * 0.84 + y as f32 * 0.16).round() as u8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    assert!(
        near(rgb(&face, 250, 180), base),
        "the board is {:?}",
        rgb(&face, 250, 180)
    );
}

fn alpha(face: &CartFace, x: u32, y: u32) -> u8 {
    face.rgba[((y * face.w + x) * 4 + 3) as usize]
}

/// The open cart grows out of the cart on the shelf, so its first frame has to be that cart
/// exactly, and its last the place the mockup put it.
#[test]
fn the_open_cart_starts_as_the_shelf_cart_and_lands_where_the_mockup_has_it() {
    assert_eq!(
        shelf_cart(System::Gba),
        Placed {
            x: 240.0,
            y: 172.5,
            w: 240.0,
            h: 135.0
        }
    );
    assert_eq!(board_at(0.0, System::Gba), shelf_cart(System::Gba));
    assert_eq!(
        board_at(1.0, System::Gba),
        Placed {
            x: 174.0,
            y: 150.0,
            w: 372.0,
            h: 209.0
        }
    );
    assert_eq!(board_at(1.7, System::Gba), board_at(1.0, System::Gba), "an overshoot kept growing");
}

#[test]
fn the_lid_leaves_level_and_rests_turned() {
    assert_eq!(lid_at(0.0, System::Gba), (shelf_cart(System::Gba), 0.0));
    let (rest, turn) = lid_at(1.0, System::Gba);
    assert_eq!(
        rest,
        Placed {
            x: 288.0,
            y: 30.0,
            w: 144.0,
            h: 81.0
        }
    );
    assert_eq!(turn, LID_TURN);
    assert!(
        (LID_TURN.to_degrees() + 5.0).abs() < 1e-4,
        "{}",
        LID_TURN.to_degrees()
    );
}

#[test]
fn the_beats_split_one_progress() {
    assert_eq!((slide_of(0.0), lift_of(0.0)), (0.0, 0.0));
    assert_eq!((slide_of(SLIDE_SHARE), lift_of(SLIDE_SHARE)), (1.0, 0.0));
    assert_eq!((slide_of(1.0), lift_of(1.0)), (1.0, 1.0));
    assert!((slide_of(SLIDE_SHARE / 2.0) - 0.5).abs() < 1e-5);
}

/// The back half is the cart that was standing there; it does not move until the front is off it.
#[test]
fn the_back_half_stays_on_the_shelf_while_the_front_slides() {
    for p in [0.0, 0.1, 0.25, SLIDE_SHARE] {
        assert_eq!(board_at(p, System::Gba), shelf_cart(System::Gba), "the back moved at progress {p}");
    }
}

/// A third of the cart, level: the travel that unhooks a real shell once its screw is out.
#[test]
fn the_front_slides_up_a_third_before_it_lifts() {
    assert_eq!(SLIDE_UP, CART_H as f32 / 3.0);
    let shelf = shelf_cart(System::Gba);
    let (slid, turn) = lid_at(SLIDE_SHARE, System::Gba);
    assert_eq!(
        slid,
        Placed {
            y: shelf.y - SLIDE_UP,
            ..shelf
        }
    );
    assert_eq!(turn, 0.0);
    let (half, turn) = lid_at(SLIDE_SHARE / 2.0, System::Gba);
    assert!(half.y < shelf.y && half.y > slid.y && turn == 0.0 && half.w == shelf.w);
}

/// The lift picks the lid up from where the slide left it rather than from the shelf.
#[test]
fn the_lift_starts_where_the_slide_ends() {
    let (slid, _) = lid_at(SLIDE_SHARE, System::Gba);
    let (next, _) = lid_at(SLIDE_SHARE + 0.001, System::Gba);
    assert!((next.y - slid.y).abs() < 0.5 && (next.w - slid.w).abs() < 0.5);
}

#[test]
fn board_units_land_on_the_panel_at_one_and_a_half_times() {
    let board = board_at(1.0, System::Gba);
    assert_eq!(on_board(board, 0.0, 0.0), (174.0, 150.0));
    assert_eq!(on_board(board, 240.0, 135.0), (546.0, 359.0));
}

/// An empty socket names the core it is for, quietly: the chip's own name is the loud one.
#[test]
fn a_socket_names_its_core_at_half_strength() {
    for core in Core::ALL {
        let face = socket_face(core);
        assert_eq!((face.w, face.h), (SOCKET_W, SOCKET_H));
        let strongest = (12..34)
            .flat_map(|y| (16..48).map(move |x| (x, y)))
            .map(|(x, y)| alpha(&face, x, y))
            .max()
            .unwrap();
        assert!(
            (60..=150).contains(&strongest),
            "{core:?}'s name is at alpha {strongest}"
        );
    }
}

/// The outline's left edge is the socket SVG's closing stroke, sitting right at `x = 0` for
/// the straight run of it: a partly covered pixel there is the outline ink, `#eef5e6`, at some
/// fraction of full alpha. Premultiplied, the compositor's own blend then multiplies that
/// fraction in a second time and darkens it; straight alpha keeps the ink's own brightness at
/// any coverage.
#[test]
fn a_partly_covered_socket_edge_pixel_keeps_the_outline_inks_brightness() {
    let face = socket_face(Core::Mgba);
    let (r, a) = (6..(face.h - 6))
        .find_map(|y| {
            let i = ((y * face.w) * 4) as usize;
            let a = face.rgba[i + 3];
            (a > 0 && a < 255).then(|| (face.rgba[i], a))
        })
        .expect("no partly covered pixel on the socket's left edge");
    assert!(
        r > 200,
        "the outline ink darkened at partial coverage: R={r} at alpha={a}"
    );
}

/// A turned quad's edge is not antialiased, so the chip's outline has to sit inside the
/// texture: the outermost `TURN_PAD` pixels are nothing.
#[test]
fn the_chip_is_padded_clear_on_every_side() {
    for core in [Core::Mgba, Core::Gpsp] {
        let face = chip_face(core);
        assert_eq!(
            (face.w, face.h),
            (slot_ui::CORE_W + 2 * TURN_PAD, slot_ui::CORE_H + 2 * TURN_PAD)
        );
        for y in 0..face.h {
            for x in 0..face.w {
                let ring = x < TURN_PAD
                    || y < TURN_PAD
                    || x >= face.w - TURN_PAD
                    || y >= face.h - TURN_PAD;
                if ring {
                    assert_eq!(alpha(&face, x, y), 0, "{core:?} is not clear at {x},{y}");
                }
            }
        }
    }
}

/// The chip is one shape and every core wears it — the board no longer changes which socket it
/// is standing in, it changes the name written on the chip, and a chip whose mark changed with
/// the core would make the name slide out from under a different picture every press.
///
/// What tells two cores apart is the name, which is a face of its own so that it can travel.
#[test]
fn the_chip_is_one_shape_and_each_core_has_a_name_of_its_own() {
    let light = |face: &CartFace| {
        face.rgba
            .chunks_exact(4)
            .filter(|p| p[0] > 200 && p[1] > 200 && p[2] > 200 && p[3] > 200)
            .count()
    };
    assert_eq!(
        chip_face(Core::Mgba).rgba,
        chip_face(Core::Gpsp).rgba,
        "two cores, two chips"
    );
    assert!(
        light(&slot_ui::core_name_face(Core::Mgba)) > 20,
        "mGBA's name is not there"
    );
    assert!(
        light(&slot_ui::core_name_face(Core::Gpsp)) > 20,
        "gpSP's name is not there"
    );
    assert_ne!(
        slot_ui::core_name_face(Core::Mgba).rgba,
        slot_ui::core_name_face(Core::Gpsp).rgba,
        "both cores wear the same name"
    );
}

#[test]
fn padding_moves_nothing() {
    let face = chip_face(Core::Gpsp);
    let more = padded(&face, 3);
    assert_eq!((more.w, more.h), (face.w + 6, face.h + 6));
    for (x, y) in [(0, 0), (10, 7), (face.w - 1, face.h - 1)] {
        let a = ((y * face.w + x) * 4) as usize;
        let b = (((y + 3) * more.w + x + 3) * 4) as usize;
        assert_eq!(face.rgba[a..a + 4], more.rgba[b..b + 4]);
    }
}

#[test]
fn the_shadow_is_darkest_in_the_middle_and_gone_at_the_corners() {
    let s = chip_shadow_face();
    assert_eq!((s.w, s.h), (SHADOW_W, SHADOW_H));
    assert!(alpha(&s, SHADOW_W / 2, SHADOW_H / 2) > alpha(&s, SHADOW_W / 8, SHADOW_H / 2));
    assert_eq!(alpha(&s, 0, 0), 0);
}

/// The seated chip's name sits on the chip's own opaque black body, not on nothing, so its
/// edges have to blend toward whatever it lands on instead of snapping straight to full ink —
/// the name slides, so it is drawn over a chip, over a board and over the edge of the board in
/// turn, and hard-edged type over that reads as a sticker rather than as words.
///
/// It is the name's own face now, not a mark inked into the chip's body: the chip is one shape
/// for every core and the name is what changes.
#[test]
fn the_core_names_edges_are_antialiased() {
    for core in [Core::Mgba, Core::Gpsp] {
        let name = slot_ui::core_name_face(core);
        let soft = (0..name.h)
            .flat_map(|y| (0..name.w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let a = alpha(&name, x, y);
                a > 0 && a < 255
            })
            .count();
        assert!(
            soft > 5,
            "{core:?}'s name has only {soft} soft-edged pixels"
        );
    }
}
