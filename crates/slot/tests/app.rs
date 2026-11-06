mod common;

use std::time::Duration;

use slot::app::{App, Phase, EJECT_S, INSERT_S, SEATED_AT};
use slot::audio::Sfx;
use slot::session::Session;
use slot_input::{Action, Btn, RawEvent};
use slot_store::{write_slot_state, Cart, Core, Platform, SlotState};
use slot_ui::{opening, Draw, TexId, CART_H, CART_W};

fn play(a: &mut App) {
    a.apply(Action::GbaDown(Btn::A));
    a.apply(Action::GbaUp(Btn::A));
}

fn app_with_carts(stems: &[&str]) -> App {
    App::new(
        stems
            .iter()
            .map(|stem| Cart {
                platform: Platform::Gba,
                stem: (*stem).to_string(),
                rom: format!("Games/GBA/{stem}.gba").into(),
                label: None,
                code: String::new(),
                shell: None,
                title: stem.to_uppercase(),
            })
            .collect(),
    )
}

fn is_mouth(d: &Draw) -> bool {
    match *d {
        Draw::Rect { colour, .. } => (0..3).all(|i| (colour[i] - opening()[i]).abs() < 0.001),
        _ => false,
    }
}

fn playing(stem: &str) -> App {
    let mut a = app_with_carts(&[stem, "Zzz"]);
    a.apply(Action::Insert);
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a
}

