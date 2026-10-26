//! Arcade games' names. An arcade game is a zip named for its set, `sf2ce.zip`, and the shelf
//! shows the name the arcade databases give it instead: Street Fighter II': Champion Edition.

use std::collections::HashMap;
use std::sync::OnceLock;

/// `set<TAB>name` a line, written by tools/arcade/names.py.
const NAMES: &str = include_str!("../assets/arcade_names.tsv");

/// The same table in Chinese, written by tools/arcade/names_zh.py off a Pegasus front end's
/// `metadata.pegasus.txt`. Asked first: where a set has a Chinese name, that is the one the
/// shelf shows, whichever tongue the interface is in.
const NAMES_ZH: &str = include_str!("../assets/arcade_names_zh.tsv");

/// The sets that are a board's BIOS or a shared device, not a game, which sit beside the games
/// for the cores to find but have no place on the shelf.
const NOT_GAMES: &[&str] = &[
    "neogeo",
    "neocdz",
    "pgm",
    "skns",
    "nmk004",
    "isgsm",
    "decocass",
    "cchip",
    "bubsys",
    "qsound",
    "qsound_hle",
    "ym2608",
    "stvbios",
    "hng64",
    "konamigx",
    "midssio",
    "namco51",
    "namco52",
    "namco53",
    "namco54",
    "namco62",
    "sp-1",
    "coh1000c",
    "coh1000a",
    "coh3002c",
    "cpzn1",
    "cpzn2",
    "tps",
    "taitofx1",
    "naomi",
    "awbios",
    "megaplay",
    "megatech",
    "playch10",
    "nss",
    "sys573",
    "f355bios",
    "hod2bios",
    "airlbios",
    "crysbios",
    "ar_bios",
    "galgbios",
    "kf2_bios",
];

fn names() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    MAP.get_or_init(|| NAMES.lines().filter_map(|l| l.split_once('\t')).collect())
}

fn names_zh() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    MAP.get_or_init(|| NAMES_ZH.lines().filter_map(|l| l.split_once('\t')).collect())
}

/// The game a set is, by the zip's name without `.zip`, or None for a set nobody knows.
/// Chinese first, so a set that has a Chinese name is called by it; the English table is what
/// answers for every set that does not.
pub fn name(set: &str) -> Option<&'static str> {
    let set = set.to_ascii_lowercase();
    names_zh()
        .get(set.as_str())
        .or_else(|| names().get(set.as_str()))
        .copied()
}

/// Whether a set is a BIOS or a device rather than a game.
pub fn is_bios(set: &str) -> bool {
    NOT_GAMES.iter().any(|b| set.eq_ignore_ascii_case(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_is_named_for_its_game_whatever_its_case() {
        // Chinese first: both of these have one, so that is the name the shelf shows.
        assert_eq!(name("SF2CE"), Some("街头霸王II'-四大天王"));
        assert_eq!(name("mslug"), Some("合金弹头"));
        assert_eq!(name("not-a-set"), None);
    }

    #[test]
    fn a_set_with_no_chinese_name_is_still_named_in_english() {
        assert_eq!(name("10yard"), Some("10-Yard Fight (World, set 1)"));
    }

    #[test]
    fn a_bios_is_not_a_game() {
        assert!(is_bios("neogeo") && is_bios("PGM"));
        assert!(!is_bios("mslug"));
    }
}
