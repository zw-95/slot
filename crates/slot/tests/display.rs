//! What a card's `System/display.txt` is allowed to ask for.
//!
//! The file is one line — `mask cc_gba cc_gb cc_gbc cc_sfc cc_fc fit` — and it is the only thing
//! that survives a reboot, so every preset the interface can reach has to come back as itself.

mod common;

use common::tmp_root_with_carts;
use slot::root::MASK_STATES;
use slot::root::{display_modes, write_display_modes};

#[test]
fn the_whole_ring_survives_the_file() {
    // Every preset the Advance's ring holds, written and read back. The clamp that reads the file
    // was left on the old count of five when the scanline was added, so the two newest presets
    // were written, shown, persisted — and then silently read back as LCD3x 100% on the next boot.
    // Nothing else in the tree could have caught it: the file was always the right file, and the
    // number in it was always the right number.
    let d = tmp_root_with_carts(&["Emerald"]);
    for mask in 0..MASK_STATES as u8 {
        write_display_modes(d.path(), mask, [3, 7, 2, 11, 13], 0);
        let (read, cc, fit) = display_modes(d.path());
        assert_eq!(read, mask, "preset {mask} did not survive the file");
        assert_eq!(cc, [3, 7, 2, 11, 13], "and neither did the five colour slots");
        assert_eq!(fit, 0, "nor the fit");
    }
    // Past the end clamps to the last preset rather than wrapping, for a file edited on a PC.
    write_display_modes(d.path(), 99, [0, 0, 0, 0, 0], 0);
    assert_eq!(display_modes(d.path()).0 as usize, MASK_STATES - 1);
}

/// The fit is the newest number in the file, and a card that has never pressed SELECT+B has no
/// number for it at all. What it is owed is the fit it has been showing all along — whole source
/// pixels — rather than whichever of the three happens to be first in some other list.
#[test]
fn every_fit_survives_the_file_and_a_file_without_one_reads_the_first() {
    let d = tmp_root_with_carts(&["Emerald"]);
    for fit in 0..3u8 {
        write_display_modes(d.path(), 2, [0, 0, 0, 0, 0], fit);
        assert_eq!(display_modes(d.path()).2, fit, "fit {fit} did not survive");
    }
    // The five- and six-number files the Family Computer's slot and then the fit grew the line
    // past: neither has a fit in it, and both read the one the panel was built around.
    let file = d.path().join("System/display.txt");
    std::fs::write(&file, "2 5 300 1 7").unwrap();
    assert_eq!(display_modes(d.path()).2, 0, "a file with no fit reads the first");
    std::fs::write(&file, "2 5 300 1 7 5").unwrap();
    assert_eq!(display_modes(d.path()).2, 0, "and so does one written before the fit existed");
    // Past the end clamps, as every other index in the file does.
    std::fs::write(&file, "2 0 0 0 0 0 9").unwrap();
    assert!(display_modes(d.path()).2 < 3, "a fit off the end of the ring");
}

#[test]
fn a_file_from_before_the_super_nintendo_gives_it_the_advances_colour() {
    // The file used to hold one colour fewer. A card written then has no number for the fourth
    // machine, and the reading that keeps that card looking like itself is the one the colour
    // machines already had — not a silent drop to grayscale. Every machine added after it
    // inherits the same way, and for the same reason.
    let d = tmp_root_with_carts(&["Emerald"]);
    let file = d.path().join("System/display.txt");
    std::fs::write(&file, "2 5 300 1").unwrap();
    assert_eq!(
        display_modes(d.path()).1,
        [5, 300, 1, 5, 5],
        "the Super Nintendo takes the Advance's grade"
    );

    std::fs::write(&file, "2 5 300 1 7").unwrap();
    assert_eq!(
        display_modes(d.path()).1,
        [5, 300, 1, 7, 5],
        "and so does the Family Computer"
    );
}

#[test]
fn a_missing_or_older_file_still_reads() {
    // A card from before the colour machines had a slot each, and one with nothing at all. Neither
    // is an error: the older two-integer file meant "whatever was chosen, for every machine", and
    // that is still what it means.
    let d = tmp_root_with_carts(&["Emerald"]);
    let file = d.path().join("System/display.txt");
    let _ = std::fs::remove_file(&file);
    assert_eq!(
        display_modes(d.path()),
        (2, [0, 0, 0, 0, 0], 0),
        "the built-in default"
    );

    std::fs::write(&file, "1 4").unwrap();
    assert_eq!(
        display_modes(d.path()),
        (1, [4, 4, 4, 4, 4], 0),
        "one colour, every machine"
    );

    std::fs::write(&file, "not a number").unwrap();
    assert_eq!(
        display_modes(d.path()),
        (2, [0, 0, 0, 0, 0], 0),
        "and it does not panic on nonsense"
    );
}
