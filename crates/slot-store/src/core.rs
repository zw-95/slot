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
    /// Gambatte. muOS's own default for the Game Boy and Game Boy Color, and therefore this
    /// one's: an accurate core that has been the answer for those two machines for years.
    Gambatte,
    /// SameBoy, the one that emulates the boot ROMs and gets the CGB's colour right.
    SameBoy,
    /// Gearboy: small, and the least accurate of the Game Boy cores here.
    Gearboy,
    /// VBA-M, the old VisualBoyAdvance. Runs the Game Boy carts as well as the Advance ones,
    /// which is why it appears on three machines' lists and not one.
    Vbam,
    /// `vba_next`, VBA-M's lighter fork.
    VbaNext,
    /// Mednafen's Game Boy Advance core.
    MednafenGba,
    /// mGBA built with the rumble patches. Same emulator, with a motor in it.
    MgbaRumble,
    /// Mednafen's Super Nintendo core, the `mednafen_supafaust` libretro build.
    Supafaust,
    /// Snes9x's 2005 branch, the one that grew accuracy patches. Cheap enough for this
    /// handheld and more compatible than the 2002 build it came from.
    Snes9x2005Plus,
    /// Snes9x's 2010 branch. More accurate than 2005+, and the most expensive of the three.
    Snes9x2010,
    /// The 2005 branch without the patches. Offered because it is on the card, not because it
    /// is better than `Plus` — it is the same core a year earlier.
    Snes9x2005,
    /// The 2002 branch: the cheapest Snes9x there is, and the least compatible. Kept for the
    /// handful of games that ran on it and on nothing else.
    Snes9x2002,
    /// Snes9x current, the branch the project still maintains.
    Snes9x,
    /// `snes9x_next`, Snes9x's accuracy fork.
    Snes9xNext,
    /// Mesen's Super Nintendo core. The most accurate thing here and the second most expensive.
    MesenS,
    /// bsnes, the original. Cycle accurate and the heaviest core on the card by a long way.
    Bsnes,
    /// The 2014 rewrite, in its performance profile: much of bsnes's accuracy at a price this
    /// handheld can sometimes pay.
    Bsnes2014Performance,
    /// The Mercury accuracy profile of the same 2014 core.
    BsnesMercuryPerformance,
    /// The 2014 core as it was before it was rewritten in C++11.
    BsnesCplusplus98,
    /// Snes9x's very old Chimera branch. On the card, and the least of these by some distance,
    /// but it costs nothing to leave it in the list.
    ChimeraSnes,
    /// FCEUmm, the Family Computer core: muOS's own default for the machine — `default =
    /// fceumm` in that archive's global.ini — and the core that runs essentially all of the
    /// card's largest library. First on its machine's list for that reason, and not alone on
    /// it: one entry is not a choice, and the machine's board had no picker to draw.
    Fceumm,
    /// Nestopia, the accurate one: cycle-checked where FCEUmm is compatible, and the core to
    /// reach for when a cart behaves wrongly rather than slowly. Five megabytes against
    /// FCEUmm's four, which is the price of the accuracy and the reason it is second.
    Nestopia,
    /// QuickNES, the cheap one: a megabyte and a third, and the fastest of the three on a
    /// machine this size. It gives up accuracy at the edges — a handful of mappers and some
    /// of the audio — which is a fair trade on a handheld and a bad one on a desktop, so it
    /// sits last rather than first: the cores are offered best-first, and a player who wants
    /// the small fast one should have to walk past the two that are better.
    QuickNes,
}