#[test]
fn insert_waits_for_the_core_even_after_the_animation_floor() {
    let mut a = app_with_carts(&["Emerald"]);
    a.apply(Action::Insert);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(
        matches!(a.phase(), Phase::Inserting { .. }),
        "advanced without the core"
    );
    a.on_core_ready();
    a.update(1.0 / 60.0);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn insert_does_not_advance_before_the_animation_floor_even_if_the_core_is_instant() {
    let mut a = app_with_carts(&["Emerald"]);
    a.apply(Action::Insert);
    a.on_core_ready();
    a.update(1.0 / 60.0);
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
}

#[test]
fn a_disc_spins_and_goes_dark_before_its_core_may_start() {
    let mut a = app_with_platforms(&[(Platform::Dreamcast, "Crazy Taxi 2")]);
    a.apply(Action::Insert);
    assert!(!a.may_spawn_core(), "the core started over the insert");
    for _ in 0..60 {
        a.update(1.0 / 60.0);
    }
    assert!(
        !a.may_spawn_core(),
        "the core may start before the disc has spun"
    );
    while !a.may_spawn_core() {
        a.update(1.0 / 60.0);
    }
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
    a.on_core_ready();
    a.update(1.0 / 60.0);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn a_cart_may_start_its_core_as_it_goes_in() {
    let mut a = app_with_carts(&["Emerald"]);
    a.apply(Action::Insert);
    assert!(a.may_spawn_core());
}

fn seconds_until(a: &mut App, done: fn(&App) -> bool) -> f32 {
    let mut t = 0.0;
    while !done(a) && t < 5.0 {
        a.update(1.0 / 60.0);
        t += 1.0 / 60.0;
    }
    t
}

#[test]
fn the_insert_reads_as_a_push_and_the_eject_takes_the_same_time() {
    let mut a = app_with_carts(&["Emerald", "Zzz"]);
    a.apply(Action::Insert);
    a.on_core_ready();
    let insert = seconds_until(&mut a, |a| matches!(a.phase(), Phase::Playing { .. }));
    a.apply(Action::Eject);
    let eject = seconds_until(&mut a, |a| matches!(a.phase(), Phase::Shelf));
    assert!(insert >= 0.4, "insert is {insert}s, still a wipe");
    assert!(
        eject > EJECT_S,
        "the eject is {eject}s, so the cart moved before the picture was out"
    );
}

#[test]
fn the_game_does_not_appear_the_instant_the_cart_seats() {
    let mut a = app_with_carts(&["Emerald"]);
    a.apply(Action::Insert);
    a.on_core_ready();
    while a.seat() < 1.0 {
        a.update(1.0 / 60.0);
    }
    assert!(
        matches!(a.phase(), Phase::Inserting { .. }),
        "revealed on the same frame it seated"
    );
    let beat = ((INSERT_S - SEATED_AT) * 60.0).ceil() as u32 + 1;
    for _ in 0..beat {
        a.update(1.0 / 60.0);
    }
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn the_game_does_not_draw_during_the_insert() {
    let d = common::tmp_root_with_real_carts(&["Emerald", "Fusion"]);
    common::clocked(d.path());
    let mut s = Session::boot(d.path().to_path_buf());
    s.feed([RawEvent::Down(Btn::A), RawEvent::Up(Btn::A)], 16);
    for i in 0..120 {
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
        if matches!(s.app().phase(), Phase::Inserting { .. }) {
            assert!(
                !s.game_visible(),
                "frame {i}: the game is playing behind the cart"
            );
        }
    }
    assert!(
        s.game_visible(),
        "the core never published, so the insert proved nothing"
    );
}

#[test]
fn the_reveal_waits_for_the_power_on_to_finish() {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let mut a = App::boot(d.path());
    a.apply(Action::Insert);
    a.on_core_ready();
    while a.seat() < 1.0 {
        a.update(1.0 / 60.0);
    }
    assert!(
        a.screen_power() < 1.0,
        "the screen was already on when the cart landed"
    );
    for _ in 0..20 {
        a.update(1.0 / 60.0);
    }
    assert!((a.screen_power() - 1.0).abs() < 0.01);
}

#[test]
fn the_cart_sounds_when_it_reaches_the_slot_and_not_when_it_starts_moving() {
    let mut a = app_with_carts(&["Emerald", "Zzz"]);
    a.apply(Action::Insert);
    a.update(1.0 / 60.0);
    assert_eq!(a.take_sfx(), None, "it sounded before it touched anything");
    let mut heard = None;
    while a.seat() < 1.0 && heard.is_none() {
        a.update(1.0 / 60.0);
        heard = a.take_sfx();
    }
    assert_eq!(heard, Some(Sfx::Insert));
}

#[test]
fn a_resumed_cart_makes_no_sound() {
    let d = common::tmp_root_with_carts(&["Emerald", "Zzz"]);
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Emerald".into()),
            clock_set: true,
            utc_offset_min: 0,
            ..Default::default()
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert_eq!(a.take_sfx(), None);
}

#[test]
fn the_cart_sounds_as_it_comes_free_and_not_before_the_screen_is_out() {
    let mut a = playing("Emerald");
    a.take_sfx();
    a.apply(Action::Eject);
    assert_eq!(a.take_sfx(), None, "it sounded over a live picture");
    let mut heard = None;
    for _ in 0..120 {
        a.update(1.0 / 60.0);
        if let Some(s) = a.take_sfx() {
            heard = Some(s);
            break;
        }
    }
    assert_eq!(heard, Some(Sfx::Eject));
    assert_eq!(a.screen_power(), 0.0, "the picture was still going out");
}

#[test]
fn a_cart_that_fails_to_load_returns_to_the_shelf() {
    let mut a = app_with_carts(&["Broken"]);
    a.apply(Action::Insert);
    a.on_core_failed();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(matches!(a.phase(), Phase::Shelf));
}

#[test]
fn a_refused_cart_pushes_back_out_from_where_it_caught() {
    let mut a = app_with_carts(&["Broken"]);
    a.apply(Action::Insert);
    for _ in 0..6 {
        a.update(1.0 / 60.0);
    }
    let caught = a.seat();
    assert!(
        caught > 0.05 && caught < 0.95,
        "test needs a part seated cart, got {caught}"
    );
    a.on_core_failed();
    assert!(
        (a.seat() - caught).abs() < 1e-3,
        "cart jumped from {caught} to {}",
        a.seat()
    );
}

#[test]
fn an_empty_shelf_has_nothing_to_insert() {
    let mut a = app_with_carts(&[]);
    a.apply(Action::Insert);
    assert!(matches!(a.phase(), Phase::Shelf));
}

#[test]
fn face_buttons_drive_the_shelf_only_while_it_is_showing() {
    let mut a = app_with_carts(&["Emerald", "Wars"]);
    a.apply(Action::GbaDown(Btn::Right));
    play(&mut a);
    let Phase::Inserting { cart, .. } = a.phase() else {
        panic!("A on the shelf did not insert: {:?}", a.phase())
    };
    assert_eq!(cart, "Wars");

    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::GbaDown(Btn::Left));
    a.apply(Action::Eject);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::Insert);
    let Phase::Inserting { cart, .. } = a.phase() else {
        panic!("insert after eject did nothing: {:?}", a.phase())
    };
    assert_eq!(cart, "Wars", "the game's d-pad moved the shelf behind it");
}

#[test]
fn a_held_direction_walks_the_shelf_and_a_release_stops_it() {
    let mut a = app_with_carts(&["A", "B", "C", "D", "E", "F", "G"]);
    a.apply(Action::GbaDown(Btn::Right));
    for _ in 0..30 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::GbaUp(Btn::Right));
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    play(&mut a);
    let Phase::Inserting { cart, .. } = a.phase() else {
        panic!("A on the shelf did not insert: {:?}", a.phase())
    };
    assert_eq!(cart, "C", "one press and one repeat, then nothing");
}

