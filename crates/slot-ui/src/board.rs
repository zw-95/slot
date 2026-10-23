//! The core picker's open cart: the back half of the shell with the board in it, drawn per cart
//! because the shell is that cart's plastic and the ROM carries that cart's name.
//!
//! Drawn here in pixels and not rastered from art, which is the change: the art was one machine's
//! board, and stretching it to every other machine's shape is what put an Advance board inside a
//! Super Nintendo cart. A board is drawn at the size it is asked for now, so a portrait cart gets
//! a portrait board and each machine's own shape gets its own traces.

use slot_gfx::OUT_W;
use slot_store::{Cart, Core};

use crate::art;
use crate::cart::{size_for, CartFace, CART_H, CART_W};
use crate::shelf::foot_y;
use crate::shell::shell_for;
use crate::slot_chrome::ease;
use crate::text;
use slot_store::clean_label;



/// The cart's 240×135 at 1.55×, the size the open cart is shown at. Sharp only at its own size.
pub const BOARD_W: u32 = 372;
pub const BOARD_H: u32 = 209;

/// The ROM's body inside the board face: board units (43, 31) to (89, 89).
pub const ROM_X: u32 = 67;
pub const ROM_Y: u32 = 48;
pub const ROM_W: u32 = 71;
pub const ROM_H: u32 = 90;



/// Words to a line on the ROM, and lines to the chip. Counted in characters rather than
/// measured, so the split is a fact about the name and the tests can state it; the fitter
/// then shrinks any line that is still too wide.
const MARK_LINE_CHARS: usize = 10;
const MARK_LINES: usize = 3;
const MARK_PAD: u32 = 4;
const MARK_PX: f32 = 10.0;
const MARK_MIN_PX: f32 = 6.0;
/// Grey on black, as a mask ROM is marked: legible, and nothing like the socket names, which
/// are the words on the board that mean something.
const MARK_INK: [u8; 3] = [0xbd, 0xbd, 0xbd];

/// The game's title, a few words to a line. The dump's bracketed tags are facts about the file
/// rather than the game, and the chip does not carry them.
pub fn rom_marking(stem: &str) -> Vec<String> {
    let mut title: Vec<String> = Vec::new();
    for word in clean_label(stem).to_uppercase().split_whitespace() {
        match title.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= MARK_LINE_CHARS => {
                line.push(' ');
                line.push_str(word);
            }
            _ => title.push(word.to_string()),
        }
    }
    if title.len() > MARK_LINES {
        let tail = title.split_off(MARK_LINES - 1).join(" ");
        title.push(tail);
    }
    title
}

/// The marking alone, on nothing, the size of the ROM's body — times `scale`, because the board
/// it is inked onto is not always drawn at the size the art was drawn at: a portrait cart's
/// shell is taller than its board is, and the board inside it is smaller than an Advance one.
pub fn rom_marking_face(stem: &str, scale: f32) -> CartFace {
    let title = rom_marking(stem);
    let (w, h) = (
        (ROM_W as f32 * scale).round() as u32,
        (ROM_H as f32 * scale).round() as u32,
    );
    let mut face = CartFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    let Some(font) = text::label_font() else {
        return face;
    };
    let max_w = (w - 2 * MARK_PAD) as f32;
    let line_h = (MARK_PX * scale * 1.25).ceil() as u32;
    let mut top = h.saturating_sub(line_h * title.len() as u32) / 2;
    for line in &title {
        let layout = text::fit(font, line, max_w, 1, MARK_PX * scale, MARK_MIN_PX * scale);
        ink_band(&mut face, top, line_h, &layout, MARK_INK, 1.0);
        top += line_h;
    }
    face
}

