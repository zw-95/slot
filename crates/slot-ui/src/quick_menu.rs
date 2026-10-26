use crate::bitmaps::{button_legend, ButtonLegend};
use crate::draw::{Canvas, Draw, TexId, OUT_H, OUT_W};
use crate::plate::{
    blit, centred_hints, word_face, word_ink, UndoFace, CAP_GAP, HINT_EDGE, HINT_H, LEGEND_GAP,
};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::{opening, MOUTH_H};
use crate::text;
use slot_i18n::{t, MsgKey};
use slot_store::Platform;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickRow {
    FastForward,
    FastForwardSound,
    Screen,
    Rumble,
    DateTime,
    Clock,
    /// Whether the menus click and clack.
    MenuSounds,
    /// Whether the menus nudge the rumble motor.
    Haptics,
    /// Whether the sound follows the picture onto a TV.
    TvSound,
    About,
    Game,
    EjectSave,
    Turbo,
    Rewind,
    GbPalettes,
    RunAhead,
    Theme,
    Wallpaper,
    General,
    Recents,
    ShowTitles,
    WallpaperTint,
    Resume,
    States,
    Settings,
    ExitGame,
    PlayScreen,
    PlayGameplay,
    Shader,
    ColourCorrection,
    Palette,
    Bezel,
    /// Which disc of a multi-disc game is in the drive.
    Disc,
    Network,
    Wifi,
    WifiNetworks,
    Bluetooth,
    BtDevices,
    /// The screen mirrored to a Chromecast.
    Cast,
    /// Cast, from the menu over a game.
    PlayCast,
    /// SSH, file sharing, Tailscale and the VPN.
    Services,
    /// unslot's and BaseOS's updates.
    Updates,
    /// The cores that do not come with unslot, to download.
    Cores,
    /// Labels for the card's games, found online.
    GameArt,
    /// Signing in to RetroAchievements, or out.
    Achievements,
    /// Restart, Shut Down and Reload unslot.
    Power,
    Restart,
    ShutDown,
    /// Starts slot again, without restarting the device.
    Reload,
    /// Which language the interface speaks.
    Language,
    /// A platform's own page under Screen, and the rows on it.
    ScreenOf(Platform),
    ShaderOf(Platform),
    ColourCorrectionOf(Platform),
    BezelOf(Platform),
}

/// The rows that are the same whatever the platform, in index order.
const GENERIC: [QuickRow; 50] = [
    QuickRow::FastForward,
    QuickRow::FastForwardSound,
    QuickRow::Screen,
    QuickRow::Rumble,
    QuickRow::DateTime,
    QuickRow::Clock,
    QuickRow::About,
    QuickRow::Game,
    QuickRow::EjectSave,
    QuickRow::Turbo,
    QuickRow::Rewind,
    QuickRow::GbPalettes,
    QuickRow::RunAhead,
    QuickRow::Theme,
    QuickRow::Wallpaper,
    QuickRow::General,
    QuickRow::Recents,
    QuickRow::ShowTitles,
    QuickRow::WallpaperTint,
    QuickRow::Resume,
    QuickRow::States,
    QuickRow::Settings,
    QuickRow::ExitGame,
    QuickRow::PlayScreen,
    QuickRow::PlayGameplay,
    QuickRow::Shader,
    QuickRow::ColourCorrection,
    QuickRow::Palette,
    QuickRow::Bezel,
    QuickRow::Disc,
    QuickRow::Network,
    QuickRow::Wifi,
    QuickRow::WifiNetworks,
    QuickRow::Bluetooth,
    QuickRow::BtDevices,
    QuickRow::Services,
    QuickRow::Updates,
    QuickRow::Cores,
    QuickRow::GameArt,
    QuickRow::MenuSounds,
    QuickRow::Haptics,
    QuickRow::Power,
    QuickRow::Restart,
    QuickRow::ShutDown,
    QuickRow::Reload,
    QuickRow::Achievements,
    QuickRow::Cast,
    QuickRow::PlayCast,
    QuickRow::TvSound,
    QuickRow::Language,
];

/// How many rows each platform has of its own: its page, shader, colour correction and bezel.
const PER_PLATFORM: usize = 4;