fn app_with_platforms(carts: &[(Platform, &str)]) -> App {
    App::new(
        carts
            .iter()
            .map(|(platform, stem)| Cart {
                platform: *platform,
                stem: (*stem).to_string(),
                rom: format!(
                    "Games/{}/{stem}.{}",
                    platform.dir_name(),
                    platform.extensions()[0]
                )
                .into(),
                label: None,
                code: String::new(),
                shell: None,
                title: stem.to_uppercase(),
            })
            .collect(),
    )
}

fn tap(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

#[test]
fn the_shoulders_ring_over_the_shelves() {
    let mut a = app_with_platforms(&[(Platform::Gba, "Emerald"), (Platform::Gb, "Tetris")]);
    assert_eq!(a.selected_stem(), Some("Emerald"));
    tap(&mut a, Btn::R1);
    assert_eq!(
        a.selected_stem(),
        Some("Tetris"),
        "R1 did not reach the Game Boy shelf"
    );
    tap(&mut a, Btn::R1);
    assert_eq!(
        a.selected_stem(),
        Some("Emerald"),
        "the ring did not come back round"
    );
    tap(&mut a, Btn::L1);
    assert_eq!(
        a.selected_stem(),
        Some("Tetris"),
        "L1 did not reach the Game Boy shelf"
    );
}

#[test]
fn a_colour_cart_stands_on_a_shelf_of_its_own() {
    let mut a = app_with_platforms(&[
        (Platform::Gba, "Emerald"),
        (Platform::Gb, "Tetris"),
        (Platform::Gbc, "Chromatic"),
    ]);
    tap(&mut a, Btn::R1);
    assert_eq!(a.selected_stem(), Some("Tetris"));
    tap(&mut a, Btn::R1);
    assert_eq!(
        a.selected_stem(),
        Some("Chromatic"),
        "the Colour cart shares the Game Boy shelf"
    );
    tap(&mut a, Btn::R1);
    assert_eq!(
        a.selected_stem(),
        Some("Emerald"),
        "three shelves did not ring back round to the first"
    );
    tap(&mut a, Btn::L1);
    assert_eq!(a.selected_stem(), Some("Chromatic"));
}

#[test]
fn every_shelf_keeps_the_cart_it_was_left_on() {
    let mut a = app_with_platforms(&[
        (Platform::Gba, "Emerald"),
        (Platform::Gba, "Fusion"),
        (Platform::Gb, "Tetris"),
        (Platform::Gb, "Zelda"),
    ]);
    tap(&mut a, Btn::Right);
    assert_eq!(a.selected_stem(), Some("Fusion"));
    tap(&mut a, Btn::R1);
    assert_eq!(a.selected_stem(), Some("Tetris"));
    tap(&mut a, Btn::Right);
    assert_eq!(a.selected_stem(), Some("Zelda"));

    tap(&mut a, Btn::L1);
    assert_eq!(
        a.selected_stem(),
        Some("Fusion"),
        "the Game Boy Advance shelf forgot where it was"
    );
    tap(&mut a, Btn::R1);
    assert_eq!(
        a.selected_stem(),
        Some("Zelda"),
        "the Game Boy shelf forgot where it was"
    );
}

#[test]
fn a_shelf_comes_back_settled_where_it_was_left() {
    let mut a = app_with_platforms(&[
        (Platform::Gba, "Aaa"),
        (Platform::Gba, "Bbb"),
        (Platform::Gba, "Ccc"),
        (Platform::Gb, "Tetris"),
    ]);
    let faces: Vec<TexId> = (0..4).map(|i| TexId::from_raw(700 + i)).collect();
    a.set_faces(faces.clone());
    tap(&mut a, Btn::Right);
    tap(&mut a, Btn::Right);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    let settled = tex_at(&frame(&a), faces[2])
        .expect("no face for the selected cart")
        .1;

    tap(&mut a, Btn::R1);
    assert!(
        tex_at(&frame(&a), faces[3]).is_some(),
        "the Game Boy cart has no face of its own"
    );
    tap(&mut a, Btn::L1);
    a.update(1.0 / 60.0);
    let back = tex_at(&frame(&a), faces[2])
        .expect("no face on the way back")
        .1;
    assert!(
        (back[0] - settled[0]).abs() < 1.0,
        "the row came back at {} rather than settled at {}",
        back[0],
        settled[0]
    );
}

#[test]
fn the_shoulders_do_nothing_when_there_is_one_shelf_to_be_on() {
    let mut a = app_with_platforms(&[(Platform::Gba, "Emerald"), (Platform::Gba, "Fusion")]);
    for btn in [Btn::L1, Btn::R1] {
        tap(&mut a, btn);
        assert_eq!(
            a.selected_stem(),
            Some("Emerald"),
            "{btn:?} moved a shelf with no neighbour"
        );
        assert_eq!(
            a.toast(),
            None,
            "{btn:?} said something about doing nothing"
        );
        assert_eq!(
            a.shelf_shake(),
            0.0,
            "{btn:?} refused where it should have done nothing"
        );
    }
}

#[test]
fn the_switch_changes_the_name_on_the_band_and_says_nothing() {
    let mut a = app_with_platforms(&[
        (Platform::Gba, "Emerald"),
        (Platform::Gb, "Tetris"),
        (Platform::Gbc, "Chromatic"),
    ]);
    assert_eq!(
        a.shelf_platform_name(),
        Some("Game Boy Advance"),
        "the card did not open on the GBA"
    );
    for (btn, want) in [
        (Btn::R1, "Game Boy"),
        (Btn::R1, "Game Boy Color"),
        (Btn::R1, "Game Boy Advance"),
        (Btn::L1, "Game Boy Color"),
    ] {
        a.apply_at(Action::GbaDown(btn), 1_000);
        assert_eq!(
            a.shelf_platform_name(),
            Some(want),
            "{btn:?} left the wrong name on the case"
        );
        assert_eq!(a.toast(), None, "{btn:?} put a banner up over the carts");
    }
    a.update(5.0);
    assert_eq!(
        a.shelf_platform_name(),
        Some("Game Boy Color"),
        "the name faded"
    );
}

#[test]
fn a_card_on_one_shelf_names_nothing_because_there_is_nowhere_to_switch_to() {
    let mut alone = app_with_platforms(&[(Platform::Gba, "Emerald"), (Platform::Gba, "Fusion")]);
    assert_eq!(
        alone.shelf_platform_name(),
        None,
        "a card with only Game Boy Advance carts named its platform anyway"
    );
    for btn in [Btn::L1, Btn::R1] {
        alone.apply_at(Action::GbaDown(btn), 1_000);
        assert_eq!(
            alone.shelf_platform_name(),
            None,
            "{btn:?} named a shelf on a card that has one"
        );
    }

    let two = app_with_platforms(&[(Platform::Gba, "Emerald"), (Platform::Gb, "Tetris")]);
    assert_eq!(
        two.shelf_platform_name(),
        Some("Game Boy Advance"),
        "a card with two shelves did not say which one it was on"
    );
}

#[test]
fn the_carousel_opens_on_a_shelf_that_has_carts() {
    let a = app_with_platforms(&[(Platform::Gbc, "Chromatic")]);
    assert_eq!(a.selected_stem(), Some("Chromatic"));
}

#[test]
fn the_shoulders_belong_to_the_game_while_one_is_playing() {
    let mut a = app_with_platforms(&[
        (Platform::Gba, "Emerald"),
        (Platform::Gba, "Fusion"),
        (Platform::Gb, "Tetris"),
    ]);
    a.apply(Action::Insert);
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    tap(&mut a, Btn::R1);
    assert_eq!(a.toast(), None, "the game's shoulder button named a shelf");
    assert_eq!(
        a.selected_stem(),
        Some("Emerald"),
        "the game's shoulder button switched the shelf behind it"
    );
}

#[test]
fn a_held_direction_does_not_follow_the_shelf_it_was_pressed_on() {
    let mut a = app_with_platforms(&[
        (Platform::Gba, "Aaa"),
        (Platform::Gba, "Bbb"),
        (Platform::Gba, "Ccc"),
        (Platform::Gb, "Tetris"),
    ]);
    a.apply_at(Action::GbaDown(Btn::Right), 0);
    assert_eq!(a.selected_stem(), Some("Bbb"));
    a.apply_at(Action::GbaDown(Btn::R1), 10);
    a.apply_at(Action::GbaDown(Btn::L1), 20);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert_eq!(
        a.selected_stem(),
        Some("Bbb"),
        "the row walked on by itself once the shelf came back"
    );
}

#[test]
fn a_cart_in_flight_is_not_also_left_standing_on_the_shelf() {
    let mut a = app_with_carts(&["Emerald"]);
    a.apply(Action::Insert);
    a.update(0.2);
    let mut out = Vec::new();
    a.draw(&mut out);
    let carts = out
        .iter()
        .filter(|d| match **d {
            Draw::Rect { w, h, .. }
            | Draw::Tex { w, h, .. }
            | Draw::Part { w, h, .. }
            | Draw::Turned { w, h, .. } => {
                w > 200.0
                    && w <= CART_W as f32 + 0.01
                    && (w / h - CART_W as f32 / CART_H as f32).abs() < 0.05
            }
            Draw::Game | Draw::Shot { .. } => false,
        })
        .count();
    assert_eq!(
        carts, 1,
        "the shelf still holds the cart the slot is taking"
    );
}

#[test]
fn the_shelf_shows_the_empty_slot() {
    let a = app_with_carts(&["Emerald", "Zzz"]);
    let mut out = Vec::new();
    a.draw(&mut out);
    assert!(
        out.iter().any(is_mouth),
        "the shelf has no slot, so the cart has nowhere visible to go"
    );
}

#[test]
fn inserting_still_has_a_mouth_to_go_into() {
    let mut a = app_with_carts(&["Emerald"]);
    a.apply(Action::Insert);
    a.update(1.0 / 60.0);
    let mut out = Vec::new();
    a.draw(&mut out);
    assert!(
        out.iter().any(is_mouth),
        "the cart has nothing to slide into"
    );
}

#[test]
fn eject_returns_to_the_shelf_only_once_the_cart_is_out() {
    let mut a = playing("Emerald");
    a.apply(Action::Eject);
    dark(&mut a);
    while a.seat() > 0.0 {
        assert!(
            matches!(a.phase(), Phase::Ejecting { .. }),
            "the shelf came back with the cart {} of the way in",
            a.seat()
        );
        a.update(1.0 / 60.0);
    }
    a.update(1.0 / 60.0);
    assert!(matches!(a.phase(), Phase::Shelf));
}

fn dark(a: &mut App) {
    for _ in 0..300 {
        if a.screen_power() == 0.0 {
            return;
        }
        a.update(1.0 / 60.0);
    }
    panic!("the screen never went dark");
}

fn lists_game(a: &App) -> bool {
    let mut out = Vec::new();
    a.draw(&mut out);
    out.iter().any(|d| matches!(d, Draw::Game))
}

#[test]
fn the_game_layer_is_listed_only_once_the_screen_is_up() {
    let mut a = app_with_carts(&["Emerald", "Zzz"]);
    a.set_game_ready(true);
    a.apply(Action::Insert);
    a.on_core_ready();
    while a.seat() < 1.0 {
        a.update(1.0 / 60.0);
        assert!(
            !lists_game(&a),
            "the picture is drawn while the cart is still going in"
        );
    }
    for _ in 0..30 {
        a.update(1.0 / 60.0);
    }
    assert!(lists_game(&a), "the game never reached the draw list");
}

#[test]
fn the_cart_waits_for_the_screen_to_go_dark() {
    let d = common::tmp_root_with_carts(&["Emerald", "Zzz"]);
    let mut a = common::app_playing_in(d.path(), "Emerald");
    a.apply(Action::Eject);
    while a.screen_power() > 0.0 {
        assert_eq!(
            a.seat(),
            1.0,
            "the cart started leaving while the screen was still lit"
        );
        a.update(1.0 / 60.0);
        assert!(a.now() < 5_000, "the screen never went dark");
    }
    for _ in 0..40 {
        a.update(1.0 / 60.0);
    }
    assert!(
        a.seat() < 1.0,
        "the cart never left once the screen was dark"
    );
}

#[test]
fn eject_is_the_insert_backwards() {
    let d = common::tmp_root_with_carts(&["Emerald", "Zzz"]);
    let mut a = common::app_playing_in(d.path(), "Emerald");
    a.apply(Action::Eject);
    let first = a.screen_power();
    a.update(1.0 / 60.0);
    assert!(a.screen_power() < first, "the screen is not closing");
}

#[test]
fn the_quick_menu_opens_from_the_shelf_and_nowhere_else() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    let (mut s, _motor) = common::session_with_platform(d.path());

    assert!(
        matches!(s.app().phase(), slot::app::Phase::Shelf),
        "not on the shelf: {:?}",
        s.app().phase()
    );
    s.app_mut().apply(slot_input::Action::QuickMenu);
    assert!(matches!(
        s.app().phase(),
        slot::app::Phase::QuickMenu { .. }
    ));

    s.app_mut()
        .apply(slot_input::Action::GbaDown(slot_input::Btn::B));
    s.app_mut().apply(slot_input::Action::Insert);
    s.app_mut().apply(slot_input::Action::QuickMenu);
    assert!(
        !matches!(s.app().phase(), slot::app::Phase::QuickMenu { .. }),
        "a menu opened over a seated cart"
    );
}

