//! What each game wears, when it has chosen for itself.
//!
//! A machine wears one look and every game on it wears it too, until a game picks its own. **A game
//! that picks what its machine already wears is not written down**: such a game goes on following
//! the machine, and follows it again when the machine is changed later. Only a game that differs
//! is an entry, which is why clearing an entry is how a game is given back to its machine.

use std::path::Path;

use slot_store::ini;

/// What each game is wearing — only the games that chose for themselves are in it.
pub const PALETTE_FILE: &str = "Config/palette.txt";

/// The colour one game chose, being a number into its machine's list.
pub fn game_palette(root: &Path, stem: &str) -> Option<u16> {
    ini::value(root, PALETTE_FILE, stem)?.parse().ok()
}

/// What this game wears: what it says for itself, else what its machine says.
pub fn palette_for(root: &Path, machine: u16, stem: &str) -> u16 {
    game_palette(root, stem).unwrap_or(machine)
}

/// Writes what one game wears. Callers ask for this only when it differs from the machine.
pub fn write_palette(root: &Path, stem: &str, at: u16) -> std::io::Result<()> {
    ini::write(root, PALETTE_FILE, stem, &at.to_string())
}

/// Forgets what one game chose, handing it back to its machine.
pub fn clear_palette(root: &Path, stem: &str) -> std::io::Result<()> {
    ini::remove(root, PALETTE_FILE, stem)
}