/// The order the Screen page lists the platforms in, which is not the shelves' order.
const SCREEN_ORDER: [Platform; Platform::COUNT] = [
    Platform::Gba,
    Platform::Gbc,
    Platform::Gb,
    Platform::Nes,
    Platform::Snes,
    Platform::Genesis,
    Platform::MasterSystem,
    Platform::GameGear,
    Platform::PcEngine,
    Platform::NeoGeoPocket,
    Platform::WonderSwan,
    Platform::Lynx,
    Platform::PlayStation,
    Platform::Arcade,
    Platform::Cps1,
    Platform::Cps2,
    Platform::Cps3,
    Platform::NeoGeo,
    Platform::NeoGeoCd,
    Platform::Fds,
    Platform::Nintendo64,
    Platform::VirtualBoy,
    Platform::PokemonMini,
    Platform::GameAndWatch,
    Platform::Sega32x,
    Platform::SegaCd,
    Platform::Sg1000,
    Platform::Dreamcast,
    Platform::PcEngineCd,
    Platform::SuperGrafx,
    Platform::PcFx,
    Platform::Atari2600,
    Platform::Atari5200,
    Platform::Atari7800,
    Platform::Atari800,
    Platform::AtariSt,
    Platform::ColecoVision,
    Platform::Intellivision,
    Platform::Vectrex,
    Platform::Odyssey2,
    Platform::ChannelF,
    Platform::Supervision,
    Platform::MegaDuck,
    Platform::Msx,
    Platform::Commodore64,
    Platform::Amiga,
    Platform::ZxSpectrum,
    Platform::AmstradCpc,
    Platform::Dos,
    Platform::Pc98,
    Platform::X68000,
    Platform::ScummVm,
    Platform::Pico8,
    Platform::Tic80,
    Platform::Psp,
];

impl QuickRow {
    pub const COUNT: usize = GENERIC.len() + PER_PLATFORM * Platform::COUNT;

    /// Every row, each at its own `index`.
    pub const ALL: [QuickRow; QuickRow::COUNT] = {
        let mut all = [QuickRow::FastForward; QuickRow::COUNT];
        let mut i = 0;
        while i < GENERIC.len() {
            all[i] = GENERIC[i];
            i += 1;
        }
        let mut p = 0;
        while p < Platform::COUNT {
            let platform = Platform::ALL[p];
            let at = GENERIC.len() + p;
            all[at] = QuickRow::ScreenOf(platform);
            all[at + Platform::COUNT] = QuickRow::ShaderOf(platform);
            all[at + 2 * Platform::COUNT] = QuickRow::ColourCorrectionOf(platform);
            all[at + 3 * Platform::COUNT] = QuickRow::BezelOf(platform);
            p += 1;
        }
        all
    };

    pub const MAIN: [QuickRow; 6] = [
        QuickRow::General,
        QuickRow::Screen,
        QuickRow::Game,
        QuickRow::Network,
        QuickRow::Power,
        QuickRow::About,
    ];

    pub const POWER: [QuickRow; 3] = [QuickRow::Restart, QuickRow::ShutDown, QuickRow::Reload];

    pub const NETWORK: [QuickRow; 10] = [
        QuickRow::Wifi,
        QuickRow::WifiNetworks,
        QuickRow::Bluetooth,
        QuickRow::BtDevices,
        QuickRow::Cast,
        QuickRow::Services,
        QuickRow::Updates,
        QuickRow::Cores,
        QuickRow::GameArt,
        QuickRow::Achievements,
    ];

    pub const GENERAL: [QuickRow; 11] = [
        QuickRow::Wallpaper,
        QuickRow::WallpaperTint,
        QuickRow::Theme,
        QuickRow::Recents,
        QuickRow::ShowTitles,
        QuickRow::DateTime,
        QuickRow::Clock,
        QuickRow::MenuSounds,
        QuickRow::Haptics,
        QuickRow::TvSound,
        QuickRow::Language,
    ];

    pub const GAME: [QuickRow; 8] = [
        QuickRow::Disc,
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Rewind,
        QuickRow::RunAhead,
        QuickRow::Turbo,
        QuickRow::Rumble,
        QuickRow::EjectSave,
    ];

    pub const PLAY: [QuickRow; 5] = [
        QuickRow::Resume,
        QuickRow::States,
        QuickRow::Settings,
        QuickRow::PlayCast,
        QuickRow::ExitGame,
    ];

    pub const SETTINGS: [QuickRow; 2] = [QuickRow::PlayScreen, QuickRow::PlayGameplay];

    pub const PLAY_SCREEN: [QuickRow; 4] = [
        QuickRow::Shader,
        QuickRow::ColourCorrection,
        QuickRow::Palette,
        QuickRow::Bezel,
    ];

    /// The Screen page: one row for each platform.
    pub fn screen_page() -> Vec<QuickRow> {
        SCREEN_ORDER.map(QuickRow::ScreenOf).to_vec()
    }

    /// A platform's page under Screen.
    pub fn screen_of(platform: Platform) -> Vec<QuickRow> {
        let mut rows = vec![QuickRow::ShaderOf(platform)];
        if platform.spec().colour_correction {
            rows.push(QuickRow::ColourCorrectionOf(platform));
        }
        if platform.spec().palettes {
            rows.push(QuickRow::GbPalettes);
        }
        rows.push(QuickRow::BezelOf(platform));
        rows
    }

