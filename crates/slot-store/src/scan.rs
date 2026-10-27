use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::gba::title_code;
use crate::platform::{Header, Platform};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cart {
    pub platform: Platform,
    pub stem: String,
    pub rom: PathBuf,
    pub label: Option<PathBuf>,
    pub title: String,
    pub code: String,
    pub shell: Option<crate::ShellChoice>,
}

impl Cart {
    /// What the shelf calls the game: an arcade set's game, else the file's name.
    pub fn shelf_name(&self) -> &str {
        match self.platform.spec().header {
            Header::Arcade if !self.title.is_empty() => &self.title,
            _ => &self.stem,
        }
    }

    pub fn read_header(&mut self) {
        match self.platform.spec().header {
            Header::Gba => {
                if let Some((title, code)) = title_code(&self.rom) {
                    self.title = title;
                    self.code = code;
                }
            }
            Header::Gb => {
                if let Some(title) = crate::gb::title(&self.rom) {
                    self.title = title;
                }
            }
            Header::Arcade => {
                if let Some(name) = crate::arcade::name(&self.stem) {
                    self.title = name.to_string();
                }
            }
            Header::None => {}
        }
    }
}

/// 往子文件夹里最深走几层。够深到能用，浅到不会被意外套娃的目录拖死。
const MAX_SCAN_DEPTH: usize = 8;

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

pub fn scan(root: &Path) -> Result<Vec<Cart>, StoreError> {
    let read = |file: &str| std::fs::read_to_string(root.join(file)).unwrap_or_default();
    let shells = crate::cart_shell::layered(
        &read(crate::CART_SHELL_FILE),
        &read(crate::LABELS_SHELL_FILE),
    );
    let mut carts = Vec::new();
    for platform in Platform::ALL {
        let dir = root.join("Games").join(platform.dir_name());
        match std::fs::read_dir(&dir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                eprintln!("slot: scan: {}: {e}", dir.display());
                continue;
            }
        }
        let labels_dir = root.join("Labels").join(platform.dir_name());
        let labels = listing(&labels_dir);
        let first = carts.len();
        // 子文件夹也扫：Games/<机种>/<子文件夹>/ 里的游戏同样上架。
        collect_games(&dir, platform, &labels_dir, &labels, &shells, &mut carts, 0);
        let listed = in_playlists(&carts[first..]);
        let mut at = first;
        carts.retain(|c| {
            at += 1;
            at <= first || !listed.contains(&c.rom)
        });
    }
    carts.sort_by(|a, b| {
        (a.platform as u8, sort_key(a.shelf_name()))
            .cmp(&(b.platform as u8, sort_key(b.shelf_name())))
    });
    Ok(carts)
}

/// Fills `carts` with everything under `dir`, walking into subfolders as it goes.
#[allow(clippy::too_many_arguments)]
fn collect_games(
    dir: &Path,
    platform: Platform,
    labels_dir: &Path,
    labels: &HashMap<String, String>,
    shells: &HashMap<String, crate::ShellChoice>,
    carts: &mut Vec<Cart>,
    depth: usize,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<std::fs::DirEntry> = entries.filter_map(|e| e.ok()).collect();
    // 排序后才有一份稳定的上架顺序，不随文件系统返回的顺序变。
    entries.sort_by_key(|e| e.file_name());
    let arcade = platform.spec().header == Header::Arcade;
    for entry in entries {
        let path = entry.path();
        // A game folder is hidden on the other cards that carry RPG Maker games, which name one
        // `.Game` rather than `Game`; here they still count as games.
        if is_hidden(&path) && !platform.spec().folder {
            continue;
        }
        if is_dir(&entry) {
            // An RPG Maker game is its own folder, RPG_RT.ldb and its maps, music and sound
            // beside it. It is one game, so it is not walked into: every map in it would
            // otherwise be a game of its own.
            if game_in(&path, platform) {
                add_game_dir(carts, platform, &path, labels_dir, labels, shells);
                continue;
            }
            // A disc set kept in its own folder, as Redump's are, is the disc named after the
            // folder. Such a folder is one game, so it is not walked into: its every track
            // would otherwise be a game of its own. Not for arcade, where a folder beside a
            // romset holds that romset's disks.
            let disc = match arcade {
                false => disc_in(&path, platform),
                true => None,
            };
            match disc {
                Some(disc) => add_cart(carts, platform, &disc, labels_dir, labels, shells, arcade),
                // 街机不往里走：那里的目录是某个 romset 的碟片目录，不是又一层文件夹。
                None if !arcade && depth + 1 < MAX_SCAN_DEPTH => {
                    collect_games(
                        &path,
                        platform,
                        labels_dir,
                        labels,
                        shells,
                        carts,
                        depth + 1,
                    );
                }
                None => {}
            }
            continue;
        }
        if !is_file(&entry) || !platform.accepts(&path) {
            continue;
        }
        add_cart(carts, platform, &path, labels_dir, labels, shells, arcade);
    }
}

#[allow(clippy::too_many_arguments)]
fn add_cart(
    carts: &mut Vec<Cart>,
    platform: Platform,
    rom: &Path,
    labels_dir: &Path,
    labels: &HashMap<String, String>,
    shells: &HashMap<String, crate::ShellChoice>,
    arcade: bool,
) {
    let Some(stem) = rom.file_stem().and_then(|s| s.to_str()) else {
        return;
    };
    if arcade && crate::arcade::is_bios(stem) {
        return;
    }
    let label = labels
        .get(&format!("{stem}.png").to_lowercase())
        .map(|name| labels_dir.join(name));
    carts.push(Cart {
        platform,
        stem: stem.to_string(),
        title: String::new(),
        code: String::new(),
        shell: shells.get(&crate::cart_shell::key(stem)).copied(),
        label,
        rom: rom.to_path_buf(),
    });
    if arcade {
        // A table lookup, cheap enough for every game at boot, unlike reading a ROM.
        carts.last_mut().map(Cart::read_header);
    }
}

