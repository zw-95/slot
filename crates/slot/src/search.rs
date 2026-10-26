//! Finding a game on the card by name, from the shelf's X.

use slot_i18n::Lang;

/// The most games a search lists.
pub const MOST: usize = 300;

/// Lower-case words of letters and digits, the rest dropped: `Pokémon: Emerald` is
/// `pokémon emerald`.
fn words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// What a name sounds like, in lower-case letters, for a query typed on an A-Z keyboard:
/// `腾讯翻译` is `txfy`. Empty in English, whose names are already in letters.
fn sounds_like(name: &str, lang: Lang) -> String {
    match lang {
        Lang::ZhCn => slot_pinyin::initials(name).to_lowercase(),
        Lang::En => String::new(),
    }
}

/// How well a name answers a query, lower being better, or None when it does not: every word
/// typed has to start a word of the name or, in Chinese, turn up in the letters the name sounds
/// like. A name the query starts comes first, then one with a word that does, then the rest, and
/// last the ones the letters only reach through the pinyin.
pub fn rank(name: &str, query: &str, lang: Lang) -> Option<u8> {
    let query = words(query);
    if query.is_empty() {
        return None;
    }
    let sounds = sounds_like(name, lang);
    let name = words(name);
    let mut by_ear = false;
    for q in &query {
        if name.iter().any(|w| w.starts_with(q.as_str())) {
            continue;
        }
        if sounds.contains(q.as_str()) {
            by_ear = true;
        } else {
            return None;
        }
    }
    if by_ear {
        return Some(3);
    }
    let joined = name.join(" ");
    Some(match () {
        _ if joined.starts_with(&query.join(" ")) => 0,
        _ if name
            .first()
            .is_some_and(|w| w.starts_with(query[0].as_str())) =>
        {
            1
        }
        _ => 2,
    })
}

/// The indices of the `names` that answer `query`, best first and otherwise in the order given.
pub fn matches<'a>(names: impl Iterator<Item = &'a str>, query: &str, lang: Lang) -> Vec<usize> {
    let mut found: Vec<(u8, usize)> = names
        .enumerate()
        .filter_map(|(i, name)| rank(name, query, lang).map(|r| (r, i)))
        .collect();
    found.sort();
    found.into_iter().take(MOST).map(|(_, i)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_typed_has_to_start_a_word_of_the_name() {
        assert_eq!(rank("The Legend of Zelda", "zel", Lang::En), Some(2));
        assert_eq!(rank("The Legend of Zelda", "leg zel", Lang::En), Some(2));
        assert_eq!(rank("The Legend of Zelda", "elda", Lang::En), None);
        assert_eq!(rank("Metal Slug", "  ", Lang::En), None);
    }

    #[test]
    fn case_and_punctuation_do_not_matter() {
        assert_eq!(
            rank(
                "Street Fighter II': Champion Edition",
                "street fighter ii",
                Lang::En
            ),
            Some(0)
        );
        assert_eq!(rank("Pokémon Emerald", "POKÉMON", Lang::En), Some(0));
    }

    #[test]
    fn a_name_that_starts_with_the_query_comes_first() {
        let names = ["The Metal Slug Story", "Mega Man", "Metal Slug", "Metroid"];
        assert_eq!(matches(names.iter().copied(), "met", Lang::En), [2, 3, 0]);
    }

    #[test]
    fn in_chinese_a_name_is_found_by_the_letters_it_sounds_like() {
        // 腾讯翻译 reads TXFY: any run of those letters finds it, not just the ones at the front.
        assert_eq!(rank("腾讯翻译", "t", Lang::ZhCn), Some(3));
        assert_eq!(rank("腾讯翻译", "tx", Lang::ZhCn), Some(3));
        assert_eq!(rank("腾讯翻译", "fy", Lang::ZhCn), Some(3));
        assert_eq!(rank("腾讯翻译", "txfy", Lang::ZhCn), Some(3));
        assert_eq!(rank("腾讯翻译", "xf", Lang::ZhCn), Some(3));
        assert_eq!(rank("腾讯翻译", "txfz", Lang::ZhCn), None);
        assert_eq!(rank("洛克人Zero", "lkr", Lang::ZhCn), Some(3));
        assert_eq!(
            rank("洛克人Zero", "zero", Lang::ZhCn),
            Some(3),
            "the Latin in a Chinese name is found the same way"
        );
        assert_eq!(rank("洛克人Zero", "lkrzero", Lang::ZhCn), Some(3));
    }

    #[test]
    fn the_letters_only_reach_a_chinese_name_in_chinese() {
        assert_eq!(rank("腾讯翻译", "tx", Lang::En), None);
    }

    #[test]
    fn a_name_the_letters_reach_comes_before_one_only_the_pinyin_reaches() {
        let names = ["腾讯翻译", "Tetris", "Tencent 腾讯"];
        assert_eq!(matches(names.iter().copied(), "t", Lang::ZhCn), [1, 2, 0]);
    }
}