    pub fn index(self) -> usize {
        let own = |kind: usize, p: Platform| GENERIC.len() + kind * Platform::COUNT + p.index();
        match self {
            QuickRow::ScreenOf(p) => own(0, p),
            QuickRow::ShaderOf(p) => own(1, p),
            QuickRow::ColourCorrectionOf(p) => own(2, p),
            QuickRow::BezelOf(p) => own(3, p),
            row => GENERIC
                .iter()
                .position(|r| *r == row)
                .expect("every row without a platform is in GENERIC"),
        }
    }

    /// The platform a row belongs to, for the rows that belong to one.
    pub fn platform(self) -> Option<Platform> {
        match self {
            QuickRow::ScreenOf(p)
            | QuickRow::ShaderOf(p)
            | QuickRow::ColourCorrectionOf(p)
            | QuickRow::BezelOf(p) => Some(p),
            QuickRow::GbPalettes => Platform::ALL.into_iter().find(|p| p.spec().palettes),
            _ => None,
        }
    }

    pub fn page(self) -> Vec<QuickRow> {
        match self {
            QuickRow::ScreenOf(_) => QuickRow::screen_page(),
            QuickRow::ShaderOf(_)
            | QuickRow::ColourCorrectionOf(_)
            | QuickRow::BezelOf(_)
            | QuickRow::GbPalettes => match self.platform() {
                Some(p) => QuickRow::screen_of(p),
                None => QuickRow::MAIN.to_vec(),
            },
            QuickRow::Disc
            | QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::Rewind
            | QuickRow::RunAhead
            | QuickRow::Turbo
            | QuickRow::Rumble
            | QuickRow::EjectSave => QuickRow::GAME.to_vec(),
            QuickRow::Wallpaper
            | QuickRow::WallpaperTint
            | QuickRow::DateTime
            | QuickRow::Clock
            | QuickRow::MenuSounds
            | QuickRow::Haptics
            | QuickRow::TvSound
            | QuickRow::Theme
            | QuickRow::Recents
            | QuickRow::ShowTitles
            | QuickRow::Language => QuickRow::GENERAL.to_vec(),
            QuickRow::Resume
            | QuickRow::States
            | QuickRow::Settings
            | QuickRow::PlayCast
            | QuickRow::ExitGame => QuickRow::PLAY.to_vec(),
            QuickRow::PlayScreen | QuickRow::PlayGameplay => QuickRow::SETTINGS.to_vec(),
            QuickRow::Wifi
            | QuickRow::WifiNetworks
            | QuickRow::Bluetooth
            | QuickRow::BtDevices
            | QuickRow::Cast
            | QuickRow::Services
            | QuickRow::Updates
            | QuickRow::Cores
            | QuickRow::GameArt
            | QuickRow::Achievements => QuickRow::NETWORK.to_vec(),
            QuickRow::Restart | QuickRow::ShutDown | QuickRow::Reload => QuickRow::POWER.to_vec(),
            QuickRow::Shader | QuickRow::ColourCorrection | QuickRow::Palette | QuickRow::Bezel => {
                QuickRow::PLAY_SCREEN.to_vec()
            }
            _ => QuickRow::MAIN.to_vec(),
        }
    }

    pub fn parent(self) -> Option<QuickRow> {
        match self.page()[0] {
            QuickRow::ScreenOf(_) => Some(QuickRow::Screen),
            QuickRow::ShaderOf(p) => Some(QuickRow::ScreenOf(p)),
            QuickRow::Disc => Some(QuickRow::Game),
            QuickRow::Wifi => Some(QuickRow::Network),
            QuickRow::Restart => Some(QuickRow::Power),
            QuickRow::Wallpaper => Some(QuickRow::General),
            QuickRow::PlayScreen => Some(QuickRow::Settings),
            QuickRow::Shader => Some(QuickRow::PlayScreen),
            _ => None,
        }
    }

    pub fn parent_in_game(self) -> Option<QuickRow> {
        match self.parent() {
            Some(QuickRow::Game) => Some(QuickRow::PlayGameplay),
            parent => parent,
        }
    }

    pub fn centred(self) -> bool {
        let page = self.page();
        page == QuickRow::MAIN
            || page == QuickRow::PLAY
            || page == QuickRow::SETTINGS
            || page == QuickRow::POWER
            || page == QuickRow::screen_page()
    }

