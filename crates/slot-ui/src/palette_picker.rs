//! The page that picks a screen's colour: **one choice at a time, looked at against a picture**.
//!
//! A board of swatches answers "what is this one called" and leaves the rest to the head. What is
//! asked here is what the picture looks like under it, and that is answered by *showing the
//! picture under it*: the cover of a game of this machine — or the game itself, when the page was
//! opened over a game being played — put through the entry the cursor is on, so walking the list
//! walks the looks.
//!
//! The first entry is 关闭: what the machine put out before anybody chose, and what a game wears
//! until one is chosen for it.

use crate::page::{label_into, legends_into, rounded, Legend, DIM, INK, LIT};
use crate::plate::UndoFace;

pub const PICK_W: u32 = 680;
pub const PICK_H: u32 = 440;

/// The room the picture is drawn in. Everything else on the page is arranged round it.
pub const PREVIEW_W: u32 = 400;
pub const PREVIEW_H: u32 = 260;

/// One entry, as a list of them is not what this page draws: kept for the rows that have no room
/// for a picture of their own.
#[derive(Clone, Debug)]
pub struct PaletteOption {
    pub name: String,
    /// A handful of colours as this entry draws them.
    pub swatch: Vec<[u8; 3]>,
    /// Whether this is the one already worn.
    pub used: bool,
}

/// Drawn whole each time what it shows changes, and compared before that to know whether it has:
/// so **the comparison asks what is chosen, not what is drawn**.
#[derive(Clone, Debug)]
pub struct PalettePicker {
    pub title: String,
    /// The picture under the entry the cursor is on, `PREVIEW_W` by `PREVIEW_H`, RGBA.
    pub preview: Vec<u8>,
    /// What the cursor is on, called.
    pub name: String,
    /// Where the cursor is, out of how many. `0` is 关闭.
    pub at: usize,
    pub total: usize,
    /// What each button does here.
    pub legends: Vec<Legend>,
}

impl PartialEq for PalettePicker {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.name == other.name
            && self.at == other.at
            && self.total == other.total
            && self.legends == other.legends
            // The picture is remade for every entry, so two entries are never the same page.
            && self.preview.len() == other.preview.len()
            && self.preview == other.preview
    }
}

const TITLE_H: u32 = 34;
const PREVIEW_Y: u32 = 42;
/// A hair of frame round the picture, so a dark one is still a thing with edges.
const MAT: u32 = 4;
const NAME_Y: u32 = PREVIEW_Y + PREVIEW_H + 10;
const ROW_H: u32 = 30;
const FOOT_Y: u32 = PICK_H - 40;

impl PalettePicker {
    pub fn face(&self) -> UndoFace {
        let mut rgba = vec![0u8; (PICK_W * PICK_H * 4) as usize];
        label_into(
            &mut rgba,
            PICK_W,
            (0, 0, PICK_W, TITLE_H),
            &self.title,
            DIM,
            24.0,
            true,
        );
        let (x, y) = ((PICK_W - PREVIEW_W) / 2, PREVIEW_Y);
        rounded(
            &mut rgba,
            PICK_W,
            (x - MAT, y - MAT, PREVIEW_W + 2 * MAT, PREVIEW_H + 2 * MAT),
            8,
            LIT,
        );
        blit(&mut rgba, &self.preview, x, y);
        // The name, then where the cursor is in the list: the two lines under the picture, which is
        // all a page that is mostly picture has room to say.
        label_into(
            &mut rgba,
            PICK_W,
            (0, NAME_Y, PICK_W, ROW_H),
            &self.name,
            INK,
            22.0,
            true,
        );
        let total = self.total.max(1);
        label_into(
            &mut rgba,
            PICK_W,
            (0, NAME_Y + ROW_H, PICK_W, ROW_H),
            &format!("{} / {}", self.at + 1, total),
            DIM,
            20.0,
            true,
        );
        legends_into(
            &mut rgba,
            PICK_W,
            (0, FOOT_Y, PICK_W, PICK_H - FOOT_Y),
            &self.legends,
            INK,
        );
        UndoFace {
            rgba,
            w: PICK_W,
            h: PICK_H,
        }
    }
}

/// The picture onto the page, whole, at the size it was drawn.
fn blit(rgba: &mut [u8], src: &[u8], x: u32, y: u32) {
    let want = (PREVIEW_W * PREVIEW_H * 4) as usize;
    if src.len() < want {
        return;
    }
    let row = (PREVIEW_W * 4) as usize;
    for at in 0..PREVIEW_H {
        let to = (((y + at) * PICK_W + x) * 4) as usize;
        let from = (at * PREVIEW_W * 4) as usize;
        rgba[to..to + row].copy_from_slice(&src[from..from + row]);
    }
}
