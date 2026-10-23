use std::collections::HashMap;
use std::path::Path;

use crate::System;

pub const SELECTED_CORE_FILE: &str = "System/selected_core.ini";

/// Which emulator runs a cart. mGBA is the whole product's default; gpSP exists for the
/// serial hardware mGBA's libretro build does not carry; the other three are Super Nintendo
/// cores, which is the only machine so far that has more than one worth offering.
///
/// A core is not a machine, and the two are deliberately not the same list: two of these run
/// Game Boy Advance carts, three run Super Nintendo ones, and which one a cart gets is
/// written on the card per cart. The machine says what a cart *is*; the core says what runs
/// it, and `System::cores` is what says which of these a machine may be asked about.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Core {
    #[default]
    Mgba,
    Gpsp,
    /// Mednafen's Super Nintendo core, the `mednafen_supafaust` libretro build.
    Supafaust,
    /// Snes9x's 2005 branch, the one that grew accuracy patches. Cheap enough for this
    /// handheld and more compatible than the 2002 build it came from.
    Snes9x2005Plus,
    /// Snes9x's 2010 branch. More accurate than 2005+, and the most expensive of the three.
    Snes9x2010,
}

impl Core {
    /// Every variant, once. The single source of truth for "is this name a core directory" —
    /// `migrate_states` walks this rather than spelling the variant list out a second time,
    /// so a third core added here does not also have to be remembered at every call site
    /// that needs to tell a core's own directory apart from a cart's.
    pub const ALL: [Core; 5] = [
        Core::Mgba,
        Core::Gpsp,
        Core::Supafaust,
        Core::Snes9x2005Plus,
        Core::Snes9x2010,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Core::Mgba => "mgba",
            Core::Gpsp => "gpsp",
            // The libretro name, because that is what it is called on every other frontend
            // and what the `.so` on the card is called. `text` is the player's name for it.
            Core::Supafaust => "mednafen_supafaust",
            Core::Snes9x2005Plus => "snes9x2005_plus",
            Core::Snes9x2010 => "snes9x2010",
        }
    }

    /// Position in `ALL`, which is the order the picker's rows and their faces are in.
    pub fn index(self) -> usize {
        self as usize
    }

    /// What the picker calls it. Not `as_str`: that is the ini's spelling, meant to be typed
    /// by hand into a text editor on a computer, and this is the player's, meant to be read
    /// off a panel. The two are free to differ, and already do.
    pub fn text(self) -> &'static str {
        match self {
            Core::Mgba => "mGBA",
            Core::Gpsp => "gpSP",
            // Not the libretro name: that is `mednafen_supafaust`, which is a name from the
            // build system, and what the picker is putting on a panel is the emulator.
            Core::Supafaust => "Supafaust",
            Core::Snes9x2005Plus => "Snes9x 2005+",
            Core::Snes9x2010 => "Snes9x 2010",
        }
    }

    pub fn parse(s: &str) -> Option<Core> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mgba" => Some(Core::Mgba),
            "gpsp" => Some(Core::Gpsp),
            // The libretro name, and the short one a hand is likelier to type.
            "mednafen_supafaust" | "supafaust" => Some(Core::Supafaust),
            "snes9x2005_plus" | "snes9x2005plus" | "snes9x2005" => Some(Core::Snes9x2005Plus),
            "snes9x2010" | "snes9x2010_plus" => Some(Core::Snes9x2010),
            _ => None,
        }
    }
}

impl System {
    /// The cores a machine can be run by, in the order the picker lays them out: left to right
    /// across the board. The first is what a cart gets when the card says nothing about it,
    /// so it is the one that has to be right for the most carts.
    ///
    /// One entry is not a choice, and a machine with one does not get a picker at all: a board
    /// with a single socket on it is a question that can only be refused.
    pub fn cores(self) -> &'static [Core] {
        match self {
            System::Gba => &[Core::Mgba, Core::Gpsp],
            System::Gb | System::Gbc => &[Core::Mgba],
            System::Sfc => &[Core::Supafaust, Core::Snes9x2005Plus, Core::Snes9x2010],
        }
    }

    /// What runs a cart of this machine when the card says nothing about it.
    pub fn default_core(self) -> Core {
        // There is always at least one: a machine with none would have no way to run anything.
        self.cores()[0]
    }
}