pub fn board_face(cart: &Cart, system: slot_store::System) -> CartFace {
    // The card's own board, if it supplied one. Its chips are drawn into the art and its lamps
    // are magenta for the code to fill, so none of what follows applies: no shell-coloured
    // placeholders to paint and no ROM line to ink, because the card has already drawn both.
    if let Some(art) = crate::cart_art::board_art() {
        return CartFace {
            rgba: art.rgba.clone(),
            w: art.w,
            h: art.h,
        };
    }
    let shell = shell_for(&cart.code);
    let (w, h) = board_size(system);
    // The back half of the shell, in this cart's own plastic and this machine's own shape.
    let mut face = back_shell(system, shell.colour, w, h);
    // And the board inside it, drawn at the proportions it was drawn at and inset in the shell
    // rather than stretched to fill it: a portrait shell is nearly half again as tall as the
    // board is, and pulling the art to that shape would bow its traces and oval its pads.
    let (pw, ph) = pcb_size(w, h);
    let pcb = pcb_face(shell.colour, pw, ph, system as u32);
    let (px, py) = ((w - pw) / 2, (h - ph) / 2);
    over(&mut face, &pcb, px, py);
    // The ROM's marking travels with the board it is printed on, at the board's own scale, onto
    // the package the board drew for it.
    let s = pw as f32 / BOARD_W as f32;
    let (rx, ry, _, _) = rom_at(pw);
    over(
        &mut face,
        &rom_marking_face(&cart.stem, s),
        px + rx as u32,
        py + ry as u32,
    );
    face
}

/// The shell's back half: the machine's outline in this cart's plastic, darkened — it is the
/// inside of a shell that has just been taken apart — with its thickness showing at the rim.
fn back_shell(system: slot_store::System, shell: [u8; 3], w: u32, h: u32) -> CartFace {
    // Its own raster, not the shelf's cached one: that cache keeps a single size per machine, and
    // the board's is not it. See `silhouette::shell_at`.
    let (mask, depth) = crate::silhouette::shell_at(system, w, h);
    let depth = &depth;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        let a = mask[i];
        if a == 0 {
            continue;
        }
        // Dark at the rim and lifting inward over the first `RIM_PX` to the shell's own colour,
        // which is how thick a cart's wall is at the size an open cart is drawn. Past the rim
        // it is the plastic itself and not a shade of it: the back of this shell is the same
        // plastic the front of it is on the shelf.
        let k = 0.42 + 0.58 * (depth[i] as f32 / RIM_PX).min(1.0);
        rgba[i * 4..i * 4 + 3].copy_from_slice(&shell.map(|v| (v as f32 * k).round() as u8));
        rgba[i * 4 + 3] = a;
    }
    CartFace { rgba, w, h }
}

/// How big the board is inside a shell of `w`×`h`: nearly the shell's width, and at the board
/// art's own proportions — capped by height, so a portrait shell does not get a board taller
/// than the shell it sits in.
fn pcb_size(w: u32, h: u32) -> (u32, u32) {
    // The board art's own proportions across the shell's full width, and not a hair less: the
    // Advance's shell is the board art's own shape, so filling it is the same picture the board
    // always was. A portrait shell is taller than any board, so the height caps it and the
    // shell's own plastic shows above and below.
    let mut pw = w;
    let mut ph = (pw as f32 * BOARD_H as f32 / BOARD_W as f32).round() as u32;
    if ph > h {
        ph = h;
        pw = (ph as f32 * BOARD_W as f32 / BOARD_H as f32).round() as u32;
    }
    (pw, ph)
}