    pub fn child(self) -> Option<QuickRow> {
        match self {
            QuickRow::Screen => Some(QuickRow::screen_page()[0]),
            QuickRow::Game => Some(QuickRow::GAME[0]),
            QuickRow::General => Some(QuickRow::GENERAL[0]),
            QuickRow::Settings => Some(QuickRow::SETTINGS[0]),
            QuickRow::PlayScreen => Some(QuickRow::PLAY_SCREEN[0]),
            QuickRow::PlayGameplay => Some(QuickRow::GAME[0]),
            QuickRow::Network => Some(QuickRow::NETWORK[0]),
            QuickRow::Power => Some(QuickRow::POWER[0]),
            QuickRow::ScreenOf(p) => Some(QuickRow::screen_of(p)[0]),
            _ => None,
        }
    }

    pub fn shown(self, hidden: &[QuickRow]) -> Vec<QuickRow> {
        self.page()
            .into_iter()
            .filter(|r| !hidden.contains(r))
            .collect()
    }

    pub fn position(self, hidden: &[QuickRow]) -> usize {
        self.shown(hidden)
            .iter()
            .position(|r| *r == self)
            .unwrap_or(0)
    }

    /// What the row is called, in the language the interface is in.
    pub fn label(self) -> &'static str {
        match self {
            QuickRow::FastForward => t(MsgKey::RowFastForward),
            QuickRow::FastForwardSound => t(MsgKey::RowFastForwardSound),
            QuickRow::Screen | QuickRow::PlayScreen => t(MsgKey::RowScreen),
            QuickRow::Rumble => t(MsgKey::RowRumble),
            QuickRow::DateTime => t(MsgKey::RowDateTime),
            QuickRow::Clock => t(MsgKey::RowClock),
            QuickRow::MenuSounds => t(MsgKey::RowMenuSounds),
            QuickRow::Haptics => t(MsgKey::RowHaptics),
            QuickRow::TvSound => t(MsgKey::RowTvSound),
            QuickRow::About => t(MsgKey::RowAbout),
            QuickRow::Game | QuickRow::PlayGameplay => t(MsgKey::RowGameplay),
            QuickRow::EjectSave => t(MsgKey::RowEjectSave),
            QuickRow::Turbo => t(MsgKey::RowTurbo),
            QuickRow::Rewind => t(MsgKey::RowRewind),
            QuickRow::GbPalettes => t(MsgKey::RowPalettes),
            QuickRow::Palette => t(MsgKey::RowPalette),
            QuickRow::RunAhead => t(MsgKey::RowRunAhead),
            QuickRow::Theme => t(MsgKey::RowTheme),
            QuickRow::Wallpaper => t(MsgKey::RowWallpaper),
            QuickRow::General => t(MsgKey::RowGeneral),
            QuickRow::Recents => t(MsgKey::RowRecents),
            QuickRow::ShowTitles => t(MsgKey::RowShowTitles),
            QuickRow::WallpaperTint => t(MsgKey::RowWallpaperTint),
            QuickRow::Resume => t(MsgKey::RowResume),
            QuickRow::States => t(MsgKey::RowStates),
            QuickRow::Settings => t(MsgKey::RowSettings),
            QuickRow::ExitGame => t(MsgKey::RowExitGame),
            QuickRow::Shader | QuickRow::ShaderOf(_) => t(MsgKey::RowShader),
            QuickRow::ColourCorrection | QuickRow::ColourCorrectionOf(_) => {
                t(MsgKey::RowColourCorrection)
            }
            QuickRow::Bezel | QuickRow::BezelOf(_) => t(MsgKey::RowBezel),
            QuickRow::ScreenOf(p) => p.name(),
            QuickRow::Disc => t(MsgKey::RowDisc),
            QuickRow::Network => t(MsgKey::RowNetwork),
            QuickRow::Wifi => t(MsgKey::RowWifi),
            QuickRow::WifiNetworks => t(MsgKey::RowNetworks),
            QuickRow::Bluetooth => t(MsgKey::RowBluetooth),
            QuickRow::BtDevices => t(MsgKey::RowDevices),
            QuickRow::Cast | QuickRow::PlayCast => t(MsgKey::RowCast),
            QuickRow::Services => t(MsgKey::RowServices),
            QuickRow::Updates => t(MsgKey::RowUpdates),
            QuickRow::Cores => t(MsgKey::RowCores),
            QuickRow::GameArt => t(MsgKey::RowGameArt),
            QuickRow::Achievements => t(MsgKey::RowAchievements),
            QuickRow::Power => t(MsgKey::RowPower),
            QuickRow::Restart => t(MsgKey::RowRestart),
            QuickRow::ShutDown => t(MsgKey::RowShutDown),
            QuickRow::Reload => t(MsgKey::RowReload),
            QuickRow::Language => t(MsgKey::RowLanguage),
        }
    }

    pub fn opens(self) -> bool {
        matches!(
            self,
            QuickRow::DateTime
                | QuickRow::About
                | QuickRow::Screen
                | QuickRow::Game
                | QuickRow::General
                | QuickRow::Resume
                | QuickRow::States
                | QuickRow::Settings
                | QuickRow::ExitGame
                | QuickRow::PlayScreen
                | QuickRow::PlayGameplay
                | QuickRow::ScreenOf(_)
                | QuickRow::Network
                | QuickRow::WifiNetworks
                | QuickRow::BtDevices
                | QuickRow::Cast
                | QuickRow::PlayCast
                | QuickRow::Services
                | QuickRow::Updates
                | QuickRow::Cores
                | QuickRow::GameArt
                | QuickRow::Achievements
                | QuickRow::Power
                | QuickRow::Restart
                | QuickRow::ShutDown
                | QuickRow::Reload
        )
    }

    pub fn up(self, hidden: &[QuickRow]) -> QuickRow {
        let rows = self.shown(hidden);
        rows[(self.position(hidden) + rows.len() - 1) % rows.len()]
    }

    pub fn down(self, hidden: &[QuickRow]) -> QuickRow {
        let rows = self.shown(hidden);
        rows[(self.position(hidden) + 1) % rows.len()]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    Frames1,
    Dark,
    Light,
    Show,
    Hide,
    Low,
    Medium,
    High,
    Disc1,
    Disc2,
    Disc3,
    Disc4,
    Disc5,
    Hours24,
    Hours12,
    /// The language row's own two choices, each named in its own tongue.
    LangEn,
    LangZh,
}

