use slot_store::{core_for, read_selected_cores, Core, System};
use tempfile::tempdir;

fn root_with(ini: Option<&str>) -> tempfile::TempDir {
    let d = tempdir().unwrap();
    std::fs::create_dir(d.path().join("System")).unwrap();
    if let Some(text) = ini {
        std::fs::write(d.path().join("System/selected_core.ini"), text).unwrap();
    }
    d
}

#[test]
fn absent_file_means_everything_defaults() {
    let d = root_with(None);
    assert!(read_selected_cores(d.path()).is_empty());
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Mgba);
    assert_eq!(core_for(d.path(), System::Sfc, "Contra"), Core::Supafaust);
}

#[test]
fn a_listed_stem_gets_its_core() {
    let d = root_with(Some("GBA/Pokemon - Emerald Version (USA, Europe) = gpsp\n"));
    assert_eq!(
        core_for(
            d.path(),
            System::Gba,
            "Pokemon - Emerald Version (USA, Europe)"
        ),
        Core::Gpsp
    );
}

#[test]
fn an_unlisted_stem_defaults_to_mgba() {
    let d = root_with(Some("GBA/Emerald = gpsp\n"));
    assert_eq!(core_for(d.path(), System::Gba, "Metroid Fusion"), Core::Mgba);
}

/// A card is user editable. Every one of these is a typo someone will make, and not one
/// of them may cost them their shelf.
#[test]
fn malformed_lines_are_ignored_rather_than_fatal() {
    let d = root_with(Some(concat!(
        "\n",
        "# a comment\n",
        "; another comment\n",
        "[cores]\n",
        "no equals sign here\n",
        "GBA/Emerald = notacore\n",
        "  Spaced Out   =   gpsp  \n",
        "= gpsp\n",
        "Trailing =\n",
    )));
    let map = read_selected_cores(d.path());
    assert_eq!(
        map.get("Spaced Out"),
        Some(&Core::Gpsp),
        "whitespace not trimmed"
    );
    assert_eq!(
        map.get("GBA/Emerald"),
        None,
        "an unknown core name must not be stored"
    );
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Mgba);
    assert_eq!(map.len(), 1, "only the one good line should survive");
}

#[test]
fn a_later_duplicate_wins() {
    let d = root_with(Some("GBA/Emerald = mgba\nGBA/Emerald = gpsp\n"));
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Gpsp);
}

/// A stem is not a cart: `Contra` is a Super Nintendo game and, one of these days, a Mega
/// Drive one. The machine is part of the key so a card holding both does not have one line
/// answering for two carts.
#[test]
fn the_same_stem_on_two_machines_answers_for_itself() {
    let d = root_with(Some("GBA/Contra = gpsp\nSFC/Contra = snes9x2010\n"));
    assert_eq!(core_for(d.path(), System::Gba, "Contra"), Core::Gpsp);
    assert_eq!(core_for(d.path(), System::Sfc, "Contra"), Core::Snes9x2010);
}

/// Every line on every card was written before the machine was part of the key, and the only
/// machine that ever had a choice was the Advance. Those lines must keep meaning what they
/// meant, or an upgrade silently forgets how every cart on the card was set.
#[test]
fn a_bare_stem_is_still_the_advances() {
    let d = root_with(Some("Emerald = gpsp\n"));
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Gpsp);
    // And is nobody else's: a Super Nintendo cart of the same name has its own line to write.
    assert_eq!(core_for(d.path(), System::Sfc, "Emerald"), Core::Supafaust);
}

/// A hand edited file is going to name cores from the wrong machine eventually. That is a
/// typo, not a wish to run a Super Nintendo cart in a Game Boy Advance emulator.
#[test]
fn a_core_the_machine_cannot_run_is_not_an_answer() {
    let d = root_with(Some("SFC/Contra = mgba\nGB/Contra = gpsp\nGBC/Contra = gpsp\n"));
    assert_eq!(core_for(d.path(), System::Sfc, "Contra"), Core::Supafaust);
    assert_eq!(core_for(d.path(), System::Gb, "Contra"), Core::Mgba);
    assert_eq!(core_for(d.path(), System::Gbc, "Contra"), Core::Mgba);
}