/// A board, drawn: substrate, copper traces ending in pads, an edge connector along the bottom,
/// the mask ROM that carries the game, and the silkscreen outline of the socket the core sits in.
///
/// Drawn rather than rastered from art, and drawn **at the size it is asked for** — because that
/// size is the whole of the trouble this replaces. The art was one machine's board, the Advance's,
/// and every other machine's cart came apart to show it stretched to that cart's shape: a Super
/// Nintendo cart opened on an Advance board with Advance traces on it, and a Game Boy one opened
/// on the same board pulled tall enough to bow every pad on it. This keeps its own proportions at
/// whatever size it is given, so a portrait cart gets a portrait board and a wide one a wide board.
fn pcb_face(shell: [u8; 3], w: u32, h: u32, machine: u32) -> CartFace {
    let mut face = CartFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    // Substrate, pulled a little toward this cart's own plastic: a board out of a red cart should
    // not be the same green as one out of a blue one.
    let base = mix(SUBSTRATE, shell, 0.16);
    box_fill(&mut face, 0, 0, w as i32, h as i32, base, 255);
    box_line(&mut face, 2, 2, w as i32 - 2, h as i32 - 2, SILK, 255);

    // Copper: traces on a grid, each ending in a pad, coming in from the left and down from the
    // top and stopping short of the middle where the socket is.
    let ink = mix(COPPER, base, 0.30);
    // Seeded on the machine, so two machines sharing a board size still get boards that do not
    // read as the same drawing: the Super Nintendo's row is the Advance's, and its board should
    // not look like the Advance's with a different ROM on it.
    let mut k = 0;
    let mut y = 9;
    while y < h as i32 - 14 {
        let x1 = (w as f32 * (0.16 + 0.20 * wobble(k, 7 + machine))) as i32;
        hline(&mut face, 7, x1, y, ink, 255);
        disc(&mut face, x1, y, 2.5, PAD, 255);
        k += 1;
        y += 15;
    }
    let mut x = 9;
    while x < w as i32 - 14 {
        let y1 = (h as f32 * (0.18 + 0.26 * wobble(k, 11 + machine))) as i32;
        vline(&mut face, x, 7, y1, ink, 255);
        disc(&mut face, x, y1, 2.5, PAD, 255);
        k += 1;
        x += 15;
    }

    // The edge connector: gold fingers along the bottom, which is the end that goes in first.
    let gy = h as i32 - 10;
    let mut fx = 7;
    while fx < w as i32 - 7 {
        box_fill(&mut face, fx, gy, fx + 5, h as i32 - 3, GOLD, 255);
        fx += 9;
    }

    // The mask ROM, in the top corner away from the socket: a black package with pins down both
    // long sides, and the game's marking inked onto it by the caller. It sat where the board art
    // had it, which is under the middle of the board now that the core sits there — the chip was
    // drawing straight over the game's own name.
    let (rx, ry, rw, rh) = rom_at(w);
    for i in 0..8 {
        let py = ry + 4 + i * (rh - 8) / 8;
        box_fill(&mut face, rx - 5, py, rx, py + 4, PIN, 255);
        box_fill(&mut face, rx + rw, py, rx + rw + 5, py + 4, PIN, 255);
    }
    round_box(&mut face, rx, ry, rx + rw, ry + rh, 3, ROM_BODY, 255);

    // The socket: its outline in silkscreen, so the chip reads as seated in something rather than
    // floating over the traces.
    let s = w as f32 / BOARD_W as f32;
    let (sw, sh) = ((CORE_W as f32 * s) as i32, (CORE_H as f32 * s) as i32);
    box_line(
        &mut face,
        (w as i32 - sw) / 2 - 5,
        (h as i32 - sh) / 2 - 5,
        (w as i32 - sw) / 2 + sw + 5,
        (h as i32 - sh) / 2 + sh + 5,
        SILK,
        255,
    );
    face
}

/// Where the mask ROM sits on a board `w` wide: the top corner, clear of the socket the core
/// sits in at the middle, at the ROM's own size scaled to the board's.
fn rom_at(w: u32) -> (i32, i32, i32, i32) {
    let s = w as f32 / BOARD_W as f32;
    let (rw, rh) = ((ROM_W as f32 * s) as i32, (ROM_H as f32 * s) as i32);
    (w as i32 - rw - 8, 10, rw, rh)
}

/// How many pixels in from the rim the shell's wall stops darkening it.
const RIM_PX: f32 = 26.0;