impl QuickValue {
    pub const ALL: [QuickValue; 27] = [
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
        QuickValue::Lcd3x,
        QuickValue::Grid,
        QuickValue::Dot,
        QuickValue::Simpletex,
        QuickValue::Frames1,
        QuickValue::Dark,
        QuickValue::Light,
        QuickValue::Show,
        QuickValue::Hide,
        QuickValue::Low,
        QuickValue::Medium,
        QuickValue::High,
        QuickValue::Disc1,
        QuickValue::Disc2,
        QuickValue::Disc3,
        QuickValue::Disc4,
        QuickValue::Disc5,
        QuickValue::Hours24,
        QuickValue::Hours12,
        QuickValue::LangEn,
        QuickValue::LangZh,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    /// What the value reads as, in the language the interface is in. Speeds, shaders that are
    /// named after themselves and the tints' percentages read the same in both.
    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => t(MsgKey::ValueOn),
            QuickValue::Off => t(MsgKey::ValueOff),
            QuickValue::Lcd3x => "LCD3x",
            QuickValue::Grid => t(MsgKey::ValueGrid),
            QuickValue::Dot => t(MsgKey::ValueDot),
            QuickValue::Simpletex => "Simpletex",
            QuickValue::Frames1 => t(MsgKey::ValueFrames1),
            QuickValue::Dark => t(MsgKey::ValueDark),
            QuickValue::Light => t(MsgKey::ValueLight),
            QuickValue::Show => t(MsgKey::ValueShow),
            QuickValue::Hide => t(MsgKey::ValueHide),
            QuickValue::Low => "25%",
            QuickValue::Medium => "45%",
            QuickValue::High => "65%",
            QuickValue::Disc1 => t(MsgKey::ValueDisc1),
            QuickValue::Disc2 => t(MsgKey::ValueDisc2),
            QuickValue::Disc3 => t(MsgKey::ValueDisc3),
            QuickValue::Disc4 => t(MsgKey::ValueDisc4),
            QuickValue::Disc5 => t(MsgKey::ValueDisc5),
            QuickValue::Hours24 => t(MsgKey::ValueHours24),
            QuickValue::Hours12 => t(MsgKey::ValueHours12),
            QuickValue::LangEn => t(MsgKey::ValueLangEn),
            QuickValue::LangZh => t(MsgKey::ValueLangZh),
        }
    }

    /// The disc at `index`, counting from 0, for the most discs a game comes on.
    pub fn disc(index: u32) -> Option<QuickValue> {
        [
            QuickValue::Disc1,
            QuickValue::Disc2,
            QuickValue::Disc3,
            QuickValue::Disc4,
            QuickValue::Disc5,
        ]
        .get(index as usize)
        .copied()
    }

    pub fn speed(frames: u8) -> Option<QuickValue> {
        match frames {
            2 => Some(QuickValue::Speed2),
            3 => Some(QuickValue::Speed3),
            4 => Some(QuickValue::Speed4),
            6 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    pub fn frames(frames: u8) -> Option<QuickValue> {
        match frames {
            0 => Some(QuickValue::Off),
            1 => Some(QuickValue::Frames1),
            _ => None,
        }
    }

    pub fn flag(on: bool) -> QuickValue {
        if on { QuickValue::On } else { QuickValue::Off }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
pub const QUICK_EDGE: f32 = 32.0;
pub const QUICK_LABEL_X: f32 = 64.0;
const POINTER_GAP: f32 = 14.0;
const POINTER_SCALE: usize = 4;
const POINTER: [&str; 11] = [
    "X.......", "XX......", "XoX.....", "XooX....", "XoooX...", "XooooX..", "XoooX...", "XooX....",
    "XoX.....", "XX......", "X.......",
];
const POINTER_BLEED: u32 = 4;
const GROUND_SCRIM: f32 = 0.65;
const TYPE_DROP: f32 = 4.0;
const CARET_GAP: f32 = 14.0;
const CARET_PX: f32 = 24.0;
const LEGEND_Y: f32 = 427.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];
const PANEL_GAP: f32 = 12.0;

pub fn quick_label_face(row: QuickRow, lit: bool) -> UndoFace {
    quick_text_face(row.label(), if lit { MENU_INK } else { DIM_INK })
}

fn cap_centre(label: &fontdue::Font, h: u32) -> f32 {
    match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    }
}

pub fn quick_pointer_face() -> UndoFace {
    let s = POINTER_SCALE;
    let w = POINTER[0].len() * s;
    let h = (MENU_H + 2 * POINTER_BLEED) as usize;
    let centre = match text::label_font() {
        Some(label) => cap_centre(label, MENU_H),
        None => MENU_H as f32 / 2.0,
    } + POINTER_BLEED as f32;
    let tip = POINTER.len() / 2 * s + s / 2;
    let top = (centre - tip as f32).round() as usize;
    let mut rgba = vec![0u8; w * h * 4];
    for (y, row) in POINTER.iter().enumerate() {
        for (x, _) in row.chars().enumerate().filter(|(_, ch)| *ch != '.') {
            for py in top + y * s..top + (y + 1) * s {
                for px in x * s..(x + 1) * s {
                    let at = (py * w + px) * 4;
                    rgba[at..at + 4].copy_from_slice(&[
                        MENU_INK[0],
                        MENU_INK[1],
                        MENU_INK[2],
                        0xff,
                    ]);
                }
            }
        }
    }
    UndoFace {
        rgba,
        w: w as u32,
        h: h as u32,
    }
}

pub fn value_face(text: &str, lit: bool) -> UndoFace {
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK })
}

