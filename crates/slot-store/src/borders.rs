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

/// One border: the file, and the number off its name.
///
/// The number is kept because it is the only thing that says what order the borders come in —
/// a directory read hands back the card's own order, which is whoever copied them there.
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

/// `sfc12.png` for the Super Nintendo, and `None` for anything that is not that machine's: the
/// picture belongs to the machine whose prefix it carries, so a card can hold five machines'
/// borders in one directory without any of them having to sort the others out.
fn parse(system: System, name: &str) -> Option<u32> {
    let stem = name.strip_suffix(".png").or_else(|| name.strip_suffix(".PNG"))?;
    let prefix = system.dir_name().to_ascii_lowercase();
    let rest = stem.strip_prefix(&prefix)?;
    // Digits only, and at least one: `sfc1` is a border, `sfc_label` is somebody else's file.
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse().ok().filter(|n: &u32| *n > 0)
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
    let dir = match std::fs::read_dir(root.join(DIR)) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<Border> = dir
        .filter_map(|e| {
            let e = e.ok()?;
            if !e.file_type().ok()?.is_file() {
                return None;
            }
            let name = e.file_name();
            let number = parse(system, name.to_str()?)?;
            Some(Border {
                path: e.path(),
                number,
            })
        })
        .collect();
    // Numbered, then by name so two borders carrying the same number come back in the same order
    // on every boot rather than in whatever order the card's directory happens to be in.
    out.sort_by(|a, b| a.number.cmp(&b.number).then_with(|| a.path.cmp(&b.path)));
    out
}

/// How many borders a card has for one machine, which is the number the picker counts with and
/// the only answer to "is there anything to choose between" it needs.
pub fn count(root: &Path, system: System) -> usize {
    borders(root, system).len()
}

/// The one a cart is wearing, and which one the card says it is wearing.
///
/// `<rom stem> = <number>`, the same file the core's choice lives in and keyed the same way.
/// A stem with no line is `None`, which is the machine's own shell rather than a border — **that
/// is the state a cart is in until somebody picks one**, and it is not the same as having a
/// border chosen and then losing it.
///
/// Out-of-range numbers are dropped rather than clamped: a card whose borders were edited on a PC
/// may have fewer than it had, and the honest reading of "the third border" when there are two is
/// the machine's own shell, not whichever border now sits third.
pub fn read_selected(root: &Path) -> std::collections::HashMap<String, u32> {
    let text = match std::fs::read_to_string(root.join("System/borders.txt")) {
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

/// Which border a cart wears. `None` writes nothing at all, so a cart put back on its machine's
/// own shell leaves no line behind it.
pub fn write_selected(root: &Path, stem: &str, number: Option<u32>) -> std::io::Result<()> {
    let path = root.join("System/borders.txt");
    let mut map = read_selected(root);
    match number {
        Some(n) => map.insert(stem.to_string(), n),
        None => map.remove(stem),
    };
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
        std::fs::create_dir_all(d.path().join(DIR)).unwrap();
        d
    }

    fn put(d: &tempfile::TempDir, name: &str) {
        std::fs::write(d.path().join(DIR).join(name), b"not really a png").unwrap();
    }

    /// The whole point of the directory: **the count comes off the card**, so a border dropped on
    /// it is a border the picker offers without anybody editing a list.
    #[test]
    fn the_card_is_the_list_and_not_a_number_in_the_code() {
        let d = card();
        assert!(borders(d.path(), System::Sfc).is_empty(), "a card with none");
        for n in 1..=4 {
            put(&d, &format!("sfc{n}.png"));
        }
        assert_eq!(count(d.path(), System::Sfc), 4);
        // And the fifth, from a card edited on a PC — nothing here had to change for it.
        put(&d, "sfc5.png");
        assert_eq!(count(d.path(), System::Sfc), 5);
    }

    #[test]
    fn a_border_belongs_to_the_machine_in_its_name() {
        let d = card();
        put(&d, "sfc1.png");
        put(&d, "sfc2.png");
        put(&d, "gba1.png");
        put(&d, "fc1.png");
        assert_eq!(count(d.path(), System::Sfc), 2);
        assert_eq!(count(d.path(), System::Gba), 1);
        assert_eq!(count(d.path(), System::Gb), 0, "no gb borders on this card");
    }

    /// Numbered, not read off the directory: a card's own order is whoever copied them there, and
    /// `10` sorting before `2` would put the picker's ring in an order nobody wrote down.
    #[test]
    fn they_come_back_in_the_order_they_are_numbered() {
        let d = card();
        for n in [1, 2, 10, 3] {
            put(&d, &format!("gba{n}.png"));
        }
        let got: Vec<u32> = borders(d.path(), System::Gba).iter().map(|b| b.number).collect();
        assert_eq!(got, [1, 2, 3, 10]);
    }

    /// A file that is not a border is not a border: another machine's, a stray label, a directory.
    #[test]
    fn anything_that_is_not_numbered_for_the_machine_is_passed_over() {
        let d = card();
        put(&d, "sfc1.png");
        put(&d, "sfc_label.png");
        put(&d, "sfc0.png"); // numbered from one
        put(&d, "gba1.png");
        put(&d, "readme.txt");
        std::fs::create_dir_all(d.path().join(DIR).join("sfc2.png")).unwrap();
        assert_eq!(count(d.path(), System::Sfc), 1);
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

    /// The choice is per cart, written and read back, and a cart put back on its machine's own
    /// shell leaves no line behind it.
    #[test]
    fn a_cart_s_choice_survives_the_file_and_can_be_taken_away_again() {
        let d = card();
        write_selected(d.path(), "Super Mario World", Some(3)).unwrap();
        write_selected(d.path(), "Tetris", Some(1)).unwrap();
        let map = read_selected(d.path());
        assert_eq!(map.get("Super Mario World"), Some(&3));
        assert_eq!(map.get("Tetris"), Some(&1));
        assert_eq!(map.get("Emerald"), None, "a cart nobody chose for is not chosen");
        write_selected(d.path(), "Tetris", None).unwrap();
        assert_eq!(read_selected(d.path()).get("Tetris"), None);
        assert_eq!(
            read_selected(d.path()).get("Super Mario World"),
            Some(&3),
            "and the other cart's choice is still there"
        );
    }

    /// A typo on a card costs that one cart its border, not the whole file: the same bargain the
    /// core's own file makes.
    #[test]
    fn a_malformed_line_costs_one_cart_and_not_the_file() {
        let d = card();
        std::fs::write(
            d.path().join("System/borders.txt"),
            "Super Mario World = 2\nEmerald = two\n# a note\n\nTetris=1\n",
        )
        .unwrap();
        let map = read_selected(d.path());
        assert_eq!(map.get("Super Mario World"), Some(&2));
        assert_eq!(map.get("Tetris"), Some(&1), "spaces round the = are nothing");
        assert_eq!(map.get("Emerald"), None);
    }
}
