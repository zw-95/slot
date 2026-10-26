mod common;

use common::{tmp_root_with_carts, write_gb_cart};
use slot::app::App;
use slot_input::{Action, Btn};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Writes `stem` into a subfolder of the system's own folder, as a card sorted by hand would.
fn rom_in_folder(d: &TempDir, folder: &str, stem: &str) -> PathBuf {
    let path = d
        .path()
        .join("Games/GBA")
        .join(folder)
        .join(format!("{stem}.gba"));
    std::fs::create_dir_all(path.parent().expect("a rom has a folder")).expect("create dir");
    std::fs::write(&path, vec![0u8; 0x100]).expect("write rom");
    path
}

fn card() -> TempDir {
    let d = tmp_root_with_carts(&["Fusion"]);
    write_gb_cart(&d, "Tetris", "TETRIS");
    rom_in_folder(&d, "动作", "Zelda");
    rom_in_folder(&d, "汉化", "Emerald");
    d
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn on_shelf(root: &Path) -> App {
    App::boot(root)
}

#[test]
fn x_walks_the_systems_subfolders_and_back_to_everything() {
    let d = card();
    let mut a = on_shelf(d.path());

    // Everything first: the shelf names the system, not a folder.
    assert_eq!(a.slot_text().as_deref(), Some("GBA"));
    assert_eq!(a.selected_stem(), Some("Emerald"));

    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-动作"));
    assert_eq!(
        a.selected_stem(),
        Some("Zelda"),
        "the folder showed the wrong game"
    );

    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-汉化"));
    assert_eq!(a.selected_stem(), Some("Emerald"));

    // Past the last folder, back to the whole system.
    press(&mut a, Btn::X);
    assert_eq!(
        a.slot_text().as_deref(),
        Some("GBA"),
        "the last folder did not cycle back to everything"
    );
}

#[test]
fn a_short_x_does_not_open_the_keyboard() {
    let d = card();
    let mut a = on_shelf(d.path());
    press(&mut a, Btn::X);
    assert!(
        !matches!(a.phase(), slot::app::Phase::Keyboard { .. }),
        "切文件夹不该顺手把搜索叫出来"
    );
}

#[test]
fn a_held_x_opens_the_keyboard_before_it_is_let_go() {
    let d = card();
    let mut a = on_shelf(d.path());
    a.apply_at(Action::GbaDown(Btn::X), 1_000);
    // 按住不动：走够 500ms 就该开搜索，不必等松手。
    a.update(0.6);
    assert!(
        matches!(a.phase(), slot::app::Phase::Keyboard { .. }),
        "长按 X 到点就该打开搜索键盘，别等到松手"
    );
    // 松手也不该再顺手切一次文件夹。
    a.apply_at(Action::GbaUp(Btn::X), 1_700);
    assert!(
        matches!(a.phase(), slot::app::Phase::Keyboard { .. }),
        "长按之后松手，不该又去切文件夹"
    );
}

#[test]
fn x_stays_inside_the_system_the_player_is_on() {
    let d = card();
    let mut a = on_shelf(d.path());
    // 循环到最后一个文件夹再按一次，回到"全部"；再按也只是重头开始，不会跑到别的机种。
    for want in ["GBA-动作", "GBA-汉化", "GBA", "GBA-动作"] {
        press(&mut a, Btn::X);
        assert_eq!(
            a.slot_text().as_deref(),
            Some(want),
            "X 走出了当前机种，或者顺序不对"
        );
        assert_eq!(
            a.shelf_platform_name(),
            Some("Game Boy Advance"),
            "X 把机种也换了"
        );
    }
}

#[test]
fn the_count_is_the_place_in_the_row_out_of_the_games_on_the_shelf() {
    let d = card();
    let mut a = on_shelf(d.path());
    // Everything is on the shelf: three GBA games, and the hand is on the first of them.
    assert_eq!(a.count_text().as_deref(), Some("1 / 3"));
    press(&mut a, Btn::Right);
    assert_eq!(a.count_text().as_deref(), Some("2 / 3"));

    // A subfolder is a shorter list, and the count is of that list.
    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-动作"));
    assert_eq!(
        a.count_text().as_deref(),
        Some("1 / 1"),
        "the folder's one game is the whole shelf now"
    );
}

#[test]
fn a_folder_only_shows_the_games_that_are_in_it() {
    let d = card();
    let mut a = on_shelf(d.path());
    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-动作"));

    // One game in this folder, so walking the shelf cannot reach another.
    for _ in 0..3 {
        press(&mut a, Btn::Right);
        assert_eq!(a.selected_stem(), Some("Zelda"));
    }
}

#[test]
fn leaving_a_shelf_puts_the_whole_system_back_on_it() {
    let d = card();
    let mut a = on_shelf(d.path());
    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-动作"));

    press(&mut a, Btn::R1);
    assert_eq!(a.shelf_platform_name(), Some("Game Boy"));
    press(&mut a, Btn::R1);

    assert_eq!(
        a.slot_text().as_deref(),
        Some("GBA"),
        "coming back left the folder filter on the shelf"
    );
    assert_eq!(a.selected_stem(), Some("Emerald"));
}

/// A card filed the way a big library gets filed: the system's own folder holds nothing but
/// subfolders. This is the one that used to stick.
fn card_with_every_game_in_a_folder() -> TempDir {
    let d = tmp_root_with_carts(&[]);
    write_gb_cart(&d, "Tetris", "TETRIS");
    rom_in_folder(&d, "动作", "Zelda");
    rom_in_folder(&d, "汉化", "Emerald");
    d
}

#[test]
fn x_cycles_back_to_every_game_when_none_sit_in_the_systems_own_folder() {
    let d = card_with_every_game_in_a_folder();
    let mut a = on_shelf(d.path());
    // Everything: both folders' carts are on the shelf together.
    assert_eq!(a.count_text().as_deref(), Some("1 / 2"));

    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-动作"));
    press(&mut a, Btn::X);
    assert_eq!(a.slot_text().as_deref(), Some("GBA-汉化"));

    // Past the last folder, back to everything — which is not "the games sitting in the
    // system's own folder", because there are none. That was the sticking point.
    press(&mut a, Btn::X);
    assert_eq!(
        a.slot_text().as_deref(),
        Some("GBA"),
        "走完最后一个文件夹要回到全部，哪怕机种目录里一个游戏都没有"
    );
    assert_eq!(a.count_text().as_deref(), Some("1 / 2"));
}

#[test]
fn a_system_with_no_subfolders_ignores_the_key() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = on_shelf(d.path());
    press(&mut a, Btn::X);
    assert_eq!(
        a.slot_text(),
        None,
        "a card with one system names no platform, and has no folder to name either"
    );
    assert_eq!(a.selected_stem(), Some("Emerald"));
}