/// One line of type, centred across the face, in `band` rows from `top`, its coverage scaled by
/// `strength`. Composited straight-over whatever is already there: on a transparent pixel that
/// is ink colour at the coverage's own alpha, same as writing it outright, but on an opaque one
/// — the chip's body — the glyph's antialiased edge blends toward what was under it instead of
/// snapping to full ink.
fn ink_band(
    face: &mut CartFace,
    top: u32,
    band: u32,
    layout: &text::Layout,
    ink: [u8; 3],
    strength: f32,
) {
    let band = band.min(face.h.saturating_sub(top));
    for (i, a) in text::coverage(face.w, band, layout).into_iter().enumerate() {
        let a = (a as f32 * strength).round() as u8;
        if a == 0 {
            continue;
        }
        let at = ((top * face.w) as usize + i) * 4;
        let (a, d_a) = (a as f32 / 255.0, face.rgba[at + 3] as f32 / 255.0);
        let out_a = a + d_a * (1.0 - a);
        for (k, &c) in ink.iter().enumerate() {
            let d_rgb = face.rgba[at + k] as f32 / 255.0;
            let out_rgb = (c as f32 / 255.0 * a + d_rgb * d_a * (1.0 - a)) / out_a;
            face.rgba[at + k] = (out_rgb * 255.0).round() as u8;
        }
        face.rgba[at + 3] = (out_a * 255.0).round() as u8;
    }
}

/// Straight alpha over an opaque face.
fn over(dst: &mut CartFace, src: &CartFace, x: u32, y: u32) {
    for row in 0..src.h {
        for col in 0..src.w {
            let s = ((row * src.w + col) * 4) as usize;
            let a = src.rgba[s + 3] as u32;
            let (dx, dy) = (x + col, y + row);
            if a == 0 || dx >= dst.w || dy >= dst.h {
                continue;
            }
            let d = ((dy * dst.w + dx) * 4) as usize;
            for k in 0..3 {
                dst.rgba[d + k] =
                    ((src.rgba[s + k] as u32 * a + dst.rgba[d + k] as u32 * (255 - a)) / 255) as u8;
            }
            dst.rgba[d + 3] = dst.rgba[d + 3].max(a as u8);
        }
    }
}

fn shade(c: [u8; 3], f: f32) -> [u8; 3] {
    c.map(|v| (v as f32 * f).round().clamp(0.0, 255.0) as u8)
}

const SOCKET_SVG: &str = include_str!("../assets/socket.svg");

/// Where the open cart rests.
pub const BOARD_X: f32 = 174.0;
pub const BOARD_Y: f32 = 150.0;

/// Transparent border on every face that is drawn turned. A turned quad's own edge is not
/// antialiased; inside the texture, the linear filter softens it.
pub const TURN_PAD: u32 = 2;

/// The lid's tilt at rest, and the most the chip tips in flight.
pub const LID_TURN: f32 = -5.0 * std::f32::consts::PI / 180.0;
pub const CHIP_TIP: f32 = 4.0 * std::f32::consts::PI / 180.0;
/// How far the chip rises at mid-flight, in board units.
pub const HOP_LIFT: f32 = 10.0;

/// The open is one progress in two beats, and this is the slide's share of it: 160 ms of the
/// 420 ms in `slot::core_picker`, which a test there holds to its own constants.
pub const SLIDE_SHARE: f32 = 160.0 / 420.0;
/// How far the front half slides up off the back before it lifts: a third of the cart, the
/// travel that unhooks a real shell once its screw is out.
pub const SLIDE_UP: f32 = CART_H as f32 / 3.0;

/// The slide, eased on its own share of the progress: 0.0 closed, 1.0 unhooked.
pub fn slide_of(progress: f32) -> f32 {
    ease((progress / SLIDE_SHARE).clamp(0.0, 1.0))
}

/// The lift, eased on the rest of the progress: 0.0 still over the back, 1.0 at rest.
pub fn lift_of(progress: f32) -> f32 {
    ease(((progress - SLIDE_SHARE) / (1.0 - SLIDE_SHARE)).clamp(0.0, 1.0))
}

/// Each socket's face: board units of its top left, and its size. Left to right, evenly
/// spaced, one per core a machine can offer — so a machine offering two fills the left two,
/// which is where the board's art was drawn for them, and the third is where a third goes.
///
/// Three and not two because the picker draws one face per core on the machine's own list and
/// indexes this by position on the board: a third core with only two positions here was a
/// panic the first time a cart with three to choose between had its lid taken off. Adding a
/// fourth means adding a number here and a number in `CHIP_U`, and seeing on the device
/// whether the board has room — the run of sockets grows rightward from the art's own pair.
pub const SOCKET_U: [f32; 3] = [99.5, 171.5, 243.5];
pub const SOCKET_V: f32 = 59.1;
pub const SOCKET_W: u32 = 64;
pub const SOCKET_H: u32 = 46;

