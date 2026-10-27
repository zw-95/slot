mod common;

use common::tmp_root;
use slot_store::{folder_of, scan, Platform};
use tempfile::TempDir;

fn write_rom(d: &TempDir, rel: &str, title: &str) {
    let mut rom = vec![0u8; 0x100];
    rom[0xa0..0xa0 + title.len()].copy_from_slice(title.as_bytes());
    std::fs::write(d.path().join("Games").join(rel), rom).expect("write rom");
}

fn write_png(d: &TempDir, rel: &str) {
    let path = d.path().join("Labels").join(rel);
    std::fs::create_dir_all(path.parent().expect("label has a parent")).expect("create dir");
    std::fs::write(path, b"\x89PNG\r\n\x1a\n").expect("write png");
}

#[test]
fn a_png_in_labels_is_paired_to_its_rom_by_stem() {
    let d = tmp_root();
    write_rom(&d, "GBA/Pokemon Emerald.gba", "POKEMON EMER");
    write_png(&d, "GBA/Pokemon Emerald.png");
    write_rom(&d, "GBA/Advance Wars.gba", "ADVANCEWARS");
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 2);
    assert_eq!(carts[0].stem, "Advance Wars");
    assert!(carts[0].label.is_none());
    assert!(
        carts[1].label.is_some(),
        "a label in Labels/ was not picked up"
    );
    let mut emerald = carts[1].clone();
    emerald.read_header();
    assert_eq!(emerald.title, "POKEMON EMER");
}

#[test]
fn a_label_pairs_with_its_rom_whatever_the_case_of_its_name() {
    let d = tmp_root();
    write_rom(&d, "GBA/Pokemon Emerald.gba", "POKEMON EMER");
    write_png(&d, "GBA/pokemon emerald.PNG");
    let carts = scan(d.path()).unwrap();
    assert_eq!(
        carts[0].label.as_deref(),
        Some(d.path().join("Labels/GBA/pokemon emerald.PNG").as_path())
    );
}

#[test]
fn a_scan_reads_names_and_leaves_the_rom_headers_for_later() {
    let d = tmp_root();
    write_rom(&d, "GBA/Advance Wars.gba", "ADVANCEWARS");
    let mut cart = scan(d.path()).unwrap().remove(0);
    assert!(cart.title.is_empty());
    cart.read_header();
    assert_eq!(cart.title, "ADVANCEWARS");
}

#[test]
fn scan_ignores_non_gba_files() {
    let d = tmp_root();
    write_rom(&d, "GBA/Real.gba", "REAL");
    std::fs::write(d.path().join("Games/GBA/notes.txt"), "hi").unwrap();
    assert_eq!(scan(d.path()).unwrap().len(), 1);
}

#[test]
fn an_appledouble_sidecar_is_not_shelved_as_a_cart() {
    let d = tmp_root();
    write_rom(&d, "GBA/Metroid Fusion.gba", "METROID");
    write_rom(&d, "GBA/._Metroid Fusion.gba", "METROID");
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 1, "an AppleDouble sidecar reached the shelf");
    assert_eq!(carts[0].stem, "Metroid Fusion");
}

#[test]
fn header_title_of_a_truncated_rom_is_none_not_a_panic() {
    let d = tmp_root();
    std::fs::write(d.path().join("Games/GBA/Tiny.gba"), [0u8; 8]).unwrap();
    let mut cart = scan(d.path()).unwrap().remove(0);
    cart.read_header();
    assert!(cart.title.is_empty());
}

#[test]
fn a_header_title_that_is_not_text_is_dropped_rather_than_mangled() {
    let d = tmp_root();
    let mut rom = vec![0u8; 0x100];
    rom[0xa0..0xac].copy_from_slice(&[0xffu8; 12]);
    std::fs::write(d.path().join("Games/GBA/Garbage.gba"), rom).unwrap();
    let mut cart = scan(d.path()).unwrap().remove(0);
    cart.read_header();
    assert!(cart.title.is_empty());
}

#[test]
fn a_root_with_no_games_directory_scans_as_empty() {
    let d = tempfile::tempdir().unwrap();
    assert!(scan(d.path()).unwrap().is_empty());
}

