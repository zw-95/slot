//! The words unslot says, in the language the player picked.
//!
//! Every line the interface shows has a `MsgKey` here, and one word in each language in
//! `table.rs`. The two tables are one `match`, so the compiler is what says a key is missing
//! its other language.

mod keys;
mod table;

pub use keys::MsgKey;

use std::sync::atomic::{AtomicU8, Ordering};

/// The language the interface speaks.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    /// English, as unslot shipped it.
    #[default]
    En,
    /// 简体中文。
    ZhCn,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::ZhCn];

    /// What `Config/slot.state` calls it, so the choice survives a restart.
    pub fn key(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::ZhCn => "zh-CN",
        }
    }

    pub fn parse(value: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.key() == value)
    }

    /// The other one, which is what the language row steps to.
    pub fn toggle(self) -> Lang {
        match self {
            Lang::En => Lang::ZhCn,
            Lang::ZhCn => Lang::En,
        }
    }
}

static LANG: AtomicU8 = AtomicU8::new(Lang::En as u8);

/// The language every `t()` reads: English until the card's state says otherwise.
pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        x if x == Lang::ZhCn as u8 => Lang::ZhCn,
        _ => Lang::En,
    }
}

/// Switches the whole interface over. Call it when the state is read and when it changes.
pub fn set_lang(to: Lang) {
    LANG.store(to as u8, Ordering::Relaxed);
}

/// The word for `key` in `lang()`. Keys whose two languages are the same, such as a speed of
/// `2×`, are not here: they are written where they are used.
pub fn t(key: MsgKey) -> &'static str {
    table::text(lang(), key)
}

/// `t(key)` with its one `{}` filled in: "{} min ago" as "5 min ago". A translated line cannot
/// be a format string — the compiler insists on a literal it can check — so the placeholder is
/// swapped for the word instead.
pub fn filled(key: MsgKey, arg: impl std::fmt::Display) -> String {
    t(key).replace("{}", &arg.to_string())
}

/// `t(key)` with every `{}` in it filled from `args`, in the order they come. For the lines
/// that say more than one thing at a time, such as "finding art for GBA (3 of 40)".
pub fn saying(key: MsgKey, args: &[&dyn std::fmt::Display]) -> String {
    let mut line = t(key).to_string();
    for arg in args {
        line = line.replacen("{}", &arg.to_string(), 1);
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_a_word_in_every_language() {
        for key in MsgKey::ALL {
            for lang in Lang::ALL {
                set_lang(lang);
                assert!(!t(key).is_empty(), "{key:?} says nothing in {lang:?}");
            }
        }
        set_lang(Lang::En);
    }

    #[test]
    fn the_language_round_trips_through_the_state_file() {
        for lang in Lang::ALL {
            assert_eq!(Lang::parse(lang.key()), Some(lang));
        }
        assert_eq!(Lang::parse("jp"), None);
    }

    #[test]
    fn english_is_what_unslot_shipped() {
        set_lang(Lang::En);
        assert_eq!(t(MsgKey::RowAbout), "About");
        set_lang(Lang::ZhCn);
        assert_eq!(t(MsgKey::RowAbout), "关于");
        set_lang(Lang::En);
    }
}
