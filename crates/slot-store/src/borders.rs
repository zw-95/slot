//! The cart borders a card carries: `System/Borders/<machine><number>.png`.
//!
//! A cart's shell is drawn in code — an outline, a shell colour, a label plate — and that is
//! right for the one shape a machine's carts have. But a machine's carts did not all look alike,
//! and a card can carry pictures of the ones they did. One directory, one picture per border, and
//! **the number on the filename is the whole of what orders them**: `gba1.png`, `gba2.png`,
//! `sfc1.png`, `fc3.png`.
//!
//! **Nothing in here knows how many borders a machine has**, and that is the point of the
//! directory: a count written in code is a count that has to be edited every time a picture is
//! dropped on the card, and a card with a border the program has not heard of would otherwise be
//! a card showing a border it does not have. The card is the list; this only reads it.

use std::path::{Path, PathBuf};
use crate::System;

/// Where they live on the card, beside `cores` and under the same rule: a directory the user can
/// open on a PC and drop a PNG into.
pub const DIR: &str = "System/Borders";

/// Where a **card** keeps the drawings its borders are made from. Each one is a cart drawn in grey
/// plastic with a magenta label slot, and every colour it comes in is that same drawing dyed at
/// runtime — see `slot_ui::cart_art::border_art`. One template per shape, eleven colours each, and
/// the card carries no picture per colour at all.
pub const BASE: &str = "base";

/// How many colours a template is offered in. The list itself lives in the rasteriser, because
/// that is the only place a colour is anything but a triple of bytes; this is the number the
/// numbering below multiplies by, and the two have to agree.
pub const SHELL_COUNT: u32 = 11;

/// One border: the file it is drawn from, and the number it answers to.
///
/// **The file is a template and not the border.** Two entries may name the same drawing: they are
/// one shape in two colours, and the number is what says which colour. See `borders`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Border {
    pub path: PathBuf,
    pub number: u32,
}

impl Border {
    /// What it is called, which is the filename and nothing else: the number is the ordering, not
    /// a name worth showing, and a border is a picture.
    pub fn name(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string()
    }
}


/// The borders a card has for one machine, in the order they are numbered.
///
/// Empty when the directory is missing, which is every card that has not been given any: a machine
/// with no borders is a machine whose carts keep the shells they were born with, and that is not
/// an error to report, it is the state every card starts in.
///
/// A file that cannot be read, or a PNG that will not decode, is still listed — the border is on
/// the card and the user can see it there. What is *drawn* with a picture that would not decode
/// is decided by the rasteriser, which answers with nothing and falls back to the shell in code.
pub fn borders(root: &Path, system: System) -> Vec<Border> {
    let mut out = Vec::new();
    // Template by template, colour by colour — **the shapes outer loop, the colours inner**, so
    // every colour of one drawing sits together and stepping the ring walks shape after shape.
    let shapes = templates(root, system);
    for (shape, path) in shapes.iter().enumerate() {
        for colour in 0..SHELL_COUNT {
            out.push(Border {
                path: path.clone(),
                number: (shape as u32) * SHELL_COUNT + colour + 1,
            });
        }
    }
    out
}

/// The grey drawings this machine's borders are made from: every `base/<machine>_base_NN.png` on
/// the card, in name order.
///
/// A shape is a shape and nothing else here — the count always answers how many drawings there
/// are, never how many borders, which is that number times the colours. Empty is not an error to
/// report: a card with no drawings for this machine is a card whose carts keep the shells they
/// were born with, which is the state every card starts in.
pub fn templates(root: &Path, system: System) -> Vec<PathBuf> {
    let Ok(dir) = std::fs::read_dir(root.join(DIR).join(BASE)) else {
        return Vec::new();
    };
    let prefix = format!("{}_base_", system.dir_name().to_ascii_lowercase());
    let mut out: Vec<PathBuf> = dir
        .filter_map(|e| {
            let e = e.ok()?;
            if !e.file_type().ok()?.is_file() {
                return None;
            }
            // Lower-cased before the name is read, because the card is edited on a filesystem
            // that does not care and hands back `MD_base_01.png` where `md_base_01` was written.
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            let rest = name.strip_prefix(&prefix)?.strip_suffix(".png")?;
            // Digits only: `_base_final2` is somebody's draft, not a shape.
            if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            Some(e.path())
        })
        .collect();
    // By name, so `_base_02` follows `_base_01` on every card rather than in whatever order the
    // card's own directory hands them back.
    out.sort();
    out
}

/// How many borders a card has for one machine, which is the number the picker counts with and
/// the only answer to "is there anything to choose between" it needs.
pub fn count(root: &Path, system: System) -> usize {
    borders(root, system).len()
}