#[test]
fn a_cart_takes_the_platform_of_the_folder_it_is_in() {
    let d = tmp_root();
    write_rom(&d, "GBA/Metroid Fusion.gba", "METROID");
    std::fs::write(d.path().join("Games/GB/Tetris.gb"), vec![0u8; 0x150]).unwrap();
    std::fs::write(d.path().join("Games/GBC/Chromatic.gbc"), vec![0u8; 0x150]).unwrap();

    let carts = scan(d.path()).unwrap();

    let by_stem = |s: &str| carts.iter().find(|c| c.stem == s).unwrap().platform;
    assert_eq!(by_stem("Metroid Fusion"), Platform::Gba);
    assert_eq!(by_stem("Tetris"), Platform::Gb);
    assert_eq!(by_stem("Chromatic"), Platform::Gbc);
}

#[test]
fn a_gba_rom_in_the_game_boy_folder_does_not_appear() {
    let d = tmp_root();
    std::fs::write(d.path().join("Games/GB/Wrong.gba"), vec![0u8; 0x150]).unwrap();
    assert!(scan(d.path()).unwrap().is_empty());
}

#[test]
fn an_unreadable_platform_folder_does_not_take_the_rest_of_the_library_with_it() {
    let d = tmp_root();
    write_rom(&d, "GBA/Metroid Fusion.gba", "METROID");
    std::fs::write(d.path().join("Games/GB/Tetris.gb"), vec![0u8; 0x150]).unwrap();
    std::fs::remove_dir(d.path().join("Games/GBC")).unwrap();
    std::fs::write(d.path().join("Games/GBC"), b"not a directory").unwrap();

    let carts = scan(d.path()).expect("one bad folder must not fail the whole scan");

    assert_eq!(carts.len(), 2, "the readable shelves were lost too");
    assert!(carts.iter().any(|c| c.stem == "Metroid Fusion"));
    assert!(carts.iter().any(|c| c.stem == "Tetris"));
}

#[test]
fn the_same_stem_on_two_platforms_is_two_carts_with_two_labels() {
    let d = tmp_root();
    write_rom(&d, "GBA/Tetris.gba", "TETRIS");
    std::fs::write(d.path().join("Games/GB/Tetris.gb"), vec![0u8; 0x150]).unwrap();
    std::fs::create_dir_all(d.path().join("Labels/GB")).unwrap();
    std::fs::write(d.path().join("Labels/GB/Tetris.png"), b"\x89PNG\r\n\x1a\n").unwrap();

    let carts = scan(d.path()).unwrap();

    assert_eq!(carts.len(), 2);
    let gb = carts.iter().find(|c| c.platform == Platform::Gb).unwrap();
    let gba = carts.iter().find(|c| c.platform == Platform::Gba).unwrap();
    assert!(gb.label.is_some(), "the Game Boy label was not found");
    assert!(
        gba.label.is_none(),
        "the GBA cart borrowed the Game Boy label"
    );
}

fn write_file(d: &TempDir, rel: &str, body: &str) {
    let path = d.path().join("Games").join(rel);
    std::fs::create_dir_all(path.parent().expect("a game has a folder")).expect("create dir");
    std::fs::write(path, body).expect("write game");
}

#[test]
fn a_multi_disc_game_is_shelved_once_as_its_playlist() {
    let d = tmp_root();
    write_file(&d, "PS/Final Fantasy VII (Disc 1).chd", "");
    write_file(&d, "PS/Final Fantasy VII (Disc 2).chd", "");
    write_file(
        &d,
        "PS/Final Fantasy VII.m3u",
        "# discs\nFinal Fantasy VII (Disc 1).chd\r\nFinal Fantasy VII (Disc 2).chd\n",
    );
    write_file(&d, "PS/Crash Bandicoot.cue", "");
    let stems: Vec<String> = scan(d.path())
        .unwrap()
        .into_iter()
        .filter(|c| c.platform == Platform::PlayStation)
        .map(|c| c.stem)
        .collect();
    assert_eq!(stems, ["Crash Bandicoot", "Final Fantasy VII"]);
}

