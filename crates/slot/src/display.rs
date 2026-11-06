//! What the picture's colour can be made to look like, and the one greyscale machine's own
//! palettes.
//!
//! Two tables, because the machines want two different things. A **colour machine** (GBA, GBC,
//! SFC…) is graded: the picture already has every hue and what is asked of it is a *look*, which
//! is a 3x3 and a black point. A **Game Boy** is a greyscale machine and has no hues to grade —
//! what it has is four shades, and the whole question is what those four are tinted with, so it
//! walks its own table of palettes instead (see `crate::palettes`).
//!
//! A grade is described here twice over, and both descriptions are needed: `grade_rgb` answers
//! what a grade does to *one colour*, which is how the browser draws a swatch of it before it is
//! picked, and `grade_look` gives the shader what it needs to do the same thing to every pixel
//! at once.

/// One grade: what the browser calls it, and what it is.
pub struct Grade {
    pub name: &'static str,
    pub kind: GradeKind,
}

/// How a grade makes its look.
pub enum GradeKind {
    /// Straight through.
    Off,
    /// A 3x3 and the gamma it runs under, both in linear space when the gamma asks for it.
    Matrix([[f32; 3]; 3], f32),
    /// Two stops of a 1D ramp — dark then light — walked by the picture's own luma.
    Ramp([u8; 3], [u8; 3]),
    /// A baked 32³ cube, laid out exactly like the shader reads it: 1024×32 RGBA,
    /// `idx = (b*1024 + g*32 + r)*4`.
    Lut(&'static [u8]),
}

/// The look of no look at all: the picture as the core drew it.
pub const GRADE_OFF: u16 = 0;
/// Where 灰度 sits — the grade that keeps reading the card's own setting, so it needs naming
/// rather than being found by counting.
pub const GRADE_GRAY: u16 = 9;

/// Grayscale: plain luma, Rec.601 weights. The picture collapses to its brightness and loses hue
/// entirely. Run in encoded space (gamma 1.0), which is what the old behaviour was.
const DEFAULT_CC: [[f32; 3]; 3] = [
    [0.299, 0.587, 0.114],
    [0.299, 0.587, 0.114],
    [0.299, 0.587, 0.114],
];
/// AGB-001: the original Game Boy Advance panel. It had a small, non-sRGB gamut, so games —
/// mastered oversaturated to fight the dim reflective screen — look washed out, faintly green
/// and dark on it. A desaturation to s = 0.6 with a mild green cast, run in linear space so the
/// midtones darken as the real panel did.
const AGB_001_CC: [[f32; 3]; 3] = [
    [0.7052, 0.2301, 0.0447],
    [0.1220, 0.8515, 0.0465],
    [0.1100, 0.2160, 0.5939],
];
/// NDS: the DS Phat's screen, a softer near-sRGB middle ground. Mild desaturation (s = 0.88), no
/// tint, and a lower gamma so it reads brighter — which is what a backlit panel does.
const NDS_CC: [[f32; 3]; 3] = [
    [0.91588, 0.07044, 0.01368],
    [0.03588, 0.95044, 0.01368],
    [0.03588, 0.07044, 0.89368],
];

/// The grades a colour machine walks, **page one then page two, nine to a page**: the first is
/// what the panel was, the second is what the picture can be made to look like.
pub const GRADES: [Grade; 18] = [
    // ---- page 1: 实机系 ----
    Grade {
        name: "无矫正",
        kind: GradeKind::Off,
    },
    Grade {
        name: "AGB-001",
        kind: GradeKind::Matrix(AGB_001_CC, 2.2),
    },
    Grade {
        name: "libretro GBA",
        kind: GradeKind::Matrix(
            [
                [0.82355, 0.17745, -0.09100],
                [0.09100, 0.59150, 0.22750],
                [0.14333, 0.12968, 0.63700],
            ],
            2.2,
        ),
    },
    // VBA/No$GBA full colour: target 2.45 with a display gamma of 1.45, which one gamma cannot
    // hold exactly; 1.69 is the best single value and the look is a nostalgic one anyway.
    Grade {
        name: "VBA 全彩",
        kind: GradeKind::Matrix(
            [
                [0.73000, 0.27000, 0.00000],
                [0.08250, 0.67750, 0.24000],
                [0.08250, 0.24000, 0.67750],
            ],
            1.69,
        ),
    },
    Grade {
        name: "GBC 转置",
        kind: GradeKind::Matrix(
            [
                [0.88000, 0.05000, -0.03000],
                [0.07250, 0.60000, 0.35000],
                [0.02500, 0.05000, 1.20000],
            ],
            1.7,
        ),
    },
    Grade {
        name: "NDS",
        kind: GradeKind::Matrix(NDS_CC, 2.0),
    },
    Grade {
        name: "DB16",
        kind: GradeKind::Lut(include_bytes!("../assets/db16.bin")),
    },
    Grade {
        name: "DB32",
        kind: GradeKind::Lut(include_bytes!("../assets/db32.bin")),
    },
    Grade {
        name: "Commodore 64",
        kind: GradeKind::Lut(include_bytes!("../assets/c64.bin")),
    },
    // ---- page 2: 风格系 ----
    Grade {
        name: "灰度",
        kind: GradeKind::Matrix(DEFAULT_CC, 1.0),
    },
    Grade {
        name: "双色 深蓝米白",
        kind: GradeKind::Ramp([16, 24, 64], [246, 238, 214]),
    },
    Grade {
        name: "双色 紫青",
        kind: GradeKind::Ramp([48, 24, 80], [150, 245, 235]),
    },
    Grade {
        name: "双色 墨绿淡黄",
        kind: GradeKind::Ramp([18, 44, 32], [238, 232, 168]),
    },
    Grade {
        name: "双色 棕奶白",
        kind: GradeKind::Ramp([58, 34, 22], [250, 226, 190]),
    },
    Grade {
        name: "TealOrange",
        kind: GradeKind::Lut(include_bytes!("../assets/tealorange.bin")),
    },
    Grade {
        name: "HorrorBlue",
        kind: GradeKind::Lut(include_bytes!("../assets/horrorblue.bin")),
    },
    Grade {
        name: "Classic Chrome",
        kind: GradeKind::Lut(include_bytes!("../assets/classicchrome.bin")),
    },
    Grade {
        name: "BleachBypass",
        kind: GradeKind::Lut(include_bytes!("../assets/bleachbypass2.bin")),
    },
];

/// What a grade does to **one colour** — the same arithmetic the shader runs, on a single pixel,
/// so the browser can show what a grade is before it is picked.
///
/// It used to be run on the grey axis, which answers "what does this grade do to white and black"
/// and most grades leave those alone — every entry came out looking alike and grey. A hue is what
/// a picture is made of, so a hue is what is fed in.
pub fn grade_rgb(g: &Grade, rgb: [u8; 3]) -> [u8; 3] {
    let mut out = [0u8; 3];
    match g.kind {
        GradeKind::Off => out = rgb,
        // As the real 3x3 rather than a row sum: with a grey input every column is the same
        // number and the sum was correct, but a hue has three different ones.
        GradeKind::Matrix(m, gamma) => {
            let lin = rgb.map(|c| (c as f32 / 255.0).powf(gamma));
            for row in 0..3 {
                let x = m[row][0] * lin[0] + m[row][1] * lin[1] + m[row][2] * lin[2];
                out[row] = (x.clamp(0.0, 1.0).powf(1.0 / gamma) * 255.0).round() as u8;
            }
        }
        // Walked by the picture's own luma, Rec.601 weights.
        GradeKind::Ramp(dark, light) => {
            let l = 0.299 * rgb[0] as f32 + 0.587 * rgb[1] as f32 + 0.114 * rgb[2] as f32;
            for c in 0..3 {
                let (a, b) = (dark[c] as f32, light[c] as f32);
                out[c] = (a + (b - a) * (l / 255.0)).clamp(0.0, 255.0).round() as u8;
            }
        }
        // The cube is 32 per channel, so a colour is quantised to the cell it lands in — which is
        // what the shader's own lookup does.
        GradeKind::Lut(cube) => {
            let (r, g2, b) = (
                rgb[0] as usize / 8,
                rgb[1] as usize / 8,
                rgb[2] as usize / 8,
            );
            let at = (b * 1024 + g2 * 32 + r) * 4;
            out = [cube[at], cube[at + 1], cube[at + 2]];
        }
    }
    out
}

/// What the shader is handed to make a grade happen to the whole picture.
pub enum Look {
    /// A 3x3, a black point added after it, and the gamma both run under.
    Plain([[f32; 3]; 3], [f32; 3], f32),
    /// A baked cube for the shader to read, which is the only way to do a tone map.
    Cube(&'static [u8]),
}

/// A grade as the shader takes it.
///
/// A ramp is **not** a third kind of thing to the shader: walking two stops by the picture's luma
/// is a matrix whose rows are the distance between the stops, times the luma weights, plus the
/// dark stop as a black point. So four mechanisms reach the card and the card only has two.
pub fn grade_look(g: &Grade) -> Look {
    match g.kind {
        GradeKind::Off => Look::Plain(
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [0.0; 3],
            1.0,
        ),
        GradeKind::Matrix(m, gamma) => Look::Plain(m, [0.0; 3], gamma),
        GradeKind::Ramp(dark, light) => {
            let mut m = [[0.0f32; 3]; 3];
            for c in 0..3 {
                let span = (light[c] as f32 - dark[c] as f32) / 255.0;
                m[c] = [span * 0.299, span * 0.587, span * 0.114];
            }
            Look::Plain(m, dark.map(|c| c as f32 / 255.0), 1.0)
        }
        GradeKind::Lut(cube) => Look::Cube(cube),
    }
}

/// Where the four shades sit in the greys the core hands over. Not evenly spaced: mGBA's
/// greyscale is its own four levels (RGB555 `0x7FFF / 0x56B5 / 0x294A / 0x0000`, which is
/// 255 / 173 / 82 / 0 once widened), and the lookup has to put each shade's colour *on* its
/// level rather than on an idealised third of the range.
const GB_SHADE_LEVELS: [u8; 4] = [255, 173, 82, 0];

/// Turn four shades into a 256-entry RGBA lookup, so the shader can be exact.
///
/// Every grey the machine can hand over gets a colour, interpolated between the two shades either
/// side of it. A Game Boy only draws four levels, but the lookup is fed a continuous value as
/// well, and one that only knew about four of them would posterise it.
///
/// Built in encoded space, the space the panels are described in, so a shade lands on its own
/// colour exactly rather than a gamma away from it.
pub fn gb_lut(pal: [[u8; 3]; 4]) -> [u8; 1024] {
    let lv = GB_SHADE_LEVELS;
    let mut lut = [0u8; 1024];
    for g in 0..256u32 {
        // The pair of shades this grey falls between. `lv` runs bright to dark and `g` runs dark
        // to bright, so the search walks the other way.
        let mut lo = 0;
        while lo < 3 && (g as u8) < lv[lo + 1] {
            lo += 1;
        }
        let (hi_lv, lo_lv) = (lv[lo] as u32, lv[(lo + 1).min(3)] as u32);
        let t = if hi_lv == lo_lv {
            0
        } else {
            ((hi_lv - g) * 255 / (hi_lv - lo_lv)).min(255)
        };
        let a = pal[lo];
        let b = pal[(lo + 1).min(3)];
        let i = (g * 4) as usize;
        for c in 0..3 {
            let (x, y) = (a[c] as u32, b[c] as u32);
            lut[i + c] = ((x * (255 - t) + y * t + 127) / 255) as u8;
        }
        lut[i + 3] = 255;
    }
    lut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grade_is_reachable_and_the_plain_ones_keep_white_and_black_sane() {
        assert_eq!(GRADES.len(), 18);
        assert_eq!(GRADES[GRADE_OFF as usize].name, "无矫正");
        assert_eq!(GRADES[GRADE_GRAY as usize].name, "灰度");
        // The one grade that must not move anything: in and out are the same colour.
        assert_eq!(
            grade_rgb(&GRADES[GRADE_OFF as usize], [200, 90, 30]),
            [200, 90, 30]
        );
    }

    #[test]
    fn a_ramp_runs_from_its_dark_stop_to_its_light_one() {
        let at = GRADES
            .iter()
            .position(|g| matches!(g.kind, GradeKind::Ramp(..)))
            .expect("no ramp grade");
        let (dark, light) = match GRADES[at].kind {
            GradeKind::Ramp(d, l) => (d, l),
            _ => unreachable!(),
        };
        assert_eq!(grade_rgb(&GRADES[at], [0, 0, 0]), dark);
        assert_eq!(grade_rgb(&GRADES[at], [255, 255, 255]), light);
    }

    #[test]
    fn a_palette_lookup_puts_each_shade_on_its_own_grey() {
        let pal = [[255, 255, 255], [173, 173, 173], [82, 82, 82], [0, 0, 0]];
        let lut = gb_lut(pal);
        for (at, level) in GB_SHADE_LEVELS.iter().enumerate() {
            let i = *level as usize * 4;
            assert_eq!(
                [lut[i], lut[i + 1], lut[i + 2]],
                pal[at],
                "the lookup misses shade {at} at grey {level}"
            );
        }
        assert_eq!(lut[3], 255, "the lookup is not opaque");
    }
}
