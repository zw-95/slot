//! A core's name has to survive being written and read back.
//!
//! `write_selected_core` writes `Core::as_str` into `selected_core.ini` on the card, and
//! `read_selected_cores` reads it back with `Core::parse`. Those are two lists in two functions
//! and nothing has ever held them against each other, so a name one of them spells and the other
//! does not know is a cart that silently forgets its core: the choice is written, shown, and then
//! replaced by the machine's default on the next read. That is what happened to `snes9x2010`
//! until 2026-10-24 — pick it, confirm, open the picker again, and the first core was back.

use slot_store::Core;

/// Every core this build can name, spelled the way it writes itself, and read back as itself.
///
/// The test that would have caught the Super Nintendo's 2010 core by the collar: `as_str` gave
/// `snes9x2010` and `parse` had no such name, so every cart that chose it came back on `snes9x`.
#[test]
fn every_core_round_trips_through_its_own_name() {
    for core in Core::ALL {
        assert_eq!(
            Core::parse(core.as_str()),
            Some(core),
            "{:?} writes itself as {:?} and cannot read that back",
            core,
            core.as_str()
        );
    }
}

/// And the other way about: every name `parse` answers has to be a name `as_str` can produce, or
/// the ini would be carrying a spelling nothing writes and nothing else recognises.
#[test]
fn every_name_parse_answers_is_a_name_as_str_writes() {
    // The spellings a hand is likelier to type, which `parse` answers and `as_str` does not write.
    for alias in [
        "same_boy",
        "vba-m",
        "vbanext",
        "beetle_gba",
        "mgbarumble",
        "supafaust",
        "snes9x2005plus",
        "snes9x_2010",
        "snes9x-2010",
        "snes9xnext",
        "mesen",
        "bsnes2014",
        "bsnes_mercury",
        "chimerasnes",
        "fceum",
    ] {
        let core = Core::parse(alias).unwrap_or_else(|| panic!("{alias} parses to nothing"));
        assert_eq!(Core::parse(core.as_str()), Some(core), "{alias} -> {core:?}");
    }
}

/// The Super Nintendo's own list, which is where the missing name was: thirteen cores, every one
/// of them a name the card can be told to use.
#[test]
fn a_chosen_super_nintendo_core_is_the_core_that_comes_back() {
    let path = std::env::temp_dir().join(format!("slot-core-round-trip-{}", std::process::id()));
    let _ = std::fs::create_dir_all(path.join("System"));
    let _ = std::fs::remove_file(path.join("System/selected_core.ini"));
    for core in slot_store::System::Sfc.cores() {
        slot_store::write_selected_core(&path, slot_store::System::Sfc, "Super Mario World", *core)
            .expect("could not write");
        let read = slot_store::core_for(&path, slot_store::System::Sfc, "Super Mario World");
        assert_eq!(read, *core, "{core:?} came back as {read:?}");
    }
    let _ = std::fs::remove_file(path.join("System/selected_core.ini"));
}