#[test]
fn a_disc_set_in_its_own_folder_is_shelved_as_the_disc_named_after_it() {
    let d = tmp_root();
    write_file(
        &d,
        "DC/Tony Hawk's Pro Skater 2 (USA)/Tony Hawk's Pro Skater 2 (USA).cue",
        "",
    );
    write_file(
        &d,
        "DC/Tony Hawk's Pro Skater 2 (USA)/Tony Hawk's Pro Skater 2 (USA) (Track 1).bin",
        "",
    );
    write_file(&d, "DC/downloaded_images/Crazy Taxi.png", "");
    write_file(&d, "DC/Crazy Taxi.chd", "");
    let carts: Vec<_> = scan(d.path())
        .unwrap()
        .into_iter()
        .filter(|c| c.platform == Platform::Dreamcast)
        .collect();
    let stems: Vec<&str> = carts.iter().map(|c| c.stem.as_str()).collect();
    assert_eq!(stems, ["Crazy Taxi", "Tony Hawk's Pro Skater 2 (USA)"]);
    assert_eq!(
        carts[1].rom,
        d.path().join("Games/DC/Tony Hawk's Pro Skater 2 (USA)/Tony Hawk's Pro Skater 2 (USA).cue")
    );
}

#[test]
fn a_folder_beside_an_arcade_set_is_not_a_game_of_its_own() {
    let d = tmp_root();
    write_file(&d, "ARCADE/kof98/kof98.zip", "");
    assert!(scan(d.path())
        .unwrap()
        .iter()
        .all(|c| c.platform != Platform::Arcade));
}

#[test]
fn a_playlist_hides_discs_only_on_its_own_shelf() {
    let d = tmp_root();
    write_file(&d, "MD/Sonic.md", "");
    write_file(&d, "PS/Sonic.m3u", "../MD/Sonic.md\n");
    let carts = scan(d.path()).unwrap();
    assert!(
        carts
            .iter()
            .any(|c| c.platform == Platform::Genesis && c.stem == "Sonic"),
        "a playlist on another shelf hid a game it has no business with"
    );
}

#[test]
fn a_system_without_a_header_keeps_its_file_name_as_its_title() {
    let d = tmp_root();
    write_file(&d, "NES/Super Mario Bros. (World).nes", "NES\x1a");
    let mut cart = scan(d.path()).unwrap().remove(0);
    cart.read_header();
    assert_eq!(cart.platform, Platform::Nes);
    assert_eq!(cart.stem, "Super Mario Bros. (World)");
    assert_eq!(
        cart.title, "",
        "a headerless system read a title out of the file"
    );
}

#[test]
fn an_arcade_set_shelves_as_its_game_and_a_bios_not_at_all() {
    let d = tmp_root();
    let dir = d.path().join("Games/NEOGEO");
    std::fs::create_dir_all(&dir).unwrap();
    for set in ["mslug", "neogeo", "kof98"] {
        std::fs::write(dir.join(format!("{set}.zip")), b"PK").unwrap();
    }
    let carts = scan(d.path()).unwrap();
    let shelved: Vec<(&str, &str)> = carts
        .iter()
        .filter(|c| c.platform == Platform::NeoGeo)
        .map(|c| (c.stem.as_str(), c.shelf_name()))
        .collect();
    // In the order of their games' names, not their sets'. Both of these have a Chinese name,
    // so they are shelved under that and sorted by its reading: 合金弹头 before 拳皇'98.
    assert_eq!(
        shelved,
        [
            ("mslug", "合金弹头"),
            ("kof98", "拳皇'98-梦战斗未结束"),
        ]
    );
}

#[test]
fn a_game_in_a_subfolder_is_shelved_with_the_rest() {
    let d = tmp_root();
    write_rom(&d, "GBA/Metroid Fusion.gba", "METROID");
    write_file(&d, "GBA/汉化/Pokemon Emerald.gba", "");
    write_file(&d, "GBA/动作/Sonic Advance.gba", "");
    let carts = scan(d.path()).unwrap();
    let stems: Vec<&str> = carts.iter().map(|c| c.stem.as_str()).collect();
    assert_eq!(
        stems,
        ["Metroid Fusion", "Pokemon Emerald", "Sonic Advance"]
    );
}

#[test]
fn a_disc_set_in_a_subfolder_is_still_one_game() {
    let d = tmp_root();
    write_file(&d, "DC/动作/Crazy Taxi.chd", "");
    write_file(&d, "DC/动作/Tony Hawk/Tony Hawk.cue", "");
    write_file(&d, "DC/动作/Tony Hawk/Tony Hawk (Track 1).bin", "");
    let carts: Vec<_> = scan(d.path())
        .unwrap()
        .into_iter()
        .filter(|c| c.platform == Platform::Dreamcast)
        .collect();
    let stems: Vec<&str> = carts.iter().map(|c| c.stem.as_str()).collect();
    assert_eq!(stems, ["Crazy Taxi", "Tony Hawk"]);
}