#[test]
fn both_b_and_menu_take_the_about_screen_back_to_the_quick_menu() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    for out in [
        slot_input::Action::GbaDown(slot_input::Btn::B),
        slot_input::Action::QuickMenu,
    ] {
        let (mut s, _motor) = common::session_with_platform(d.path());
        s.app_mut().apply(slot_input::Action::QuickMenu);
        for _ in 0..slot_ui::QuickRow::About.position(&[]) {
            s.app_mut()
                .apply(slot_input::Action::GbaDown(slot_input::Btn::Down));
        }
        s.app_mut()
            .apply(slot_input::Action::GbaDown(slot_input::Btn::A));
        assert!(matches!(s.app().phase(), slot::app::Phase::About));
        s.app_mut().apply(out);
        assert_eq!(
            s.app().quick_menu(),
            Some(slot_ui::QuickRow::About),
            "{out:?} did not take the label back to the menu"
        );
    }
}

fn on_shelf(stems: &[&str]) -> (tempfile::TempDir, App) {
    booted_on_shelf(common::tmp_root_with_carts(stems))
}

fn booted_on_shelf(d: tempfile::TempDir) -> (tempfile::TempDir, App) {
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let app = App::boot(d.path());
    (d, app)
}

fn frame(app: &App) -> Vec<Draw> {
    let mut out = Vec::new();
    app.draw(&mut out);
    out
}

