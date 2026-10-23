//! One machine's picture geometry: how big a frame its core hands us, and where on the
//! panel that frame lands.
//!
//! This used to be two constants, because there used to be one machine. The panel is 720x480
//! and the GBA is 240x160, and 240 lands in 720 exactly three times — which is why no other
//! scale is ever asked about, and why the LCD3x table can be a 3x3 mask sampled once per
//! source pixel rather than a resample.
//!
//! Game Boy is 160x144 and lands in the panel three times too, but 160x144 is short of the
//! panel in both directions, so it does not fill it: it gets a 480x432 window centred in the
//! frame. That window is what a bezel goes around.
//!
//! Game Boy and Game Boy Color get one row between them rather than one each: their frames
//! are the same 160x144, so they want the same texture and the same window, and a second copy
//! would only be a second place for the two to drift apart.

use crate::surface::{OUT_H, OUT_W};

/// The GBA's own frame. The pair this module used to be.
pub const GBA_SRC_W: u32 = 240;
pub const GBA_SRC_H: u32 = 160;

/// Game Boy's frame, and Game Boy Color's.
pub const GB_SRC_W: u32 = 160;
pub const GB_SRC_H: u32 = 144;

/// The Super Nintendo's. Twice, not three times: 256x224 at three would be 768x672, which is
/// off the panel in both directions. Two is the only integer scale that fits the whole frame.
pub const SFC_SRC_W: u32 = 256;
pub const SFC_SRC_H: u32 = 224;

/// The Family Computer's, which is the Super Nintendo's frame stood on its head: 256x240, the
/// console's 240 visible lines with the filler ones left in by the core. At three it is 240
/// lines off the panel in height; at two it is 512x480, which fills the panel's height with
/// four pixels to spare and leaves a bezel's width down each side.
pub const FC_SRC_W: u32 = 256;
pub const FC_SRC_H: u32 = 240;

/// Which machine's picture is on the panel.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum System {
    Gba,
    /// Game Boy *and* Game Boy Color: one row, because one frame size.
    Gb,
    /// The Super Nintendo. The first machine here whose frame does not divide into the panel
    /// three times, and so the first that answers a different scale.
    Sfc,
    /// The Family Computer, whose frame is the Super Nintendo's with eight more lines on the
    /// bottom — and so the second machine that answers 2, and the second the 3x3 mask has to
    /// stay away from.
    Fc,
}

/// How a machine's frame is fitted to the panel. Three ways, and the one that was the only one
/// until there was a choice: `Integer`.
///
/// They are a **ring**, walked by one key while a game is up — see `next`. Which one is worn is
/// a preference like the mask's or the colour's, and is kept beside them in `display.txt`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Fit {
    /// Whole source pixels, as many times as fit without leaving the panel. The picture is never
    /// stretched and never cropped, and every source pixel is the same whole number of panel
    /// pixels across and down — which is what the panel was built around, and the only scale at
    /// which the LCD3x lattice means what it says: it is one RGB triad per *source* pixel.
    Integer,
    /// The picture's own shape, grown until one pair of edges is on the panel's: the largest it
    /// can be without being stretched or cropped. A source pixel is no longer a whole number of
    /// panel pixels, so the lattice stands down — see `System::screen_rect`.
    Fill,
    /// The panel's shape: pulled to its edges, whatever that does to the picture's proportions.
    /// Nothing is cropped either, and a 160x144 frame on a 720x480 panel is a wide picture —
    /// which is the trade, and the reason it is the last of the three rather than the first.
    Full,
}

impl Fit {
    /// The ring, in the order the key walks it.
    pub const ALL: [Fit; 3] = [Fit::Integer, Fit::Fill, Fit::Full];

    /// The next fit round the ring, and back to the first from the last — a walk with no end to
    /// fall off, because a preference stepped through one press at a time has to be gettable
    /// back to the front of without counting.
    pub fn next(self) -> Fit {
        match self {
            Fit::Integer => Fit::Fill,
            Fit::Fill => Fit::Full,
            Fit::Full => Fit::Integer,
        }
    }
}