#[test]
fn a_cart_knows_which_subfolder_it_sits_in() {
    let d = tmp_root();
    write_file(&d, "GBA/Tetris.gba", "");
    write_file(&d, "GBA/汉化/Pokemon Emerald.gba", "");
    write_file(&d, "DC/动作/Tony Hawk/Tony Hawk.cue", "");
    let carts = scan(d.path()).unwrap();
    let folder = |stem: &str| {
        let cart = carts.iter().find(|c| c.stem == stem).expect("cart");
        folder_of(cart.platform, &cart.rom)
    };
    assert_eq!(
        folder("Tetris"),
        None,
        "a cart in the system's root has no folder"
    );
    assert_eq!(folder("Pokemon Emerald").as_deref(), Some("汉化"));
    assert_eq!(
        folder("Tony Hawk").as_deref(),
        Some("动作"),
        "a disc set's own folder is the game, not a folder of games"
    );
}

#[test]
fn a_set_the_databases_do_not_know_keeps_its_file_name() {
    let d = tmp_root();
    let dir = d.path().join("Games/ARCADE");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("myhack.zip"), b"PK").unwrap();
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts[0].shelf_name(), "myhack");
}

/// Writes an RPG Maker game: a folder with the engine's database in it, and the map files that
/// must not be shelved as games of their own.
fn write_rpg(d: &TempDir, game: &str) {
    write_file(&d, &format!("RPGMAKER/{game}/RPG_RT.ldb"), "ldb");
    write_file(&d, &format!("RPGMAKER/{game}/Map0001.lmu"), "map");
}

#[test]
fn an_rpg_maker_game_is_the_folder_it_comes_in() {
    let d = tmp_root();
    write_rpg(&d, "7DAYS");
    let carts = scan(d.path()).unwrap();
    let carts: Vec<_> = carts
        .into_iter()
        .filter(|c| c.platform == Platform::RpgMaker)
        .collect();
    assert_eq!(carts.len(), 1, "one folder, one game");
    assert_eq!(carts[0].stem, "7DAYS");
    assert_eq!(carts[0].rom, d.path().join("Games/RPGMAKER/7DAYS"));
    assert!(carts[0].rom.is_dir());
}

#[test]
fn a_game_folder_hidden_the_way_other_cards_hide_them_is_still_a_game() {
    let d = tmp_root();
    write_rpg(&d, ".7DAYS");
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 1, "a hidden game folder was passed over");
    assert_eq!(carts[0].stem, "7DAYS", "the dot is not part of its name");
    // Its label is looked for under the name the shelf shows, not the one on the card.
    write_png(&d, "RPGMAKER/7DAYS.png");
    let cart = scan(d.path()).unwrap().remove(0);
    assert!(cart.label.is_some(), "the label was not found");
}

#[test]
fn nothing_inside_a_game_folder_is_shelved_of_its_own() {
    let d = tmp_root();
    write_rpg(&d, "OFF 汉化版");
    write_file(&d, "RPGMAKER/OFF 汉化版/Music/town.mid", "");
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 1, "the game's own files became games");
    assert_eq!(carts[0].stem, "OFF 汉化版");
}

#[test]
fn a_game_folder_in_a_subfolder_is_shelved_with_the_rest() {
    let d = tmp_root();
    write_rpg(&d, "Mogeko城堡");
    write_rpg(&d, "汉化/Lisa The First");
    let carts = scan(d.path()).unwrap();
    let stems: Vec<&str> = carts.iter().map(|c| c.stem.as_str()).collect();
    assert_eq!(stems, ["Lisa The First", "Mogeko城堡"]);
    assert_eq!(
        folder_of(carts[0].platform, &carts[0].rom).as_deref(),
        Some("汉化"),
        "a game that is a folder still knows which folder it sits in"
    );
}

#[test]
fn a_zipped_rpg_maker_game_is_a_game_too() {
    let d = tmp_root();
    write_file(&d, "RPGMAKER/Oneshot.zip", "PK");
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 1);
    assert_eq!(carts[0].stem, "Oneshot");
    assert!(carts[0].rom.is_file());
}

#[test]
fn a_folder_of_nothing_an_engine_knows_is_walked_into_like_any_other() {
    let d = tmp_root();
    write_file(&d, "RPGMAKER/汉化/ground.zip", "PK");
    // A folder whose name the engine does not answer to is just a folder of games.
    write_file(&d, "RPGMAKER/声音/README.txt", "");
    assert_eq!(scan(d.path()).unwrap().len(), 1);
}