fn tex_at(out: &[Draw], want: TexId) -> Option<(usize, [f32; 4])> {
    out.iter().enumerate().find_map(|(i, d)| match *d {
        Draw::Tex {
            x, y, w, h, tex, ..
        } if tex == want => Some((i, [x, y, w, h])),
        _ => None,
    })
}

/// Where the core list's cursor is, when the list is open. START opens one list, whichever machine
/// the cart is for, and on the core that cart already runs.
fn core_list(app: &App) -> Option<usize> {
    match app.phase() {
        Phase::Cores { cursor } => Some(*cursor),
        _ => None,
    }
}

/// A WonderSwan has one core, so there is nothing to choose between.
#[test]
fn start_opens_no_core_list_on_a_system_with_one_core() {
    let (_d, mut ws) = booted_on_shelf(common::tmp_root_with_ws_carts(&["Gunpey", "Zzz"]));
    ws.apply(Action::GbaDown(Btn::Start));
    assert_eq!(core_list(&ws), None, "a WonderSwan cart opened a core list");
    assert!(matches!(ws.phase(), Phase::Shelf), "{:?}", ws.phase());
}

#[test]
fn start_on_the_shelf_opens_the_core_list_on_the_carts_current_core() {
    let (d, mut app) = on_shelf(&["Emerald", "Zzz"]);
    assert_eq!(app.selected_stem(), Some("Emerald"));

    app.apply(Action::GbaDown(Btn::Start));
    assert_eq!(
        core_list(&app),
        Some(0),
        "a cart with no line of its own runs mGBA"
    );
    app.apply(Action::GbaDown(Btn::B));

    slot_store::write_selected_core(d.path(), "Emerald", Core::Gpsp).unwrap();
    app.apply(Action::GbaDown(Btn::Start));
    assert_eq!(
        core_list(&app),
        Some(1),
        "the list opens on the core the cart already uses"
    );
}

