use slot_store::{initial, sort_key};

#[test]
fn titles_file_digits_first_then_a_to_z_whatever_their_case() {
    let mut names = vec![
        "Zebra",
        "apple",
        "3D Pinball",
        "Metroid",
        "1943",
        "banana",
        "Apotris",
    ];
    names.sort_by_key(|n| sort_key(n));
    assert_eq!(
        names,
        vec![
            "1943",
            "3D Pinball",
            "Apotris",
            "apple",
            "banana",
            "Metroid",
            "Zebra"
        ]
    );
}

#[test]
fn a_title_led_by_punctuation_files_last() {
    let mut names = vec!["[BIOS] Test", "Apotris", "1943"];
    names.sort_by_key(|n| sort_key(n));
    assert_eq!(names, vec!["1943", "Apotris", "[BIOS] Test"]);
}

#[test]
fn digits_and_punctuation_share_one_stop() {
    assert_eq!(initial("1943"), '#');
    assert_eq!(initial("3D Pinball"), '#');
    assert_eq!(initial("[BIOS] Test"), '#');
    assert_eq!(initial("apple"), 'A');
    assert_eq!(initial("Apotris"), 'A');
    assert_eq!(initial("  Metroid"), 'M', "leading space hid the letter");
}

#[test]
fn a_chinese_name_files_under_the_letter_its_reading_starts_with() {
    assert_eq!(initial("腾讯翻译"), 'T', "it filed under 腾");
    assert_eq!(initial("洛克人Zero"), 'L');
    assert_eq!(initial("宝可梦-红宝石"), 'B');
    assert_eq!(initial(" 恶魔城"), 'E', "leading space hid the hanzi");
    assert_eq!(initial("1943"), '#');
}

#[test]
fn a_chinese_name_sorts_where_it_sounds() {
    let mut names = vec!["Zelda", "腾讯翻译", "Apotris", "洛克人", "Metroid"];
    names.sort_by_key(|n| sort_key(n));
    assert_eq!(
        names,
        vec!["Apotris", "洛克人", "Metroid", "腾讯翻译", "Zelda"],
        "the Chinese names did not land among the letters they sound like"
    );
}

use slot_store::{Cart, Platform};
use slot_ui::Shelf;

fn shelf_of(names: &[&str]) -> Shelf {
    Shelf::new(
        names
            .iter()
            .map(|n| Cart {
                platform: Platform::Gba,
                stem: (*n).to_string(),
                rom: format!("Games/GBA/{n}.gba").into(),
                label: None,
                code: String::new(),
                shell: None,
                title: n.to_uppercase(),
            })
            .collect(),
    )
}

const ROW: [&str; 7] = [
    "1943",
    "Apotris",
    "Advance Wars",
    "Metroid",
    "Mario Kart",
    "Zelda",
    "Zzz",
];

#[test]
fn down_crosses_to_the_next_letter() {
    let mut s = shelf_of(&ROW);
    assert_eq!(s.index, 0);
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "Apotris");
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "Metroid");
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "Zelda");
}

#[test]
fn up_lands_on_the_start_of_the_letter_before_leaving_it() {
    let mut s = shelf_of(&ROW);
    s.select(4);
    s.jump_prev_letter();
    assert_eq!(s.carts[s.index].stem, "Metroid", "it left the Ms too early");
    s.jump_prev_letter();
    assert_eq!(s.carts[s.index].stem, "Apotris");
}

#[test]
fn the_letters_wrap_at_both_ends() {
    let mut s = shelf_of(&ROW);
    s.select(5);
    s.jump_next_letter();
    assert_eq!(
        s.carts[s.index].stem, "1943",
        "the end did not loop forward"
    );

    s.select(0);
    s.jump_prev_letter();
    assert_eq!(
        s.carts[s.index].stem, "Zelda",
        "the start did not loop back"
    );
}

#[test]
fn a_wrap_travels_the_way_the_press_asked() {
    let mut s = shelf_of(&ROW);
    s.select(5);
    let before = s.scroll_target();
    s.jump_next_letter();
    assert!(
        s.scroll_target() > before,
        "Down at the end turned round instead of carrying on forwards"
    );

    s.select(0);
    let before = s.scroll_target();
    s.jump_prev_letter();
    assert!(
        s.scroll_target() < before,
        "Up at the start turned round instead of carrying on backwards"
    );
}

fn shelf_sorted(names: &[&str]) -> Shelf {
    let mut names = names.to_vec();
    names.sort_by_key(|n| sort_key(n));
    shelf_of(&names)
}

#[test]
fn up_and_down_step_through_the_letters_a_chinese_name_sounds_like() {
    let mut s = shelf_sorted(&["Apotris", "洛克人", "Metroid", "腾讯翻译", "Zelda"]);
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "洛克人", "L did not come after A");
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "Metroid");
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "腾讯翻译", "T did not come after M");
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].stem, "Zelda");
    s.jump_prev_letter();
    assert_eq!(s.carts[s.index].stem, "腾讯翻译", "back from Z is T, not 腾");
}

#[test]
fn a_row_of_one_letter_stays_put() {
    let mut s = shelf_of(&["Metroid", "Mario Kart"]);
    s.jump_next_letter();
    assert_eq!(
        s.carts[s.index].stem, "Metroid",
        "it moved within one letter"
    );
    s.jump_prev_letter();
    assert_eq!(s.carts[s.index].stem, "Metroid");
}

/// An arcade set the way a scan leaves it: the zip's name as its stem, the game's name in
/// Chinese read out of the database at scan time, and the shelf sorted by that name.
fn arcade_shelf_of(sets: &[(&str, &str)]) -> Shelf {
    let mut carts: Vec<Cart> = sets
        .iter()
        .map(|(set, zh)| Cart {
            platform: Platform::Arcade,
            stem: (*set).to_string(),
            rom: format!("Games/ARCADE/{set}.zip").into(),
            label: None,
            code: String::new(),
            shell: None,
            title: (*zh).to_string(),
        })
        .collect();
    carts.sort_by_key(|c| sort_key(c.shelf_name()));
    Shelf::new(carts)
}

#[test]
fn up_and_down_step_through_the_letters_an_arcade_sets_chinese_name_sounds_like() {
    // Sorted by their names in Chinese these are #, 吃豆人 (C), 合金弹头 (H). By their zip
    // names the last two are both M — one letter — so a jump that reads the zip skips H
    // altogether and lands back on the digit.
    let mut s = arcade_shelf_of(&[
        ("1941", "1941 海战"),
        ("mslug", "合金弹头"),
        ("mspacman", "吃豆人"),
    ]);
    s.jump_next_letter();
    assert_eq!(s.carts[s.index].shelf_name(), "吃豆人");
    s.jump_next_letter();
    assert_eq!(
        s.carts[s.index].shelf_name(),
        "合金弹头",
        "it filed the sets under their zips' names, where these two are both M"
    );
    s.jump_prev_letter();
    assert_eq!(s.carts[s.index].shelf_name(), "吃豆人");
}

#[test]
fn an_arcade_set_is_labelled_with_the_game_not_its_zip() {
    // What search reads, and so what it can find: the game's name in Chinese, not "mslug".
    let s = arcade_shelf_of(&[("mslug", "合金弹头")]);
    assert_eq!(slot_ui::label_text(&s.carts[0]), "合金弹头");
}
