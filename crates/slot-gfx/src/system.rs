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

    /// The rect the picture fills with the screen fully up, in offscreen pixels.
    ///
    /// Centred, which is all it takes: the GBA's own window comes out as the whole frame
    /// (240 lands in 720 three times and 160 in 480 three times, so the margins are zero),
    /// and Game Boy's comes out as 480x432 sat in the middle. One formula, and the machine
    /// that fills the panel is the case that falls out of it rather than a special case.
    pub fn screen_base(self) -> (f32, f32, f32, f32) {
        let w = self.src_w() as f32 * self.scale() as f32;
        let h = self.src_h() as f32 * self.scale() as f32;
        ((OUT_W as f32 - w) / 2.0, (OUT_H as f32 - h) / 2.0, w, h)
    }
}
