#!/usr/bin/env python3
"""Turns a Pegasus `metadata.pegasus.txt` into slot's `set<TAB>name` table.

Pegasus pairs a shown name with the file it belongs to:

    game: 1941-反击战
    file: 1941.zip

slot asks for a name by the set — the zip's name without its extension — so that is what the
table is keyed on, lower-cased to match `arcade::name`, which lower-cases what it looks up.

    python3 tools/arcade/names_zh.py <metadata.pegasus.txt> <arcade_names_zh.tsv>
"""

import os
import sys

DEFAULT_SRC = r"D:\安装包\Game\ROM-天马\ROMS\FBNEO\metadata.pegasus.txt"
DEFAULT_DST = r"crates\slot-store\assets\arcade_names_zh.tsv"


def read_pairs(text):
    """Every (set, name) Pegasus gave a file for, in the order they appear."""
    pairs = []
    name = None
    for line in text.splitlines():
        line = line.rstrip()
        if line.startswith("game:"):
            name = line[len("game:") :].strip()
        elif line.startswith("file:") and name:
            base = os.path.basename(line[len("file:") :].strip())
            pairs.append((os.path.splitext(base)[0].lower(), name))
            name = None
    return pairs


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_SRC
    dst = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_DST

    with open(src, encoding="utf-8") as f:
        pairs = read_pairs(f.read())

    # Sorted and de-duplicated: the table is read into a map, and a set named twice would
    # otherwise be whichever came last.
    rows = sorted(set(pairs))
    with open(dst, "w", encoding="utf-8", newline="\n") as f:
        for set_name, name in rows:
            f.write(f"{set_name}\t{name}\n")

    print(f"wrote {len(rows)} names to {dst}")


if __name__ == "__main__":
    main()