/// The one a cart is wearing: `System/Borders/<machine>.txt`, **one file per machine**.
///
/// `<rom stem> = <number>`, the same shape as the core's own file and keyed the same way, but
/// kept apart by machine: a machine added later is a file added later, and nothing that reads the
/// Game Boy's choices has to know a Super Nintendo exists. Two carts called `Tetris` on two
/// machines are two carts, and a file shared between them would have to invent a name for the
/// pair.
///
/// A stem with no line is `None`, which is the machine's own shell rather than a border — **that
/// is the state a cart is in until somebody picks one**, and it is not the same as having a
/// border chosen and then losing it.
///
/// Out-of-range numbers are dropped rather than clamped: a card whose borders were edited on a PC
/// may have fewer than it had, and the honest reading of "the third border" when there are two is
/// the machine's own shell, not whichever border now sits third.
pub fn read_selected(root: &Path, system: System) -> std::collections::HashMap<String, u32> {
    let text = match std::fs::read_to_string(selection_path(root, system)) {
        Ok(t) => t,
        Err(_) => return std::collections::HashMap::new(),
    };
    let mut out = std::collections::HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((stem, number)) = line.split_once('=') else {
            continue;
        };
        let Ok(number) = number.trim().parse::<u32>() else {
            continue;
        };
        let stem = stem.trim();
        if !stem.is_empty() {
            out.insert(stem.to_string(), number);
        }
    }
    out
}

/// Where a machine's choices live: **beside that machine's own pictures**, so the directory is the
/// whole of the feature and a machine nobody has chosen for leaves no trace on the card.
fn selection_path(root: &Path, system: System) -> std::path::PathBuf {
    root.join(DIR)
        .join(format!("{}.txt", system.dir_name().to_ascii_lowercase()))
}