impl System {
    /// How many panel pixels one source pixel covers, nearest neighbour and exactly.
    ///
    /// A method rather than a constant because this is the one number a new machine answers
    /// differently: 256x224 lands in the panel twice, not three times. Both machines that
    /// exist answer 3, which is the number the panel was built around — 240 across 720, and
    /// 160x144 across the 480x432 window it leaves.
    ///
    /// The 3x3 LCD mask is tied to this: it is one RGB triad per *source* pixel, so it only
    /// means what it says at 3. A machine that answers 2 has to be drawn without it — bezel
    /// and a single-line scanline instead — which is the thing to settle when one arrives.
    pub fn scale(self) -> u32 {
        match self {
            System::Gba => 3,
            System::Gb => 3,
            // 512x448 centred: 104 panel pixels of margin either side and 16 above and below,
            // which is where a bezel goes.
            System::Sfc => 2,
            // 512x480 centred: the panel's own height, two pixels of margin above and below,
            // and 104 down each side for a bezel. The same answer as the Super Nintendo's for
            // the same reason — three does not fit — and it is its own line because a frame
            // that fills the panel exactly is worth reading as its own case, not as SFC's.
            System::Fc => 2,
        }
    }

    /// The frame the core hands us, in pixels. This is also how many times the 3x3 mask
    /// tiles across the window: one RGB triad per source pixel, exactly — which is why a
    /// different source size needs no change to the mask itself.
    pub fn src_w(self) -> u32 {
        match self {
            System::Gba => GBA_SRC_W,
            System::Gb => GB_SRC_W,
            System::Sfc => SFC_SRC_W,
            System::Fc => FC_SRC_W,
        }
    }

    pub fn src_h(self) -> u32 {
        match self {
            System::Gba => GBA_SRC_H,
            System::Gb => GB_SRC_H,
            System::Sfc => SFC_SRC_H,
            System::Fc => FC_SRC_H,
        }
    }

    /// The rect the picture fills with the screen fully up, in offscreen pixels, **at the fit
    /// that is worn**.
    ///
    /// Centred whatever the fit is, which is all it takes: the GBA's own window comes out as the
    /// whole frame (240 lands in 720 three times and 160 in 480 three times, so the margins are
    /// zero), and Game Boy's comes out as 480x432 sat in the middle. One formula, and the machine
    /// that fills the panel is the case that falls out of it rather than a special case.
    ///
    /// The two fractional fits are where the lattice stops: it is one RGB triad per source pixel,
    /// which is a statement about a whole number of panel pixels per source pixel, and at 2.7 or
    /// 2.25 of them a triad lands across a source pixel's edge. `whole` says whether the scale
    /// the caller is drawing at is still that number, and it is false for everything but
    /// `Integer`.
    pub fn screen_rect(self, fit: Fit) -> (f32, f32, f32, f32) {
        let (sw, sh) = (self.src_w() as f32, self.src_h() as f32);
        let (w, h) = match fit {
            Fit::Integer => (sw * self.scale() as f32, sh * self.scale() as f32),
            // As large as the panel allows at the picture's own shape: the smaller of the two
            // scales is the one that puts a pair of edges on the panel's, and the other pair
            // is what is left over. A frame taller than the panel is shaped is capped by height
            // and letterboxed down the sides; one wider than it is shaped is the other way round.
            Fit::Fill => {
                let k = (OUT_W as f32 / sw).min(OUT_H as f32 / sh);
                (sw * k, sh * k)
            }
            // Stretched to the panel, and no centring left to do.
            Fit::Full => (OUT_W as f32, OUT_H as f32),
        };
        ((OUT_W as f32 - w) / 2.0, (OUT_H as f32 - h) / 2.0, w, h)
    }

    /// Whether the fit puts a whole number of panel pixels under every source pixel, which is
    /// the only scale the LCD3x lattice is drawn at.
    pub fn whole(fit: Fit) -> bool {
        matches!(fit, Fit::Integer)
    }

    /// The rect at the integer scale, which is the one the panel was built around and the one
    /// everything assumed before there were three to choose between.
    pub fn screen_base(self) -> (f32, f32, f32, f32) {
        self.screen_rect(Fit::Integer)
    }
}
