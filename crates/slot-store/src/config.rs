use std::io;
use std::path::Path;

use crate::atomic::atomic_write;
use slot_i18n::Lang;

pub const CONFIG_DIR: &str = "Config";

/// The language has a file of its own, as the recents and the favorites do: `slot.state` is
/// written and read field by field, and every test of it names every field.
const LANGUAGE_FILE: &str = "language.txt";

const MOVED: [&str; 4] = [
    "slot.state",
    "selected_core.ini",
    "video_mode.ini",
    "cart_shell.ini",
];

const RENAMED: [(&str, &str); 4] = [
    ("Config/selected_core.ini", crate::SELECTED_CORE_FILE),
    ("Config/video_mode.ini", "Config/video_mode.txt"),
    ("Config/cart_shell.ini", crate::CART_SHELL_FILE),
    ("Labels/cart_shell.ini", crate::LABELS_SHELL_FILE),
];

pub fn move_config(root: &Path) -> io::Result<()> {
    let config = root.join(CONFIG_DIR);
    std::fs::create_dir_all(&config)?;
    for name in MOVED {
        let old = root.join("System").join(name);
        let new = config.join(name);
        if old.is_file() && !new.exists() {
            std::fs::rename(&old, &new)?;
        }
    }
    for (old, new) in RENAMED {
        let old = root.join(old);
        let new = root.join(new);
        if old.is_file() && !new.exists() {
            std::fs::rename(&old, &new)?;
        }
    }
    Ok(())
}

/// Which language the interface speaks, as the card remembers it. English when it says nothing.
pub fn read_language(root: &Path) -> Lang {
    std::fs::read_to_string(root.join(CONFIG_DIR).join(LANGUAGE_FILE))
        .ok()
        .and_then(|text| Lang::parse(text.trim()))
        .unwrap_or_default()
}

/// Remembers the language across a restart.
pub fn write_language(root: &Path, lang: Lang) -> io::Result<()> {
    atomic_write(
        &root.join(CONFIG_DIR).join(LANGUAGE_FILE),
        lang.key().as_bytes(),
    )
}