/// Which border a cart wears. `None` writes nothing at all, so a cart put back on its machine's
/// own shell leaves no line behind it — and a machine nothing was ever chosen for keeps no file.
pub fn write_selected(
    root: &Path,
    system: System,
    stem: &str,
    number: Option<u32>,
) -> std::io::Result<()> {
    let path = selection_path(root, system);
    let mut map = read_selected(root, system);
    match number {
        Some(n) => map.insert(stem.to_string(), n),
        None => map.remove(stem),
    };
    if map.is_empty() {
        let _ = std::fs::remove_file(path);
        return Ok(());
    }
    let mut lines: Vec<String> = map
        .iter()
        .map(|(stem, n)| format!("{stem} = {n}"))
        .collect();
    lines.sort();
    let mut text = lines.join("\n");
    text.push('\n');
    std::fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join(DIR).join(BASE)).unwrap();
        d
    }

    /// A **drawing**, which is not a border: one grey shape, and the raw material of eleven of
    /// them. The card is edited on a PC, so nothing here ever needs a file per colour.
    fn put(d: &tempfile::TempDir, name: &str) {
        std::fs::write(
            d.path().join(DIR).join(BASE).join(name),
            b"not really a png",
        )
        .unwrap();
    }

    /// The whole point of the directory: **the count comes off the card**, so a drawing dropped on
    /// it is a border the picker offers without anybody editing a list — and offered in every
    /// colour at that, because the colours belong to the code and the shapes to the card.
    #[test]
    fn the_card_is_the_list_and_not_a_number_in_the_code() {
        let d = card();
        assert!(borders(d.path(), System::Sfc).is_empty(), "a card with none");
        for n in 1..=4 {
            put(&d, &format!("sfc_base_{n:02}.png"));
        }
        assert_eq!(count(d.path(), System::Sfc), 4 * SHELL_COUNT as usize);
        // And the fifth, from a card edited on a PC — nothing here had to change for it.
        put(&d, "sfc_base_05.png");
        assert_eq!(count(d.path(), System::Sfc), 5 * SHELL_COUNT as usize);
    }

    #[test]
    fn a_border_belongs_to_the_machine_in_its_name() {
        let d = card();
        put(&d, "sfc_base_01.png");
        put(&d, "sfc_base_02.png");
        put(&d, "gba_base_01.png");
        put(&d, "fc_base_01.png");
        assert_eq!(count(d.path(), System::Sfc), 2 * SHELL_COUNT as usize);
        assert_eq!(count(d.path(), System::Gba), SHELL_COUNT as usize);
        assert_eq!(count(d.path(), System::Gb), 0, "no Game Boy drawings on this card");
    }

    /// Numbered, not read off the directory: a card's own order is whoever copied them there, and
    /// `_base_10` arriving before `_base_02` would put the picker's ring in an order nobody wrote
    /// down — which is why the drawings are sorted by name before the colours are handed out.
    #[test]
    fn they_come_back_in_the_order_they_are_numbered() {
        let d = card();
        for n in [1, 2, 10, 3] {
            put(&d, &format!("gba_base_{n:02}.png"));
        }
        let got: Vec<u32> = borders(d.path(), System::Gba).iter().map(|b| b.number).collect();
        // Shape after shape — 01, 02, 03, 10 — and inside each one, its colours from first to last.
        let want: Vec<u32> = (0..4)
            .flat_map(|shape| (1..=SHELL_COUNT).map(move |c| shape * SHELL_COUNT + c))
            .collect();
        assert_eq!(got, want);
    }

    /// A file that is not a drawing is not a drawing: another machine's, a stray draft, a
    /// directory. Nothing here decides what is inside the PNG — that is the rasteriser's answer,
    /// and a drawing it cannot open is still one the card carries.
    #[test]
    fn anything_that_is_not_numbered_for_the_machine_is_passed_over() {
        let d = card();
        put(&d, "sfc_base_01.png");
        put(&d, "sfc_base_draft.png"); // not numbered
        put(&d, "gba_base_01.png"); // another machine's
        std::fs::write(
            d.path().join(DIR).join(BASE).join("readme.txt"),
            b"notes",
        )
        .unwrap();
        std::fs::create_dir_all(d.path().join(DIR).join(BASE).join("sfc_base_02.png")).unwrap();
        assert_eq!(count(d.path(), System::Sfc), SHELL_COUNT as usize);
        assert_eq!(borders(d.path(), System::Sfc)[0].number, 1);
    }

    /// A card that has never been given the directory at all — which is every card in existence
    /// until this week — reads as none rather than as an error.
    #[test]
    fn a_card_with_no_borders_directory_reads_as_none() {
        let d = tempfile::tempdir().unwrap();
        assert!(borders(d.path(), System::Sfc).is_empty());
        assert_eq!(count(d.path(), System::Sfc), 0);
    }

    /// The choice is per cart **and per machine**, written and read back, and a cart put back on
    /// its machine's own shell leaves no line behind it.
    #[test]
    fn a_cart_s_choice_survives_the_file_and_can_be_taken_away_again() {
        let d = card();
        write_selected(d.path(), System::Sfc, "Super Mario World", Some(3)).unwrap();
        write_selected(d.path(), System::Sfc, "Tetris", Some(1)).unwrap();
        let map = read_selected(d.path(), System::Sfc);
        assert_eq!(map.get("Super Mario World"), Some(&3));
        assert_eq!(map.get("Tetris"), Some(&1));
        assert_eq!(map.get("Emerald"), None, "a cart nobody chose for is not chosen");
        write_selected(d.path(), System::Sfc, "Tetris", None).unwrap();
        assert_eq!(read_selected(d.path(), System::Sfc).get("Tetris"), None);
        assert_eq!(
            read_selected(d.path(), System::Sfc).get("Super Mario World"),
            Some(&3),
            "and the other cart's choice is still there"
        );
    }

    /// One file per machine, which is the whole point of the split: a cart of the same name on two
    /// machines is two carts, and neither machine's file says anything about the other's.
    #[test]
    fn the_same_cart_on_two_machines_is_two_choices_in_two_files() {
        let d = card();
        write_selected(d.path(), System::Sfc, "Tetris", Some(4)).unwrap();
        write_selected(d.path(), System::Gb, "Tetris", Some(2)).unwrap();
        assert_eq!(read_selected(d.path(), System::Sfc).get("Tetris"), Some(&4));
        assert_eq!(read_selected(d.path(), System::Gb).get("Tetris"), Some(&2));
        // And a third machine that has never been chosen for reads as nothing at all, with no file
        // written for it.
        assert!(read_selected(d.path(), System::Gba).is_empty());
        assert!(
            !d.path().join(DIR).join("gba.txt").exists(),
            "a machine nobody chose for left a file behind"
        );
        assert!(d.path().join(DIR).join("sfc.txt").exists());
        assert!(d.path().join(DIR).join("gb.txt").exists());
        // Taking the Game Boy's only choice away takes the whole file with it.
        write_selected(d.path(), System::Gb, "Tetris", None).unwrap();
        assert!(!d.path().join(DIR).join("gb.txt").exists());
        assert_eq!(
            read_selected(d.path(), System::Sfc).get("Tetris"),
            Some(&4),
            "and the Super Nintendo's is untouched"
        );
    }

    /// A machine's own choices file is not one of that machine's pictures: it sits in the same
    /// directory and it is not a PNG.
    #[test]
    fn a_machine_s_choices_are_not_counted_as_one_of_its_borders() {
        let d = card();
        put(&d, "sfc_base_01.png");
        write_selected(d.path(), System::Sfc, "Tetris", Some(1)).unwrap();
        assert_eq!(count(d.path(), System::Sfc), SHELL_COUNT as usize);
    }

    /// A typo on a card costs that one cart its border, not the whole file: the same bargain the
    /// core's own file makes.
    #[test]
    fn a_malformed_line_costs_one_cart_and_not_the_file() {
        let d = card();
        std::fs::write(
            d.path().join(DIR).join("sfc.txt"),
            "Super Mario World = 2\nEmerald = two\n# a note\n\nTetris=1\n",
        )
        .unwrap();
        let map = read_selected(d.path(), System::Sfc);
        assert_eq!(map.get("Super Mario World"), Some(&2));
        assert_eq!(map.get("Tetris"), Some(&1), "spaces round the = are nothing");
        assert_eq!(map.get("Emerald"), None);
    }
}