/// The chip seated in each socket, unpadded: board units of its top left, and its size.
pub const CHIP_U: [f32; 3] = [101.0, 173.0, 245.0];
pub const CHIP_V: f32 = 60.6;
pub const CHIP_W: u32 = 59;
pub const CHIP_H: u32 = 41;

pub const SHADOW_W: u32 = 66;
pub const SHADOW_H: u32 = 12;

/// 11 board units, the socket names' size in the mockup.
const NAME_PX: f32 = 17.0;
const NAME_MIN_PX: f32 = 8.0;
/// The name on the chip, which is the one name on the board and is read on its own rather than
/// as one of three: set bigger than a socket's was, and shrunk only if a core's name will not
/// fit the width at full size.
const NAME_BIG_PX: f32 = 30.0;
const NAME_BIG_MIN_PX: f32 = 15.0;
/// The name's face, and how far it travels: the width it is allowed, and the distance either
/// side of the middle that a leaving name goes to and an arriving one comes from.
pub const NAME_W: u32 = 190;
pub const NAME_H: u32 = 48;
pub const NAME_TRAVEL: f32 = 150.0;
/// The chip's size: the board's, now that it has it to itself.
pub const CORE_W: u32 = 150;
pub const CORE_H: u32 = 104;
const SOCKET_INK: [u8; 3] = [0xee, 0xf5, 0xe6];
const CHIP_INK: [u8; 3] = [0xf2, 0xf2, 0xf2];

/// Where the lid rests, and the enlargement it rests at — the Advance lid's own 144 over its
/// 240, so a portrait cart's lid comes out portrait rather than at the Advance's proportions.
const LID_REST: Placed = Placed {
    x: 288.0,
    y: 30.0,
    w: 144.0,
    h: 81.0,
};
const LID_ZOOM: f32 = 144.0 / 240.0;

/// The board's enlargement: the Advance board's own 372 over its 240. Capped by
/// `BOARD_MAX_H` for the portrait machines — see `board_size`.
const BOARD_ZOOM: f32 = 372.0 / 240.0;
const BOARD_MAX_H: f32 = 290.0;

/// A rect on the panel.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Placed {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// How big the opened cart is drawn, per machine: the cart's own shape at the board's
/// enlargement, and not one shape borrowed for all four.
///
/// It was borrowed until every machine had cores to choose between — the picker was written for
/// the Advance, and a Game Boy cart had one core and nothing to pick. A portrait cart opened at
/// the Advance's factor is 428 tall, which with the board's y put its foot 98 past the bottom of
/// a 480 panel, so a board is capped by height and a portrait one is drawn a little smaller.
pub fn board_size(system: slot_store::System) -> (u32, u32) {
    let size = size_for(system);
    let zoom = BOARD_ZOOM.min(BOARD_MAX_H / size.h as f32);
    (
        (size.w as f32 * zoom).round() as u32,
        (size.h as f32 * zoom).round() as u32,
    )
}

/// The highlighted cart as the shelf stands it, once the row has settled — which is a cart of
/// the machine whose shelf is up. A Game Boy cart is portrait and stands higher than an Advance
/// one, and the open grows out of whatever is standing there rather than out of an Advance cart
/// that is not.
pub fn shelf_cart(system: slot_store::System) -> Placed {
    let size = size_for(system);
    Placed {
        x: (OUT_W - size.w) as f32 / 2.0,
        y: foot_y(size) - size.h as f32,
        w: size.w as f32,
        h: size.h as f32,
    }
}

/// The back half: standing where the shelf stood it through the slide, then growing to its rest.
pub fn board_at(progress: f32, system: slot_store::System) -> Placed {
    let (w, h) = board_size(system);
    let rest = Placed {
        x: BOARD_X,
        y: BOARD_Y,
        w: w as f32,
        h: h as f32,
    };
    lerp(shelf_cart(system), rest, lift_of(progress))
}