fn quick_text_face(label: &str, colour: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, label, OUT_W as f32, 1, MENU_PX, MENU_PX);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

pub fn quick_caret_face(right: bool) -> UndoFace {
    let glyph = if right { '\u{f0da}' } else { '\u{f0d9}' };
    let (Some(symbols), Some(label)) = (crate::icon::symbols_font(), text::label_font()) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let (m, cov) = symbols.rasterize(glyph, CARET_PX);
    let (w, h) = (m.width as u32, MENU_H);
    let centre = cap_centre(label, h);
    let top = (centre - m.height as f32 / 2.0).round() as i32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for gy in 0..m.height {
        let y = top + gy as i32;
        if y < 0 || y >= h as i32 {
            continue;
        }
        for gx in 0..m.width {
            let at = ((y as u32 * w + gx as u32) * 4) as usize;
            let a = cov[gy * m.width + gx];
            rgba[at..at + 4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub fn quick_legend_faces(ink: [u8; 3]) -> [UndoFace; 5] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), t(MsgKey::BtnBack), hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            t(MsgKey::BtnChange),
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), t(MsgKey::BtnOpen), hint, false),
        with_hint(
            button_face(ButtonLegend::B),
            t(MsgKey::RowResume),
            hint,
            true,
        ),
        with_hint(
            button_face(ButtonLegend::X),
            t(MsgKey::RowRestart),
            hint,
            false,
        ),
    ]
}

/// Cancel, Swap, Choose. Swap keeps the button before the word, the way the old arrows did.
pub fn core_button_legends() -> Vec<UndoFace> {
    vec![
        with_hint(button_face(ButtonLegend::B), t(MsgKey::BtnCancel), None, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            t(MsgKey::BtnSwap),
            None,
            true,
        ),
        with_hint(button_face(ButtonLegend::A), t(MsgKey::BtnChoose), None, false),
        with_hint(button_face(ButtonLegend::Y), t(MsgKey::BtnAllGba), None, false),
    ]
}

