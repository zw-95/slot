//! The row of letters at the top of the shelf: `#` and A through Z, the one the hand is on in
//! the middle and largest, the ones ahead and behind to either side of it and smaller. It is
//! how a player gets about a shelf of hundreds, so it is always up, and the letter it rests on
//! is whichever one the game under the hand starts with — jumped to or simply walked to.

use slot_gfx::{Canvas, Draw, TexId};

use crate::cart::CartFace;
use crate::hud::HUD_INK;
use crate::icon::haloed;
use crate::plate::HINT_H;
use crate::slot_chrome::SLOT_NAME_Y;
use crate::text;

/// `#` first, then A through Z: the order `slot_store::initial` files games under. `#` gathers
/// every game that starts with a digit, a symbol, or a character there is no reading for — a
/// row that began at A would leave those with nowhere to stand.
pub const LETTERS: [char; 27] = [
    '#', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
    'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Where a letter stands in that row, or None for a character that is not one of them.
pub fn index_of(c: char) -> Option<usize> {
    LETTERS.iter().position(|l| *l == c)
}

/// The letters are baked one square a piece, big enough to be read at the size the middle one
/// is drawn — the row is drawn by scaling that one face down, never up.
const BOX: u32 = 40;
const PX: f32 = 30.0;

pub fn letter_face(c: char) -> CartFace {
    let Some(font) = text::label_font() else {
        return CartFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, &c.to_string(), BOX as f32, 1, PX, 12.0);
    haloed(&text::coverage(BOX, BOX, &layout), BOX, BOX, HUD_INK)
}

/// How big the letter in the middle is drawn.
const MID: f32 = 30.0;
/// How far apart the letters stand. With `SPAN` this is what fixes the row's width: it does
/// not grow or shrink, however far it has rolled.
const STEP: f32 = 24.0;
/// How many letters either side of the middle one are still shown.
const SPAN: f32 = 3.0;
/// What a letter gives up for every place it is from the middle.
const SHRINK: f32 = 0.22;

/// Draws the row at `at`, a place in the letters that need not be a whole one: halfway between
/// two is what it looks like while rolling from one to the next. The letters either side of the
/// middle are drawn smaller and fainter the further out they are, so the row reads as a wheel
/// turning rather than a line being redrawn.
pub fn draw_letters(canvas: Canvas, texes: &[Option<TexId>], at: f32, out: &mut Vec<Draw>) {
    let mid = canvas.w / 2.0;
    for (i, tex) in texes.iter().enumerate() {
        let Some(tex) = *tex else {
            continue;
        };
        let d = i as f32 - at;
        if d.abs() > SPAN {
            continue;
        }
        let size = MID * (1.0 - d.abs() * SHRINK).max(0.0);
        let alpha = (1.0 - d.abs() / (SPAN + 1.0)).clamp(0.0, 1.0);
        out.push(Draw::Tex {
            x: mid + d * STEP - size / 2.0,
            // Level with the machine's name, and so free to be taller than its line.
            y: SLOT_NAME_Y + (HINT_H as f32 - size) / 2.0,
            w: size,
            h: size,
            tex,
            alpha,
        });
    }
}
