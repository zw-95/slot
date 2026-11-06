//! The page that picks a cart's shell: **a row of carts, one for each choice**, standing the way
//! the games stand on the shelf rather than a list of their names.
//!
//! A shell is a thing you look at. A list can say "模版 2" and "黑色" and leave you to guess what
//! either looks like on the cart in your hand; the row shows you — every entry is the cart drawn
//! as that entry makes it, so choosing is looking. The one under the cursor is the biggest and
//! sits in the middle, which is only how a shelf looks when you are standing close to one cart.

use crate::cart::CartFace;
use crate::page::{label_into, legends_into, over, rounded, shrink, Legend, DIM, INK, LIT};
use crate::plate::UndoFace;

pub const PICK_W: u32 = 680;
pub const PICK_H: u32 = 440;

/// One choice in the row: the cart as this choice draws it, and what to call it.
#[derive(Clone, Debug)]
pub struct ShellOption {
    /// The cart's face with this choice applied, whatever the card says it is wearing now.
    pub face: CartFace,
    pub name: String,
    /// Whether the cart is already wearing it.
    pub used: bool,
}

/// Drawn whole each time what it shows changes, and compared before that to know whether it has:
/// so **the comparison asks what is chosen, not what is drawn**. Two of these holding different
/// carts differ by the cursor alone as far as anyone can see without reading megabytes of pixel.
#[derive(Clone, Debug)]
pub struct ShellPicker {
    pub title: String,
    pub options: Vec<ShellOption>,
    pub cursor: usize,
    /// What each button does here.
    pub legends: Vec<Legend>,
}

impl PartialEq for ShellPicker {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.cursor == other.cursor
            && self.legends == other.legends
            && self.options.len() == other.options.len()
            && self
                .options
                .iter()
                .zip(&other.options)
                .all(|(a, b)| a.name == b.name && a.used == b.used)
    }
}

const TITLE_H: u32 = 34;
/// The line every cart in the row stands on: their bottoms, like a shelf's.
const FOOT: u32 = 336;
/// How tall the cart under the cursor is drawn, and how tall the rest are.
const BIG_H: u32 = 252;
const SMALL_H: u32 = 188;
/// The room between two carts in the row.
const GAP: u32 = 18;
const NAME_Y: u32 = 346;
const NAME_H: u32 = 32;
const HINTS_H: u32 = 40;

impl ShellPicker {
    pub fn face(&self) -> UndoFace {
        let mut rgba = vec![0u8; (PICK_W * PICK_H * 4) as usize];
        let at = self.cursor.min(self.options.len().saturating_sub(1));
        label_into(
            &mut rgba,
            PICK_W,
            (0, 0, PICK_W, TITLE_H),
            &self.title,
            DIM,
            24.0,
            true,
        );
        let centre = PICK_W as i64 / 2;

        // The cursor's cart first and in the middle, then outwards either side of it: everything
        // is placed from the middle, so the row never jumps when the cursor moves.
        let mut right = centre;
        let mut left = centre;
        if let Some(option) = self.options.get(at) {
            let (w, _) = size_of(&option.face, BIG_H);
            right = centre + w as i64 / 2 + GAP as i64;
            left = centre - w as i64 / 2 - GAP as i64;
            put(&mut rgba, option, centre - w as i64 / 2, BIG_H, true);
        }
        for step in 1..self.options.len() {
            let Some(option) = self.options.get(at + step) else {
                break;
            };
            let (w, _) = size_of(&option.face, SMALL_H);
            put(&mut rgba, option, right, SMALL_H, false);
            right += w as i64 + GAP as i64;
            if right >= PICK_W as i64 {
                // Far enough right that the rest are off the page; the left can still be filled.
                if at < step || at + step >= self.options.len() {
                    break;
                }
            }
        }
        for step in 1..=at {
            let Some(option) = self.options.get(at - step) else {
                break;
            };
            let (w, _) = size_of(&option.face, SMALL_H);
            put(&mut rgba, option, left - w as i64, SMALL_H, false);
            left -= w as i64 + GAP as i64;
            if left <= 0 {
                break;
            }
        }

        let name = match self.options.get(at) {
            Some(option) if option.used => option.name.clone(),
            Some(option) => option.name.clone(),
            None => String::new(),
        };
        label_into(
            &mut rgba,
            PICK_W,
            (0, NAME_Y, PICK_W, NAME_H),
            &name,
            INK,
            26.0,
            false,
        );
        legends_into(
            &mut rgba,
            PICK_W,
            (0, PICK_H - HINTS_H, PICK_W, HINTS_H),
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

/// What a cart's face comes to drawn `h` tall: its own shape kept, so a tall pak stands narrow.
fn size_of(face: &CartFace, h: u32) -> (u32, u32) {
    let w = (face.w as u64 * h as u64 / face.h.max(1) as u64) as u32;
    (w.max(1), h)
}

/// Stands one cart on the line, `x` from the left. One that would hang off the page is left out
/// rather than cut: a cart sliced by the edge reads as two broken ones.
fn put(rgba: &mut [u8], option: &ShellOption, x: i64, h: u32, big: bool) {
    let (w, _) = size_of(&option.face, h);
    if x < 0 || x + w as i64 > PICK_W as i64 {
        return;
    }
    let (x, y) = (x as u32, FOOT - h);
    if big {
        let pad = 10;
        rounded(
            rgba,
            PICK_W,
            (
                x.saturating_sub(pad),
                y.saturating_sub(pad),
                w + pad * 2,
                h + pad * 2,
            ),
            12,
            LIT,
        );
    }
    let (shrunk, w) = shrink(&option.face.rgba, option.face.w, option.face.h, h);
    over(rgba, PICK_W, &shrunk, w, h, x, y);
}
