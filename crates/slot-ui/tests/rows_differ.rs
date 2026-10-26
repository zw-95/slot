//! The menu's rows are deduped into one baked label each: rows that read alike share a texture
//! (the per-system "Screen" rows do, in English). That is only safe while the dedupe comes out
//! the same size in *every* tongue — a pair that reads alike in one and not in the other gives
//! the batch a different shape in each, and a batch is baked into the textures the one before
//! it was baked into. So every tongue has to boil down to the same number of labels.
//!
//! It gets a file of its own because the language is a global: a test that moved it would be
//! moving it under every other test running at the same time.

use slot_i18n::{set_lang, Lang};
use slot_ui::QuickRow;
use std::collections::{HashMap, HashSet};

/// What a tongue calls the menu's rows: how many wordings it needs, and which ones it shares.
fn deduped(lang: Lang) -> (usize, Vec<&'static str>) {
    set_lang(lang);
    let mut said: HashMap<&'static str, usize> = HashMap::new();
    for row in QuickRow::ALL {
        *said.entry(row.label()).or_default() += 1;
    }
    let shared = said
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|(word, _)| *word)
        .collect();
    (said.len(), shared)
}

#[test]
fn the_menu_bakes_as_many_labels_in_one_tongue_as_in_the_other() {
    let said: Vec<_> = Lang::ALL.iter().map(|&lang| (lang, deduped(lang))).collect();
    set_lang(Lang::En);
    let counts: Vec<_> = said.iter().map(|(lang, (n, _))| (*lang, *n)).collect();
    assert!(
        counts.windows(2).all(|w| w[0].1 == w[1].1),
        "the batch changes shape with the tongue: {counts:?}, shared {shared:?}",
        shared = said
            .iter()
            .map(|(lang, (_, shared))| (*lang, shared.clone()))
            .collect::<Vec<_>>()
    );
}