/// The front half and its turn: slid up off the back, then lifted from there to its rest. The
/// lid is the cart's own front, so it is the machine's shape too — a portrait cart's lid is
/// portrait, and not an Advance lid stretched over a Game Boy shell.
pub fn lid_at(progress: f32, system: slot_store::System) -> (Placed, f32) {
    let shelf = shelf_cart(system);
    let slid = Placed {
        y: shelf.y - SLIDE_UP * slide_of(progress),
        ..shelf
    };
    let lift = lift_of(progress);
    (lerp(slid, lid_rest(system), lift), LID_TURN * lift)
}

/// Where the lid rests, and the size it rests at: the same place for every machine, at that
/// machine's own proportions.
fn lid_rest(system: slot_store::System) -> Placed {
    let size = size_for(system);
    Placed {
        w: size.w as f32 * LID_ZOOM,
        h: size.h as f32 * LID_ZOOM,
        ..LID_REST
    }
}

/// A board unit on the panel, wherever the open cart currently is.
pub fn on_board(board: Placed, u: f32, v: f32) -> (f32, f32) {
    (
        board.x + u * board.w / CART_W as f32,
        board.y + v * board.h / CART_H as f32,
    )
}

/// How much bigger or smaller than its own size a board face is drawn right now.
pub fn board_zoom(board: Placed) -> f32 {
    board.w / BOARD_W as f32
}

/// `p` with `by` more on every side.
pub fn grown(p: Placed, by: f32) -> Placed {
    Placed {
        x: p.x - by,
        y: p.y - by,
        w: p.w + 2.0 * by,
        h: p.h + 2.0 * by,
    }
}

fn lerp(a: Placed, b: Placed, t: f32) -> Placed {
    Placed {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        w: a.w + (b.w - a.w) * t,
        h: a.h + (b.h - a.h) * t,
    }
}

/// A copy with `pad` transparent pixels on every side.
pub fn padded(face: &CartFace, pad: u32) -> CartFace {
    let (w, h) = (face.w + 2 * pad, face.h + 2 * pad);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let stride = (face.w * 4) as usize;
    for row in 0..face.h {
        let from = (row * face.w * 4) as usize;
        let to = (((row + pad) * w + pad) * 4) as usize;
        rgba[to..to + stride].copy_from_slice(&face.rgba[from..from + stride]);
    }
    CartFace { rgba, w, h }
}

/// An empty socket: pads, outline, and the name of the core it is for at half strength.
pub fn socket_face(core: Core) -> CartFace {
    let svg = SOCKET_SVG;
    let rgba = art::render_svg(svg, SOCKET_W, SOCKET_H)
        .unwrap_or_else(|| vec![0; (SOCKET_W * SOCKET_H * 4) as usize]);
    let mut face = CartFace {
        rgba,
        w: SOCKET_W,
        h: SOCKET_H,
    };
    if let Some(layout) = name_layout(core, (SOCKET_W - 12) as f32, NAME_PX, NAME_MIN_PX) {
        // Inside the outline, which runs from 2.9 to 26.9 units down the face.
        ink_band(&mut face, 4, 37, &layout, SOCKET_INK, 0.5);
    }
    face
}

/// The board's colours, and the chip's: a board is green with copper on it, and a chip is black
/// with tin legs. These are what the drawing below is made of, so that there is one place to
/// change what a board looks like.
const SUBSTRATE: [u8; 3] = [0x1e, 0x53, 0x36];
const COPPER: [u8; 3] = [0xc8, 0x9a, 0x4a];
const PAD: [u8; 3] = [0xd8, 0xb4, 0x66];
const GOLD: [u8; 3] = [0xd4, 0xa8, 0x3c];
const SILK: [u8; 3] = [0xe8, 0xf0, 0xe4];
const PIN: [u8; 3] = [0xb8, 0xbc, 0xb6];
const ROM_BODY: [u8; 3] = [0x18, 0x18, 0x1b];
const CHIP_BODY: [u8; 3] = [0x22, 0x24, 0x28];

