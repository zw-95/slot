//! Pinyin initials, for finding a Chinese game by the letters its name sounds like.
//!
//! The search on the shelf is typed on an A-Z keyboard, and a Chinese name has none of those
//! letters in it, so a Chinese library is searched by what its titles sound like: every hanzi
//! becomes the first letter of its reading, so 腾讯翻译 is `TXFY` and typing `t`, `tx` or `fy`
//! finds it.
//!
//! Only the initials, never whole readings: a player types a handful of letters and expects an
//! answer while they type, and a full syllable would make the match too narrow to bother with.

mod table;

pub use table::initial;

/// What a name sounds like, in the letters a player would type: every hanzi is the first letter of
/// its reading, every other letter or digit stays as it is, upper-cased, and the rest — spaces,
/// punctuation — is dropped. `腾讯翻译` is `TXFY`; `洛克人Zero` is `LKRZERO`.
///
/// A hanzi the table does not know is dropped rather than guessed at.
pub fn initials(name: &str) -> String {
    name.chars()
        .filter_map(|c| match initial(c) {
            Some(letter) => Some(letter),
            None => c.is_ascii_alphanumeric().then(|| c.to_ascii_uppercase()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::initials;

    #[test]
    fn a_name_becomes_the_letters_its_hanzi_start_with() {
        assert_eq!(initials("腾讯翻译"), "TXFY");
        assert_eq!(initials("宝可梦-红宝石"), "BKMHBS");
        assert_eq!(initials("洛克人Zero"), "LKRZERO");
        assert_eq!(initials("恶魔城晓月MOD版"), "EMCXYMODB");
    }

    #[test]
    fn a_name_in_letters_is_itself_without_its_punctuation() {
        assert_eq!(initials("Metal Slug"), "METALSLUG");
        assert_eq!(initials("1080 Snowboarding"), "1080SNOWBOARDING");
        assert_eq!(initials(""), "");
    }

    #[test]
    fn a_hanzi_the_table_does_not_know_is_dropped() {
        // U+9FFF is in no font and in no dictionary; the rest of the name still reads.
        assert_eq!(initials("腾讯翻译"), initials("腾讯翻译\u{9fff}"));
    }
}