/// Down and Up walk the list and come round at either end, as a list does.
#[test]
fn the_list_walks_its_cores_and_comes_round() {
    let (_d, mut app) = on_shelf(&["Emerald", "Zzz"]);
    app.apply(Action::GbaDown(Btn::Start));
    for want in [1, 2, 3, 0] {
        app.apply(Action::GbaDown(Btn::Down));
        assert_eq!(core_list(&app), Some(want), "after Down");
    }
    app.apply(Action::GbaDown(Btn::Up));
    assert_eq!(core_list(&app), Some(3), "Up from the first comes round");
}

#[test]
fn choosing_a_core_writes_it_and_closes() {
    let (d, mut app) = on_shelf(&["Emerald", "Zzz"]);
    app.apply(Action::GbaDown(Btn::Start));
    app.apply(Action::GbaDown(Btn::Down));
    app.apply(Action::GbaDown(Btn::A));

    assert_eq!(slot_store::core_for(d.path(), "Emerald"), Core::Gpsp);
    assert_eq!(
        slot_store::core_for(d.path(), "Zzz"),
        Core::Mgba,
        "the choice landed on a cart the shelf was not on"
    );
    assert_eq!(core_list(&app), None, "the list stayed open after a choice");
}

#[test]
fn y_in_the_list_sets_the_core_for_every_game_of_the_system() {
    let (d, mut app) = on_shelf(&["Emerald", "Zzz"]);
    app.apply(Action::GbaDown(Btn::Start));
    app.apply(Action::GbaDown(Btn::Down));
    app.apply(Action::GbaDown(Btn::Y));

    assert_eq!(core_list(&app), None);
    assert_eq!(
        slot_store::core_for_system(d.path(), Platform::Gba),
        Core::Gpsp
    );
    for stem in ["Emerald", "Zzz"] {
        assert_eq!(
            slot_store::core_for_platform(d.path(), stem, Platform::Gba),
            Core::Gpsp,
            "{stem} did not follow the system"
        );
    }
}

#[test]
fn b_closes_the_list_without_writing() {
    let (d, mut app) = on_shelf(&["Emerald", "Zzz"]);
    app.apply(Action::GbaDown(Btn::Start));
    app.apply(Action::GbaDown(Btn::Down));
    app.apply(Action::GbaDown(Btn::B));

    assert_eq!(core_list(&app), None);
    assert_eq!(
        slot_store::core_for(d.path(), "Emerald"),
        Core::Mgba,
        "backing out of the list still changed the cart"
    );
}

#[test]
fn the_list_does_not_open_on_an_empty_shelf() {
    let (_d, mut app) = on_shelf(&[]);
    app.apply(Action::GbaDown(Btn::Start));
    assert_eq!(core_list(&app), None);
}

#[test]
fn select_on_the_shelf_leaves_the_core_list_shut_so_it_can_still_chord() {
    let (_d, mut app) = on_shelf(&["Emerald", "Zzz"]);
    app.apply(Action::GbaDown(Btn::Select));
    assert_eq!(
        core_list(&app),
        None,
        "SELECT must stay free for brightness and blue light on the shelf"
    );
}