/// `a` pulled `t` of the way to `b`.
fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    a.iter()
        .zip(b)
        .map(|(&x, y)| ((x as f32 * (1.0 - t) + y as f32 * t).round() as u8))
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

/// One pixel, if it lands on the face.
fn dot(face: &mut CartFace, x: i32, y: i32, colour: [u8; 3], alpha: u8) {
    if x < 0 || y < 0 || x >= face.w as i32 || y >= face.h as i32 {
        return;
    }
    let at = ((y as u32 * face.w + x as u32) * 4) as usize;
    face.rgba[at..at + 3].copy_from_slice(&colour);
    face.rgba[at + 3] = alpha;
}

/// A filled box, clipped to the face. Half-open, as every box here is: `x1` and `y1` are one past.
fn box_fill(
    face: &mut CartFace,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    colour: [u8; 3],
    alpha: u8,
) {
    for y in y0.max(0)..y1.min(face.h as i32) {
        for x in x0.max(0)..x1.min(face.w as i32) {
            dot(face, x, y, colour, alpha);
        }
    }
}

/// The outline of a box, one pixel wide: silkscreen, and the socket the core sits in.
fn box_line(
    face: &mut CartFace,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    colour: [u8; 3],
    alpha: u8,
) {
    box_fill(face, x0, y0, x1, y0 + 1, colour, alpha);
    box_fill(face, x0, y1 - 1, x1, y1, colour, alpha);
    box_fill(face, x0, y0, x0 + 1, y1, colour, alpha);
    box_fill(face, x1 - 1, y0, x1, y1, colour, alpha);
}

/// A filled disc: a pad on a board, and the dot that says which way round a chip goes.
fn disc(face: &mut CartFace, cx: i32, cy: i32, r: f32, colour: [u8; 3], alpha: u8) {
    let rr = r * r;
    for y in (cy - r.ceil() as i32)..=(cy + r.ceil() as i32) {
        for x in (cx - r.ceil() as i32)..=(cx + r.ceil() as i32) {
            let (dx, dy) = (x - cx, y - cy);
            if (dx * dx + dy * dy) as f32 <= rr {
                dot(face, x, y, colour, alpha);
            }
        }
    }
}

/// A box with its corners rounded, which is what a chip and a ROM package are.
fn round_box(
    face: &mut CartFace,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    r: i32,
    colour: [u8; 3],
    alpha: u8,
) {
    for y in y0..y1 {
        for x in x0..x1 {
            let cx = if x < x0 + r {
                x0 + r
            } else if x >= x1 - r {
                x1 - r - 1
            } else {
                x
            };
            let cy = if y < y0 + r {
                y0 + r
            } else if y >= y1 - r {
                y1 - r - 1
            } else {
                y
            };
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx + dy * dy <= r * r {
                dot(face, x, y, colour, alpha);
            }
        }
    }
}

/// A row and a column, which between them are every trace on the board.
fn hline(face: &mut CartFace, x0: i32, x1: i32, y: i32, colour: [u8; 3], alpha: u8) {
    box_fill(face, x0, y, x1, y + 1, colour, alpha);
}

fn vline(face: &mut CartFace, x: i32, y0: i32, y1: i32, colour: [u8; 3], alpha: u8) {
    box_fill(face, x, y0, x + 1, y1, colour, alpha);
}

/// A steady "random" in `0..=1` from two integers: the same board every time for the same cart,
/// and no state to carry.
fn wobble(k: u32, salt: u32) -> f32 {
    let x = k.wrapping_mul(2_654_435_761) ^ salt.wrapping_mul(40_503);
    ((x >> 16) & 0xff) as f32 / 255.0
}