/// The line `write_selected_core` writes for a cart, and the one `core_for` looks for first.
///
/// Carries the machine, because a stem is not a cart: `Contra` is a Super Nintendo game and
/// a Mega Drive one, and a card holding both would otherwise have one line for two carts.
///
/// A bare stem is still understood, and still means the Advance — which is what every line on
/// every card says today, and the only machine that has ever had a choice was that one. So a
/// card written by any build before this one still reads exactly as it did.
fn key(system: System, stem: &str) -> String {
    format!("{}/{}", system.dir_name(), stem)
}

/// `<rom stem> = <core>`, one per line. Keyed on the stem because that is already the key
/// for `Labels/`, `Saves/` and `States/`; a card stays consistent with itself.
///
/// Every malformed line is skipped rather than raised. This file is edited by hand on a
/// card, and the cost of a typo must be that one cart opens with the default core, never
/// that the shelf fails to load.
pub fn read_selected_cores(root: &Path) -> HashMap<String, Core> {
    let mut out = HashMap::new();
    let Ok(text) = std::fs::read_to_string(root.join(SELECTED_CORE_FILE)) else {
        return out;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(';')
            || line.starts_with('[')
        {
            continue;
        }
        let Some((stem, core)) = line.split_once('=') else {
            continue;
        };
        let stem = stem.trim();
        if stem.is_empty() {
            continue;
        }
        // A name we do not know is not a failure: it is a card written for a newer build,
        // or a typo. Either way the default is the safe reading.
        if let Some(core) = Core::parse(core) {
            out.insert(stem.to_string(), core);
        }
    }
    out
}

/// The core one cart wants. Reads the file each time: it is a few lines on a card that a
/// person edits between boots, and caching it would only create a staleness question
/// nobody asked for.
///
/// `system` is what makes the answer a legal one. Two things a hand can write into the file
/// have to be read as "nothing chosen" rather than as a request: a core this machine does not
/// run (`Contra (J).sfc = mgba` is a typo, not a wish to hand a Super Nintendo cart to a Game
/// Boy Advance emulator), and a bare stem, which is what every line said before the machine
/// became part of the key and so can only ever mean the Advance.
pub fn core_for(root: &Path, system: System, stem: &str) -> Core {
    let map = read_selected_cores(root);
    let named = map
        .get(&key(system, stem))
        .or_else(|| (system == System::Gba).then(|| map.get(stem)).flatten());
    match named.copied() {
        Some(core) if system.cores().contains(&core) => core,
        _ => system.default_core(),
    }
}

/// Set one cart's core, leaving the rest of the file exactly as it was.
///
/// The line is replaced in place, or appended when the cart has no line yet. The file is
/// never rebuilt from `read_selected_cores`' map: it is meant to be opened in a text editor
/// on a computer, and a rebuild would quietly drop every comment, blank line and unparsed
/// line in it — including the note somebody wrote to themselves above a cart.
pub fn write_selected_core(
    root: &Path,
    system: System,
    stem: &str,
    core: Core,
) -> std::io::Result<()> {
    let path = root.join(SELECTED_CORE_FILE);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    let entry_key = key(system, stem);
    let entry = format!("{entry_key} = {}", core.as_str());
    let mut out = String::with_capacity(existing.len() + entry.len() + 1);
    let mut replaced = false;

    for line in existing.lines() {
        // The old spelling counts as this cart's line too, so a cart written before the key
        // carried a machine is rewritten in place rather than left behind to contradict the
        // new one — two lines for one cart, of which only one would ever be read.
        let is_this_cart = line
            .split_once('=')
            .map(|(k, _)| {
                let k = k.trim();
                k == entry_key || (system == System::Gba && k == stem)
            })
            .unwrap_or(false);
        if is_this_cart && !replaced {
            out.push_str(&entry);
            replaced = true;
        } else if is_this_cart {
            // A duplicate for the same cart: the later line already won when read, so
            // dropping it keeps the file saying one thing per cart.
            continue;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !replaced {
        out.push_str(&entry);
        out.push('\n');
    }

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    crate::atomic::atomic_write(&path, out.as_bytes())
}