#[test]
fn every_machine_offers_at_least_one_core_and_its_first_is_its_default() {
    for system in [System::Gba, System::Gb, System::Gbc, System::Sfc] {
        let cores = system.cores();
        assert!(!cores.is_empty(), "{system:?} offers nothing to run it");
        assert_eq!(system.default_core(), cores[0]);
        assert_eq!(
            core_for(
                &tempdir().unwrap().path().to_path_buf(),
                system,
                "Anything"
            ),
            cores[0]
        );
    }
    // The picker is sockets to hop between, and one socket is not a choice.
    assert_eq!(System::Gb.cores().len(), 1);
    assert!(System::Gba.cores().len() >= 2);
    assert!(System::Sfc.cores().len() >= 2);
}

#[test]
fn core_names_round_trip() {
    for c in Core::ALL {
        assert_eq!(Core::parse(c.as_str()), Some(c));
    }
    assert_eq!(
        Core::parse("MGBA"),
        Some(Core::Mgba),
        "case is not the user's problem"
    );
    assert_eq!(Core::parse("nonsense"), None);
    assert_eq!(Core::default(), Core::Mgba);
}

#[test]
fn writing_a_core_creates_the_file_when_absent() {
    let d = root_with(None);
    slot_store::write_selected_core(d.path(), System::Gba, "Emerald", Core::Gpsp).unwrap();
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Gpsp);
}

#[test]
fn writing_a_core_replaces_that_carts_line_and_leaves_the_rest_alone() {
    let d = root_with(Some(concat!(
        "# my notes\n",
        "\n",
        "GBA/Emerald = mgba\n",
        "GBA/Metroid Fusion = gpsp\n",
    )));
    slot_store::write_selected_core(d.path(), System::Gba, "Emerald", Core::Gpsp).unwrap();

    let text = std::fs::read_to_string(d.path().join("System/selected_core.ini")).unwrap();
    assert!(
        text.contains("# my notes"),
        "a hand-written comment was destroyed"
    );
    assert!(
        text.contains("GBA/Metroid Fusion = gpsp"),
        "another cart's entry was lost"
    );
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Gpsp);
    assert_eq!(core_for(d.path(), System::Gba, "Metroid Fusion"), Core::Gpsp);
    assert_eq!(
        text.matches("Emerald").count(),
        1,
        "the old line was left behind"
    );
}

/// The cart's line may already be there in the old spelling, written by a build from before
/// the machine was part of the key. Rewriting it in place is what keeps the file from saying
/// two different things about one cart.
#[test]
fn writing_over_a_bare_stem_rewrites_it_rather_than_adding_a_second_line() {
    let d = root_with(Some("Emerald = mgba\n"));
    slot_store::write_selected_core(d.path(), System::Gba, "Emerald", Core::Gpsp).unwrap();

    let text = std::fs::read_to_string(d.path().join("System/selected_core.ini")).unwrap();
    assert!(
        text.contains("GBA/Emerald = gpsp"),
        "the new spelling was not written: {text}"
    );
    assert_eq!(
        text.matches("Emerald").count(),
        1,
        "both spellings are on the card now: {text}"
    );
}

#[test]
fn writing_a_core_appends_a_cart_the_file_has_never_seen() {
    let d = root_with(Some("GBA/Emerald = gpsp\n"));
    slot_store::write_selected_core(d.path(), System::Gba, "Drill Dozer", Core::Gpsp).unwrap();
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Gpsp);
    assert_eq!(core_for(d.path(), System::Gba, "Drill Dozer"), Core::Gpsp);
}

#[test]
fn writing_the_default_still_records_it() {
    // Not a no-op: a cart set to gpsp and then back to mgba must actually change, and the
    // absence of a line means "default", which is the same answer for a different reason.
    let d = root_with(Some("GBA/Emerald = gpsp\n"));
    slot_store::write_selected_core(d.path(), System::Gba, "Emerald", Core::Mgba).unwrap();
    assert_eq!(core_for(d.path(), System::Gba, "Emerald"), Core::Mgba);
    let text = std::fs::read_to_string(d.path().join("System/selected_core.ini")).unwrap();
    assert!(text.contains("GBA/Emerald = mgba"));
}

/// A Super Nintendo cart picks between three, and the picker writes the one it was left on.
#[test]
fn writing_a_super_nintendo_core_sticks_to_that_cart() {
    let d = root_with(None);
    slot_store::write_selected_core(d.path(), System::Sfc, "Contra", Core::Snes9x2010).unwrap();
    assert_eq!(core_for(d.path(), System::Sfc, "Contra"), Core::Snes9x2010);
    // And does not reach the Advance's cart of the same name.
    assert_eq!(core_for(d.path(), System::Gba, "Contra"), Core::Mgba);
}