impl Core {
    /// Every variant, once. The single source of truth for "is this name a core directory" —
    /// `migrate_states` walks this rather than spelling the variant list out a second time,
    /// so a third core added here does not also have to be remembered at every call site
    /// that needs to tell a core's own directory apart from a cart's.
    pub const ALL: [Core; 25] = [
        Core::Mgba,
        Core::Gpsp,
        Core::Gambatte,
        Core::SameBoy,
        Core::Gearboy,
        Core::Vbam,
        Core::VbaNext,
        Core::MednafenGba,
        Core::MgbaRumble,
        Core::Supafaust,
        Core::Snes9x2005Plus,
        Core::Snes9x2010,
        Core::Snes9x2005,
        Core::Snes9x2002,
        Core::Snes9x,
        Core::Snes9xNext,
        Core::MesenS,
        Core::Bsnes,
        Core::Bsnes2014Performance,
        Core::BsnesMercuryPerformance,
        Core::BsnesCplusplus98,
        Core::ChimeraSnes,
        Core::Fceumm,
        Core::Nestopia,
        Core::QuickNes,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Core::Mgba => "mgba",
            Core::Gpsp => "gpsp",
            Core::Gambatte => "gambatte",
            Core::SameBoy => "sameboy",
            Core::Gearboy => "gearboy",
            Core::Vbam => "vbam",
            Core::VbaNext => "vba_next",
            Core::MednafenGba => "mednafen_gba",
            Core::MgbaRumble => "mgba_rumble",
            // The libretro name, because that is what it is called on every other frontend
            // and what the `.so` on the card is called. `text` is the player's name for it.
            Core::Supafaust => "mednafen_supafaust",
            Core::Snes9x2005Plus => "snes9x2005_plus",
            Core::Snes9x2010 => "snes9x2010",
            Core::Snes9x2005 => "snes9x2005",
            Core::Snes9x2002 => "snes9x2002",
            Core::Snes9x => "snes9x",
            Core::Snes9xNext => "snes9x_next",
            Core::MesenS => "mesen-s",
            Core::Bsnes => "bsnes",
            Core::Bsnes2014Performance => "bsnes2014_performance",
            Core::BsnesMercuryPerformance => "bsnes_mercury_performance",
            Core::BsnesCplusplus98 => "bsnes_cplusplus98",
            Core::ChimeraSnes => "chimerasnes",
            Core::Fceumm => "fceumm",
            Core::Nestopia => "nestopia",
            Core::QuickNes => "quicknes",
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
            Core::Gambatte => "Gambatte",
            Core::SameBoy => "SameBoy",
            Core::Gearboy => "Gearboy",
            Core::Vbam => "VBA-M",
            Core::VbaNext => "VBA Next",
            Core::MednafenGba => "Beetle GBA",
            Core::MgbaRumble => "mGBA Rumble",
            // Not the libretro name: that is `mednafen_supafaust`, which is a name from the
            // build system, and what the picker is putting on a panel is the emulator.
            Core::Supafaust => "Supafaust",
            Core::Snes9x2005Plus => "Snes9x 2005+",
            Core::Snes9x2010 => "Snes9x 2010",
            Core::Snes9x2005 => "Snes9x 2005",
            Core::Snes9x2002 => "Snes9x 2002",
            Core::Snes9x => "Snes9x",
            Core::Snes9xNext => "Snes9x Next",
            Core::MesenS => "Mesen-S",
            Core::Bsnes => "bsnes",
            Core::Bsnes2014Performance => "bsnes 2014",
            Core::BsnesMercuryPerformance => "bsnes Mercury",
            Core::BsnesCplusplus98 => "bsnes C++98",
            Core::ChimeraSnes => "ChimeraSNES",
            Core::Fceumm => "FCEUmm",
            Core::Nestopia => "Nestopia",
            Core::QuickNes => "QuickNES",
        }
    }

    pub fn parse(s: &str) -> Option<Core> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mgba" => Some(Core::Mgba),
            "gpsp" => Some(Core::Gpsp),
            "gambatte" => Some(Core::Gambatte),
            "sameboy" | "same_boy" => Some(Core::SameBoy),
            "gearboy" => Some(Core::Gearboy),
            "vbam" | "vba-m" => Some(Core::Vbam),
            "vba_next" | "vbanext" => Some(Core::VbaNext),
            "mednafen_gba" | "beetle_gba" | "beetle gba" => Some(Core::MednafenGba),
            "mgba_rumble" | "mgbarumble" => Some(Core::MgbaRumble),
            // The libretro name, and the short one a hand is likelier to type.
            "mednafen_supafaust" | "supafaust" => Some(Core::Supafaust),
            "snes9x2005_plus" | "snes9x2005plus" => Some(Core::Snes9x2005Plus),
            // **The one this list was missing**, and the reason a Super Nintendo cart came back
            // on `snes9x` after `snes9x2010` had been chosen and confirmed: `write_selected_core`
            // writes `as_str`, which spells this core `snes9x2010`, and a name this function does
            // not know is a fall back to the machine's default — so the choice was written, shown,
            // and quietly un-chosen on the next read. A name `as_str` can write must be a name
            // `parse` can read, which `every_core_round_trips_through_its_own_name` now holds.
            "snes9x2010" | "snes9x_2010" | "snes9x-2010" => Some(Core::Snes9x2010),
            "snes9x2005" => Some(Core::Snes9x2005),
            "snes9x2002" => Some(Core::Snes9x2002),
            "snes9x" => Some(Core::Snes9x),
            "snes9x_next" | "snes9xnext" => Some(Core::Snes9xNext),
            "mesen-s" | "mesens" | "mesen" => Some(Core::MesenS),
            "bsnes" => Some(Core::Bsnes),
            "bsnes2014_performance" | "bsnes2014" => Some(Core::Bsnes2014Performance),
            "bsnes_mercury_performance" | "bsnes_mercury" => Some(Core::BsnesMercuryPerformance),
            "bsnes_cplusplus98" => Some(Core::BsnesCplusplus98),
            "chimerasnes" | "chimera" => Some(Core::ChimeraSnes),
            "fceumm" | "fceum" => Some(Core::Fceumm),
            "nestopia" => Some(Core::Nestopia),
            "quicknes" | "quick_nes" => Some(Core::QuickNes),
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
            System::Gba => &[
                // muOS's default for the Advance, and the one core here that has run on this
                // device for as long as the device has run anything.
                Core::Mgba,
                // Second, and the only other core on this list that earns its place rather than
                // merely being available: gpSP carries the serial hardware mGBA's libretro build
                // does not, so two Advance carts can only talk to each other through this one.
                // Picking anything else and then starting a link is a link that never opens.
                Core::Gpsp,
                Core::Vbam,
                Core::VbaNext,
                Core::MednafenGba,
                Core::MgbaRumble,
            ],
            // **Two of the five are gone.** `gambatte` and `mgba` were the first two on this
            // list — the first being muOS's own default for both machines — and both are wrong
            // on a Game Boy cart on this device: the game comes up and comes up *mis*drawn,
            // which is a worse failure than a cart that refuses to start, because it reads as
            // the cart being broken. They are out rather than moved down: a core that draws a
            // Game Boy game wrongly is not a fallback for anything.
            //
            // What is left is three, and SameBoy is first: accurate, and the one of the three
            // that has never been reported wrong here. Then Gearboy, then VBA-M — which also
            // runs the Advance, where it stays where it is, this list being a Game Boy's.
            System::Gb | System::Gbc => &[Core::SameBoy, Core::Gearboy, Core::Vbam],
            // Thirteen, which is every Super Nintendo core muOS ships, and in the order a
            // player should meet them rather than the order the archives list them: the one
            // that has been tested first, then the accurate-but-cheap Snes9x branches, then the
            // ones that want more machine than this handheld has, and last the two that are
            // only here because they came off the card with the rest.
            //
            // The board shows three at a time and rolls the rest along under the chip, so a
            // long list costs nothing but travelling: what it does cost is that a player has to
            // walk to find out what is on it.
            System::Sfc => &[
                // muOS's own default for the machine, and so this one's: that archive's
                // `assign/Nintendo SNES - SFC/global.ini` reads `default = snes9x`. A machine's
                // default taken from the same place its cores came from is a default somebody
                // picked for these cores on this class of handheld, rather than one picked
                // here for no better reason than that it was first.
                Core::Snes9x,
                // Which was the default before, and is the only core of the thirteen that has
                // actually been run on the device. Second rather than dropped: it is the
                // answer if `snes9x` turns out not to be.
                Core::Supafaust,
                Core::Snes9x2010,
                Core::Snes9x2005Plus,
                Core::Snes9xNext,
                Core::Snes9x2002,
                Core::MesenS,
                Core::Bsnes2014Performance,
                Core::BsnesMercuryPerformance,
                Core::Bsnes,
                Core::BsnesCplusplus98,
                Core::ChimeraSnes,
                Core::Snes9x2005,
            ],
            // Three, which is every Family Computer core muOS ships that a handheld this size
            // has any use for: `mesen` (3.1 M) and `bnes` (0.25 M) are on the card too, and
            // both are out — the first because its name is the Super Nintendo's `mesen-s`
            // with the suffix dropped, so a line naming it would be read back as that core,
            // and the second because it is higan's, which wants more machine than this is.
            //
            // FCEUmm first, as muOS has it: compatible with essentially the whole library,
            // and the machine's is the card's largest. Then the accurate one, then the cheap
            // one — best first, so that the two a player should not have to reach past are
            // the two they meet first.
            System::Fc => &[Core::Fceumm, Core::Nestopia, Core::QuickNes],
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
