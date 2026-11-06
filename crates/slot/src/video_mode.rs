use std::path::Path;

use slot_gfx::{SRC_H, SRC_W, WHOLE_TEXTURE};
use slot_store::{ini, Platform};

/// What each game is shown as — only the games that chose for themselves are in it.
pub const VIDEO_MODE_FILE: &str = "Config/video_mode.txt";
/// What a machine's games are shown as, until one chooses for itself.
pub const VIDEO_PLATFORM_FILE: &str = "Config/video_mode_platform.txt";

/// How the picture is made to fill the screen:
/// * `Integer` — a whole number of times its own size, so no pixel is half a pixel wide. The
///   sharpest of the three, and the smallest: what is left over is black round the edge.
/// * `Fit` — as big as the screen holds at the picture's own shape, whatever number that comes to.
/// * `Stretch` — the screen itself, whatever that does to the shape.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VideoMode {
    #[default]
    Fit,
    Integer,
    Stretch,
}

impl VideoMode {
    /// The three, in the order the row walks them.
    pub const ALL: [VideoMode; 3] = [VideoMode::Integer, VideoMode::Fit, VideoMode::Stretch];

    pub fn as_str(self) -> &'static str {
        match self {
            VideoMode::Integer => "integer",
            VideoMode::Fit => "fit",
            VideoMode::Stretch => "stretch",
        }
    }

    pub fn parse(s: &str) -> Option<VideoMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "integer" => Some(VideoMode::Integer),
            // "actual" is what the two-way switch used to write, and it meant this.
            "fit" | "actual" => Some(VideoMode::Fit),
            "stretch" => Some(VideoMode::Stretch),
            _ => None,
        }
    }

    /// The one after this, or before it when `right` is false.
    pub fn step(self, right: bool) -> VideoMode {
        let at = VideoMode::ALL.iter().position(|m| *m == self).unwrap_or(0);
        let n = VideoMode::ALL.len();
        let to = if right { at + 1 } else { at + n - 1 };
        VideoMode::ALL[to % n]
    }

    /// Whether the picture is drawn a whole number of times its own size.
    pub fn integer(self) -> bool {
        self == VideoMode::Integer
    }
}

/// What a machine's games are shown as, until one of them chooses for itself.
pub fn platform_video_mode(root: &Path, platform: Platform) -> VideoMode {
    ini::value(root, VIDEO_PLATFORM_FILE, platform.dir_name())
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or_default()
}

/// How this game is shown: what it says for itself, else what its machine says, else the built-in.
pub fn video_mode_for(root: &Path, platform: Platform, stem: &str) -> VideoMode {
    ini::value(root, VIDEO_MODE_FILE, stem)
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or_else(|| platform_video_mode(root, platform))
}

/// Writes what one game is shown as. **A game asking for what its machine already says is not
/// written down**: such a game keeps following the machine, and goes on following it when the
/// machine is changed later. Only a game that differs from its machine is its own entry.
pub fn write_video_mode(
    root: &Path,
    platform: Platform,
    stem: &str,
    mode: VideoMode,
) -> std::io::Result<()> {
    if mode == platform_video_mode(root, platform) {
        return ini::remove(root, VIDEO_MODE_FILE, stem);
    }
    ini::write(root, VIDEO_MODE_FILE, stem, mode.as_str())
}

/// Writes what a machine's games are shown as. The games that never chose for themselves were
/// never written down, so they follow the machine to its new answer; the ones that did choose keep
/// their own.
pub fn write_platform_video_mode(
    root: &Path,
    platform: Platform,
    mode: VideoMode,
) -> std::io::Result<()> {
    ini::write(
        root,
        VIDEO_PLATFORM_FILE,
        platform.dir_name(),
        mode.as_str(),
    )
}

/// The part of the core's frame that is picture: the machine's own, without the black the core
/// pads either side of it. Every mode is drawn from this — a black band counted as picture would
/// be scaled along with it, and would shrink the picture to pay for itself.
pub fn source_rect(platform: Platform) -> [f32; 4] {
    let (w, h) = platform.picture();
    if (w, h) == (SRC_W, SRC_H) {
        return WHOLE_TEXTURE;
    }
    let x = SRC_W.saturating_sub(w) / 2;
    [x as f32 / SRC_W as f32, 0.0, w as f32 / SRC_W as f32, 1.0]
}
