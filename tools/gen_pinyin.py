#!/usr/bin/env python3
"""Generate the pinyin initial table: `crates/slot-pinyin/src/table.rs`.

The shelf's search is typed on an A-Z keyboard, and a Chinese name has none of those letters in
it, so a Chinese library is searched by what its titles sound like: every hanzi becomes the first
letter of its reading, so 腾讯翻译 is `TXFY` and typing `t`, `tx` or `fy` finds it. The table
carries no metadata: every hanzi in GB2312 plus the ones this card's titles actually use, each
with its pinyin initial, sorted by code point — so a lookup at runtime is one binary search.

    python tools/gen_pinyin.py

Needs `pypinyin` (MIT, https://github.com/mozillazg/python-pinyin), which is where the readings
come from. To add characters, edit this and run it again rather than hand-editing `table.rs`: the
`LETTERS` byte string and the `HANZI` array have to correspond one for one, and a mismatch does
not fail loudly — it quietly looks up the wrong letter (see below).

**Run the unit test**: `cargo test -p slot-pinyin`. `the_table_is_sorted` is not optional: a
binary search over an unsorted table does not error, it just returns the wrong value, so the
table's sortedness is the only thing standing between this and that bug. It was hit on the first
generation — inside a Rust multi-line byte string the newline and the indentation after it are
literal characters, and the first line's four-space indent was not absorbed by the continuation
either, so the whole table landed 465 + 4 bytes out and '宝' looked up as 'Z'. Hence the `\\`
continuations below, and the first line deliberately unindented.
"""

import os
from collections import Counter

from pypinyin import Style, lazy_pinyin

# The script lives in the tree it generates into, so the tree is one level up. The card's own
# titles are glanced at only to catch a character that falls outside GB2312; SLOT_CARD points at
# a card tree (its `Labels/` and `Games/`).
TREE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(TREE, "crates", "slot-pinyin", "src", "table.rs")
CARD = os.environ.get("SLOT_CARD", "")


def hanzi() -> list[tuple[str, str]]:
    """(hanzi, initial), sorted by code point."""
    chars = set()
    # The whole GB2312 hanzi range.
    for hi in range(0xB0, 0xF8):
        for lo in range(0xA1, 0xFF):
            try:
                ch = bytes([hi, lo]).decode("gb2312")
            except Exception:
                continue
            if len(ch) == 1:
                chars.add(ch)
    # Plus every character this card's titles actually use, in case one falls outside GB2312.
    for sub in ("Labels", "Games"):
        d = os.path.join(CARD, sub)
        if not os.path.isdir(d):
            continue
        for _, _, files in os.walk(d):
            for f in files:
                for ch in os.path.splitext(f)[0]:
                    if ord(ch) > 0x2E80:
                        chars.add(ch)

    table = {}
    for c in chars:
        p = lazy_pinyin(c, style=Style.FIRST_LETTER)
        if p and p[0] and p[0].isalpha() and p[0].isascii() and ord(c) > 0x2E80:
            table[c] = p[0].upper()
    return sorted(table.items(), key=lambda kv: ord(kv[0]))


def main():
    items = hanzi()
    if not items:
        raise SystemExit("collected no hanzi at all; check SLOT_CARD and pypinyin")
    dist = Counter(v for _, v in items)
    print("covering %d hanzi; letter distribution %s" % (len(items), dict(sorted(dist.items()))))

    out = []
    out.append('''//! Pinyin initials, one letter for every hanzi, so a name can be found by what it sounds like.
//!
//! GENERATED, and not by hand: the table below is every hanzi in GB2312 plus the ones this
//! card's titles actually use, each with the first letter of its pinyin, sorted by code point so
//! a lookup is a binary search. Regenerate rather than edit: `tools/gen_pinyin.py`, which reads
//! the readings out of pypinyin (MIT).

/// Every hanzi the table knows, sorted by code point.
const HANZI: &[char] = &[
''')
    line = []
    for c, _ in items:
        line.append("'%s'," % c)
        if len(line) == 24:
            out.append("    " + "".join(line) + "\n")
            line = []
    if line:
        out.append("    " + "".join(line) + "\n")
    out.append("];\n\n")

    # The byte string uses `\` continuations: the newline and the indent that follows it are
    # eaten, so only letters reach the data. The first line must NOT be indented — a continuation
    # eats "newline + next line's indent" and never the first character of a line.
    letters = "".join(v for _, v in items)
    out.append("/// The initial of `HANZI[i]`, as a byte, in the same order.\nconst LETTERS: &[u8] = b\"")
    for i in range(0, len(letters), 64):
        chunk = letters[i : i + 64]
        out.append(chunk + ("\\\n    " if i + 64 < len(letters) else ""))
    out.append('";\n\n')

    out.append('''/// The first letter of the reading of `c`, upper-case: `T` for 腾.
///
/// `None` for anything else — a letter, a digit, kana, and any hanzi the table does not know,
/// which a caller drops rather than guesses at.
pub fn initial(c: char) -> Option<char> {
    if c.is_ascii() {
        return None;
    }
    HANZI.binary_search(&c).ok().map(|i| LETTERS[i] as char)
}

#[cfg(test)]
mod tests {
    use super::{initial, HANZI, LETTERS};

    #[test]
    fn reads_a_hanzi_as_the_letter_it_starts_with() {
        assert_eq!(initial('腾'), Some('T'));
        assert_eq!(initial('讯'), Some('X'));
        assert_eq!(initial('翻'), Some('F'));
        assert_eq!(initial('译'), Some('Y'));
        assert_eq!(initial('宝'), Some('B'));
    }

    #[test]
    fn leaves_everything_that_is_not_a_hanzi_alone() {
        assert_eq!(initial('A'), None);
        assert_eq!(initial('7'), None);
        assert_eq!(initial(' '), None);
    }

    #[test]
    fn the_table_is_sorted() {
        // A binary search over an unsorted table silently misses, which would read the wrong
        // letter with no other symptom.
        for w in HANZI.windows(2) {
            assert!(w[0] < w[1], "table out of order at {:?}", w[0]);
        }
        assert_eq!(LETTERS.len(), HANZI.len());
    }
}
''')

    text = "".join(out)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("wrote %s (%.1f KB)" % (os.path.relpath(OUT, TREE), os.path.getsize(OUT) / 1024))
    print("now run:  cargo test -p slot-pinyin")


if __name__ == "__main__":
    main()