/// The one chip the open cart shows, and now the only thing on the board besides the name.
///
/// **Unnamed**, which is the change: the name used to be inked into the chip, and a name that
/// cannot move cannot leave. It is a face of its own now (`core_name_face`), because when the
/// core changes it is the name that travels and the chip under it that stays — the same chip,
/// whichever of them is in hand.
///
/// Drawn, not rastered from art: a body, legs down all four sides, a lit top and left edge, and
/// the dot that says which way round it goes. What it is not is a picture of one machine's chip,
/// which is what it was — the same Advance chip whatever cart had been opened.
///
/// Far bigger than a socket's chip was: three sockets shared the board between them and one chip
/// has it to itself. Padded, because it is drawn turned when the board refuses a press.
pub fn chip_face(_core: Core) -> CartFace {
    let mut face = CartFace {
        rgba: vec![0; (CORE_W * CORE_H * 4) as usize],
        w: CORE_W,
        h: CORE_H,
    };
    // The body in from the edge, with room for legs all round it.
    let (bx, by, bw, bh) = (16, 14, CORE_W as i32 - 32, CORE_H as i32 - 28);
    // Legs first, so the body covers their inner ends: they come out from under the chip.
    for i in 0..8 {
        let x = bx + i * bw / 8 + 1;
        box_fill(&mut face, x, by - 8, x + 6, by + 2, PIN, 255);
        box_fill(&mut face, x, by + bh - 2, x + 6, by + bh + 8, PIN, 255);
    }
    for i in 0..5 {
        let y = by + i * bh / 5 + 1;
        box_fill(&mut face, bx - 8, y, bx + 2, y + 6, PIN, 255);
        box_fill(&mut face, bx + bw - 2, y, bx + bw + 8, y + 6, PIN, 255);
    }
    round_box(&mut face, bx, by, bx + bw, by + bh, 5, CHIP_BODY, 255);
    // Lit along the top and the left, which is the whole of what makes it a chip and not a rect.
    let lit = shade(CHIP_BODY, 1.9);
    hline(&mut face, bx + 5, bx + bw - 5, by + 1, lit, 255);
    vline(&mut face, bx + 1, by + 5, by + bh - 5, lit, 255);
    disc(&mut face, bx + 11, by + 11, 3.0, SILK, 255);
    padded(&face, TURN_PAD)
}

/// A core's name on nothing, wide enough for the longest of them and centred in that width so
/// that it can slide: while the core changes, two of these are drawn at once, one leaving and
/// one arriving, and both travel the same distance either side of the middle.
pub fn core_name_face(core: Core) -> CartFace {
    let mut face = CartFace {
        rgba: vec![0; (NAME_W * NAME_H * 4) as usize],
        w: NAME_W,
        h: NAME_H,
    };
    let Some(layout) = name_layout(core, (NAME_W - 8) as f32, NAME_BIG_PX, NAME_BIG_MIN_PX) else {
        return face;
    };
    // Centred in the face's height, in a band the size of the type.
    let band = (NAME_BIG_PX * 1.6).ceil() as u32;
    ink_band(
        &mut face,
        (NAME_H - band) / 2,
        band,
        &layout,
        CHIP_INK,
        1.0,
    );
    face
}

/// A soft dark oval, for under the chip while it is off the board.
pub fn chip_shadow_face() -> CartFace {
    let (w, h) = (SHADOW_W, SHADOW_H);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - w as f32 / 2.0) / (w as f32 / 2.0);
            let dy = (y as f32 + 0.5 - h as f32 / 2.0) / (h as f32 / 2.0);
            let fall = (1.0 - (dx * dx + dy * dy)).max(0.0);
            rgba.extend_from_slice(&[0, 0, 0, (fall * fall * 255.0) as u8]);
        }
    }
    CartFace { rgba, w, h }
}

/// A core's name exactly as the player reads it — `mGBA`, not `MGBA`. `text::fit` capitalises
/// everything it lays out, so this builds the layout itself, shrinking only if it must.
fn name_layout(core: Core, max_w: f32, start_px: f32, min_px: f32) -> Option<text::Layout> {
    let font = text::label_font()?;
    let tracking = |px: f32| (px * 0.10).round();
    let mut px = start_px;
    while px > min_px && text::line_width(font, core.text(), px, tracking(px)) > max_w {
        px -= 1.0;
    }
    Some(text::Layout {
        lines: vec![core.text().to_string()],
        px,
        tracking: tracking(px),
    })
}