/// Cancel, Mode, Swap, Link, OK, Back, End Link.
pub fn link_button_legends(ink: [u8; 3]) -> Vec<UndoFace> {
    let hint = Some(ink);
    vec![
        with_hint(button_face(ButtonLegend::B), t(MsgKey::BtnCancel), hint, true),
        with_hint(button_face(ButtonLegend::Select), t(MsgKey::BtnMode), hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            t(MsgKey::BtnSwap),
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), t(MsgKey::BtnLink), hint, false),
        with_hint(button_face(ButtonLegend::A), t(MsgKey::BtnOk), hint, false),
        with_hint(button_face(ButtonLegend::B), t(MsgKey::BtnBack), hint, true),
        with_hint(
            button_face(ButtonLegend::A),
            t(MsgKey::BtnEndLink),
            hint,
            false,
        ),
    ]
}

/// `hint_on_right` puts the word after the button, for a legend parked on the left of the footer.
fn with_hint(button: UndoFace, label: &str, ink: Option<[u8; 3]>, hint_on_right: bool) -> UndoFace {
    let word = match ink {
        Some(ink) => word_ink(label, ink),
        None => word_face(label),
    };
    let w = button.w + CAP_GAP + word.w + HINT_EDGE;
    let h = button.h;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let text_y = h.saturating_sub(word.h) / 2;
    let (button_x, text_x) = match hint_on_right {
        true => (0, button.w + CAP_GAP),
        false => (word.w + CAP_GAP, 0),
    };
    blit(&mut rgba, w, &button.rgba, button.w, button.h, button_x, 0);
    blit(&mut rgba, w, &word.rgba, word.w, word.h, text_x, text_y);
    UndoFace { rgba, w, h }
}

fn button_face(which: ButtonLegend) -> UndoFace {
    let (rgba, w, h) = button_legend(which);
    UndoFace {
        rgba: rgba.to_vec(),
        w,
        h,
    }
}

fn button_pair(left: ButtonLegend, right: ButtonLegend) -> UndoFace {
    let (a, aw, ah) = button_legend(left);
    let (b, bw, bh) = button_legend(right);
    let (w, h) = (aw + bw, ah.max(bh));
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    blit(&mut rgba, w, a, aw, ah, 0, 0);
    blit(&mut rgba, w, b, bw, bh, aw, 0);
    UndoFace { rgba, w, h }
}

pub struct QuickMenuFaces {
    pub labels: Vec<[(TexId, u32, u32); 2]>,
    pub pointer: (TexId, u32, u32),
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub carets: [(TexId, u32, u32); 2],
    pub legend: [(TexId, u32); 5],
    pub legend_light: [(TexId, u32); 5],
}

pub struct QuickMenu<'a> {
    pub row: QuickRow,
    pub hidden: &'a [QuickRow],
    pub values: [Option<QuickValue>; QuickRow::COUNT],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    pub wallpaper: Option<[(TexId, u32, u32); 2]>,
    pub faces: Option<&'a QuickMenuFaces>,
    pub ground: Option<TexId>,
    pub palette: Option<[(TexId, u32, u32); 2]>,
    /// Each platform's bezel name, for its page under Screen.
    pub bezels: [Option<[(TexId, u32, u32); 2]>; Platform::COUNT],
    pub bezel: Option<[(TexId, u32, u32); 2]>,
    pub over_game: bool,
    /// The carousel footer carries the legends, so the centred pair is not drawn.
    pub footer_legends: bool,
    /// Where the sliding footer starts. The scrim stops there so it does not darken the strip.
    pub footer_top: f32,
    pub canvas: Canvas,
}