/// Which subfolder of `Games/<system>/` this cart sits in, if it is not in the system's root.
/// Shelves cycle through these: everything first, then one folder at a time.
pub fn folder_of(platform: Platform, rom: &Path) -> Option<String> {
    let mut trail: Vec<String> = Vec::new();
    let mut at = rom.parent()?;
    loop {
        let name = at.file_name()?.to_str()?;
        if name == platform.dir_name() {
            break;
        }
        trail.push(name.to_string());
        at = at.parent()?;
    }
    // A disc set's folder is named after the disc inside it: it is the game, not a folder of
    // games. Look one level further out for the folder the player actually made.
    if let (Some(nearest), Some(stem)) = (trail.first(), rom.file_stem().and_then(|s| s.to_str())) {
        if nearest == stem {
            trail.remove(0);
        }
    }
    // `trail` runs innermost first, so its last entry is the outermost: the top-level folder
    // under the system's own.
    trail.pop()
}

/// The discs the shelf's `.m3u` playlists name. A multi-disc game is shown once, as its playlist,
/// not once a disc.
fn in_playlists(carts: &[Cart]) -> Vec<PathBuf> {
    carts
        .iter()
        .filter(|c| {
            c.rom
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("m3u"))
        })
        .flat_map(|c| {
            let dir = c.rom.parent().map(Path::to_path_buf).unwrap_or_default();
            std::fs::read_to_string(&c.rom)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| dir.join(l))
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn listing(dir: &Path) -> HashMap<String, String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return HashMap::new();
    };
    entries
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .map(|name| (name.to_lowercase(), name))
        .collect()
}

fn is_dir(entry: &std::fs::DirEntry) -> bool {
    match entry.file_type() {
        Ok(t) if t.is_symlink() => entry.path().is_dir(),
        Ok(t) => t.is_dir(),
        Err(_) => false,
    }
}

/// The disc in `dir` named after it, in the order the platform lists its extensions.
fn disc_in(dir: &Path, platform: Platform) -> Option<PathBuf> {
    let name = dir.file_name()?.to_str()?;
    platform
        .extensions()
        .iter()
        .map(|ext| dir.join(format!("{name}.{ext}")))
        .find(|p| p.is_file())
}

/// The files that mark a folder out as one RPG Maker game, as EasyRPG names them: either engine's
/// database, either engine's own project file, or the Windows player itself.
const GAME_MARKERS: &[&str] = &["rpg_rt.ldb", "rpg_rt.lmt", "rpg_rt.exe", "easy_rt.edb"];

/// Whether `dir` is one game rather than a folder of games. Read case-insensitively: a card is
/// formatted FAT or exFAT, where `RPG_RT.LDB` is the same file.
fn game_in(dir: &Path, platform: Platform) -> bool {
    if !platform.spec().folder {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_lowercase())
        .collect();
    GAME_MARKERS.iter().any(|m| names.iter().any(|n| n == m))
}

#[allow(clippy::too_many_arguments)]
fn add_game_dir(
    carts: &mut Vec<Cart>,
    platform: Platform,
    dir: &Path,
    labels_dir: &Path,
    labels: &HashMap<String, String>,
    shells: &HashMap<String, crate::ShellChoice>,
) {
    let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    // The dot other cards put in front of a game folder's name is theirs, not the game's.
    let stem = name.strip_prefix('.').unwrap_or(name).trim();
    if stem.is_empty() {
        return;
    }
    let label = labels
        .get(&format!("{stem}.png").to_lowercase())
        .map(|name| labels_dir.join(name));
    carts.push(Cart {
        platform,
        stem: stem.to_string(),
        title: String::new(),
        code: String::new(),
        shell: shells.get(&crate::cart_shell::key(stem)).copied(),
        label,
        rom: dir.to_path_buf(),
    });
}

fn is_file(entry: &std::fs::DirEntry) -> bool {
    match entry.file_type() {
        Ok(t) if t.is_symlink() => entry.path().is_file(),
        Ok(t) => t.is_file(),
        Err(_) => false,
    }
}

/// What a game sorts by, and so which letter of the shelf it sits under: its group, then the
/// letters it reads as. A Chinese name reads as its pinyin initials, so 腾讯翻译 stands among the
/// Ts instead of after every name in letters.
pub fn sort_key(stem: &str) -> (u8, String) {
    (group_of(stem), reading(stem))
}

/// The letters a name reads as while it is being sorted: `腾讯翻译` is `TXFY`, `洛克人Zero` is
/// `LKRZERO`. Anything not led by a hanzi is just itself, upper-cased, as it always was.
fn reading(stem: &str) -> String {
    match stem.chars().find(|c| !c.is_whitespace()) {
        Some(c) if slot_pinyin::initial(c).is_some() => slot_pinyin::initials(stem),
        _ => stem.to_uppercase(),
    }
}

fn group_of(stem: &str) -> u8 {
    match stem.chars().find(|c| !c.is_whitespace()) {
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_alphabetic() => 1,
        _ => 2,
    }
}

/// The letter a game files under, the one the shelf jumps to: for a Chinese name, the first
/// letter of its reading, so 腾讯翻译 is a T and not 腾. Digits and anything the letters do not
/// reach are `#`.
pub fn initial(stem: &str) -> char {
    let Some(c) = stem.chars().find(|c| !c.is_whitespace()) else {
        return '#';
    };
    if group_of(stem) != 1 {
        return '#';
    }
    slot_pinyin::initial(c)
        .or_else(|| c.to_uppercase().next())
        .unwrap_or('#')
}

pub fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}