impl QuickMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        match (self.over_game, self.row.page() == QuickRow::PLAY_SCREEN) {
            (false, _) => draw_menu_ground(self.canvas, self.ground, out),
            (true, panel) => {
                let y = match panel {
                    true => {
                        let top = quick_row_top(self.row.page()[0], self.hidden);
                        top + self.canvas.extra() / 2.0 - PANEL_GAP
                    }
                    false => 0.0,
                };
                let bottom = self.footer_top.max(y);
                out.push(Draw::Rect {
                    x: 0.0,
                    y,
                    w: self.canvas.w,
                    h: bottom - y,
                    colour: [0.0, 0.0, 0.0, GROUND_SCRIM],
                });
            }
        }
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (self.canvas.w - QUICK_EDGE, MENU_PAD as f32);
        let main_w = self
            .row
            .page()
            .iter()
            .filter_map(|r| faces.labels.get(r.index()))
            .map(|l| l[1].1)
            .max()
            .unwrap_or(0);
        for row in self.row.shown(self.hidden) {
            if !in_view(row, self.row, self.hidden) {
                continue;
            }
            let top = quick_row_top_at(row, self.row, self.hidden) + self.canvas.extra() / 2.0;
            let y = top + TYPE_DROP;
            let lit = row == self.row;
            if let Some(&(tex, w, h)) = faces.labels.get(row.index()).map(|l| &l[lit as usize]) {
                let x = match row.centred() {
                    true => self.canvas.centred(main_w as f32),
                    false => QUICK_LABEL_X - pad,
                };
                push(out, tex, x, y, w, h);
                if lit {
                    let (tex, pw, ph) = faces.pointer;
                    let px = x + pad - POINTER_GAP - pw as f32;
                    push(out, tex, px, y - POINTER_BLEED as f32, pw, ph);
                }
            }
            let value = match row {
                QuickRow::DateTime => self.clock.map(|c| c[lit as usize]),
                QuickRow::Wallpaper => self.wallpaper.map(|c| c[lit as usize]),
                QuickRow::Palette => self.palette.map(|c| c[lit as usize]),
                QuickRow::BezelOf(p) => self.bezels[p.index()].map(|c| c[lit as usize]),
                QuickRow::Bezel => self.bezel.map(|c| c[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                continue;
            };
            if !lit || row.opens() {
                push(out, tex, right + pad - w as f32, y, w, h);
                continue;
            }
            let [(left_tex, lw, lh), (right_tex, rw, rh)] = faces.carets;
            let rx = right - rw as f32;
            push(out, right_tex, rx, y, rw, rh);
            let vx = rx - CARET_GAP + pad - w as f32;
            push(out, tex, vx, y, w, h);
            push(out, left_tex, vx + pad - CARET_GAP - lw as f32, y, lw, lh);
        }
        if self.over_game || self.footer_legends {
            return;
        }
        let [back, change, open, resume, _] = faces.legend;
        let back = match self.row.page() == QuickRow::PLAY {
            true => resume,
            false => back,
        };
        let other = if self.row.opens() { open } else { change };
        for (tex, w, x) in centred_hints(self.canvas, &[back, other], LEGEND_GAP) {
            push(out, tex, x, LEGEND_Y + self.canvas.extra(), w, HINT_H);
        }
    }
}

pub fn draw_menu_ground(canvas: Canvas, ground: Option<TexId>, out: &mut Vec<Draw>) {
    let (w, h) = (canvas.w, canvas.h);
    match ground {
        Some(tex) => {
            out.push(Draw::Part {
                x: 0.0,
                y: 0.0,
                w,
                h,
                tex,
                alpha: 1.0,
                uv: canvas.cover(OUT_W as f32),
            });
            out.push(Draw::Rect {
                x: 0.0,
                y: 0.0,
                w,
                h,
                colour: [0.0, 0.0, 0.0, GROUND_SCRIM],
            });
        }
        None => out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            colour: opening(),
        }),
    }
}

/// The most rows a page shows at once. A longer page scrolls to keep the chosen row in view.
pub const QUICK_ROWS_MAX: usize = 7;

/// Where `row` sits on its page while it is the chosen one.
pub fn quick_row_top(row: QuickRow, hidden: &[QuickRow]) -> f32 {
    quick_row_top_at(row, row, hidden)
}

/// Where `row` sits on its page while `chosen` is the row picked on it. On a page longer than
/// `QUICK_ROWS_MAX`, a row scrolled out of view sits off the window, above or below it.
pub fn quick_row_top_at(row: QuickRow, chosen: QuickRow, hidden: &[QuickRow]) -> f32 {
    let len = row.shown(hidden).len();
    let first = window_first(len, chosen.position(hidden));
    let rows = QUICK_PITCH * len.min(QUICK_ROWS_MAX) as f32;
    let top = match row.page() == QuickRow::PLAY_SCREEN {
        true => OUT_H as f32 - MOUTH_H - PANEL_GAP - rows,
        false => (LEGEND_Y - rows) / 2.0,
    };
    top + QUICK_PITCH * (row.position(hidden) as f32 - first as f32)
}

/// The first row in view on a page of `len` rows with the one at `chosen` picked: the window
/// keeps the chosen row as near its middle as the page's ends allow.
fn window_first(len: usize, chosen: usize) -> usize {
    if len <= QUICK_ROWS_MAX {
        return 0;
    }
    chosen
        .saturating_sub(QUICK_ROWS_MAX / 2)
        .min(len - QUICK_ROWS_MAX)
}

/// Whether `row` is in the window while `chosen` is picked.
fn in_view(row: QuickRow, chosen: QuickRow, hidden: &[QuickRow]) -> bool {
    let len = row.shown(hidden).len();
    let first = window_first(len, chosen.position(hidden));
    (first..first + QUICK_ROWS_MAX).contains(&row.position(hidden))
}

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}
