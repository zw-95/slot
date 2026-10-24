use std::path::Path;

use slot_store::{clean_label, Cart};

use crate::art;
use crate::shell::{shell_for, Finish, Shell};
use crate::silhouette::{cart_depth, cart_mask, detail_mask};
use crate::text;

/// Where the time inside `cart_face` goes, accumulated over every call since the last take.
///
/// A cart face costs something like 70 ms on this hardware, and the device has no profiler
/// and no console: this is how "which pass is actually expensive" gets answered from a
/// `boot.log` on a card instead of by argument.
pub mod face_profile {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    pub const PHASES: [&str; 6] = ["shell", "label", "mould", "recess", "paste", "clip"];

    static NANOS: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];

    pub fn add(phase: usize, d: Duration) {
        if let Some(slot) = NANOS.get(phase) {
            slot.fetch_add(d.as_nanos() as u64, Ordering::Relaxed);
        }
    }

    /// Milliseconds per phase, and resets the counters.
    pub fn take_ms() -> [f64; 6] {
        let mut out = [0.0; 6];
        for (i, slot) in NANOS.iter().enumerate() {
            out[i] = slot.swap(0, Ordering::Relaxed) as f64 / 1e6;
        }
        out
    }

    /// `shell 12.3 label 4.5 ...` — one line, for the boot log.
    pub fn line() -> String {
        let ms = take_ms();
        PHASES
            .iter()
            .zip(ms)
            .map(|(name, v)| format!("{name} {v:.1}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// The traced outline's own aspect, so `cart.svg` rasterises unstretched. Three across a
/// 720 wide row exactly, so the shelf can show a neighbour either side of the selection.
pub const CART_W: u32 = 240;
pub const CART_H: u32 = 135;
/// Internal face resolution multiplier. The cart face (shell, label, masks) is rasterised and
/// composited at `FACE_SCALE` times the logical cart size, then the shelf draws it into the
/// logical `CART_W`×`CART_H` quad. This keeps the centre cart crisp when the shelf scales it up
/// to 1.5x (and any larger scale): the texture is only ever downsampled, never stretched.
pub const FACE_SCALE: u32 = 2;
pub const FACE_W: u32 = CART_W * FACE_SCALE;
pub const FACE_H: u32 = CART_H * FACE_SCALE;

/// A cart's own size, per machine — and the row it stands on.
///
/// The Advance cart is landscape and its shelf stands three across; a Game Boy or Game Boy
/// Color cart is portrait, so it gets a row of its own, drawn at its own size rather than
/// enlarged. A portrait cart at the Advance row's 1.5x would be 414 px tall on a 480 px panel,
/// which is a cart with a shelf behind it rather than the other way round. Three 240 wide carts
/// fill the 720 panel edge to edge, so the pitch is exactly the cart's width and the neighbours
/// sit one cart apart.
///
/// The label panel is per mille of the face, x0 y0 x1 y1. The Advance cart's is the reference's;
/// the Game Boy's is measured (42 x 37 mm on a 57 x 65.5 mm shell — 7.5 mm in from each side,
/// 6 mm down), and nearly square where the Advance cart's is wide.
#[derive(Copy, Clone)]
pub struct CartSize {
    pub w: u32,
    pub h: u32,
    pub pitch: f32,
    /// How much bigger the selected cart is than its neighbours.
    pub middle: f32,
    /// And how big the neighbours are, as a share of their own size.
    pub side: f32,
    label: [u32; 4],
}

pub const fn size_for(system: slot_store::System) -> CartSize {
    match system {
        // A Super Nintendo cart shares the Advance's row — wide, and wider than it is tall —
        // so the same size and the same row figures. Its outline is its own and lives in
        // `silhouette`: the finger grooves down each side are the one thing that makes the
        // shell readable as a Super Nintendo cart, and the Advance's shell has none. A card
        // that ships its own `cart_sfc.png` still redraws it (see `cart_art::art_for`).
        slot_store::System::Gba => CartSize {
            w: CART_W,
            h: CART_H,
            pitch: 360.0,
            middle: 1.5,
            side: 1.0,
            label: [90, 228, 910, 863],
        },
        slot_store::System::Sfc => CartSize {
            w: CART_W,
            h: CART_H,
            pitch: 360.0,
            middle: 1.5,
            side: 1.0,
            // The label **plate** sits high and wide: a Super Famicom label is printed artwork
            // on a dark field, running nearly edge to edge across the top half of the shell,
            // and the band it leaves below — the trough, the foot bosses — is the moulding
            // that makes the shell read as anything at all. Lower or narrower and the plate
            // would be a sticker on a generic box; this placement is the reference shell's.
            label: [64, 176, 936, 624],
        },
        // A Family Computer cart is a **landscape** shell, and the card's own borders say how
        // landscape: every one of them is drawn about 1.4 wide to 1 high (the American cart
        // 512x364, the Subor 511x312), which at the row's width is a cart 170 high. Stood 212
        // high it was a cart a fifth taller than the picture on it — the shell stretched away
        // from its own label, and 1.5x on the shelf put 318 px of it on a 480 px panel, which
        // is nearer a screen than a cart.
        slot_store::System::Fc => CartSize {
            w: CART_W,
            h: 170,
            pitch: 360.0,
            middle: 1.5,
            side: 1.0,
            // A Family Computer label is a big blue panel pasted over most of the shell's
            // face, running wider than it is tall — the reference shell's sits from just
            // under the top edge to about four fifths down, clear of the mould lines below.
            // The band it leaves at the foot is where those lines live. Per mille of the face,
            // so it went down with it and needed no new measuring.
            label: [80, 150, 920, 720],
        },
        slot_store::System::Gb | slot_store::System::Gbc => CartSize {
            w: 240,
            h: 276,
            pitch: 240.0,
            middle: 1.0,
            side: 0.7,
            label: [132, 92, 868, 657],
        },
    }
}

impl CartSize {
    pub const fn face_w(&self) -> u32 {
        self.w * FACE_SCALE
    }

    pub const fn face_h(&self) -> u32 {
        self.h * FACE_SCALE
    }

    /// Where the row's feet stand: the carts stand on the row rather than float on it.
    ///
    /// A portrait cart is lifted clear of the title. The formula is right for the Advance row,
    /// where the gap it leaves under a 135 px cart is 37 px and the title has room in it. A
    /// 276 px cart pushes its foot 70 px further down, and the same gap collapses to 2 px with
    /// the title jammed against the row. Lifting the portrait row restores it — and takes the
    /// title with it, since `shelf_title_y` is derived from the foot: the row comes up by the
    /// whole lift and the title by half of it, so the gap grows by the other half.
    pub fn foot_y(&self) -> f32 {
        let derived = (crate::draw::OUT_H + self.h) as f32 / 2.0;
        if self.h == CART_H {
            derived
        } else {
            derived - 32.0
        }
    }

    /// The paper label's panel on the face, in face pixels.
    pub fn label_panel(&self) -> (u32, u32, u32, u32) {
        self.label_panel_at(self.face_w(), self.face_h())
    }

    /// The same panel against any size, for a caller drawing the cart smaller than its face —
    /// the row draws placeholders at the quad's size and needs the panel in those units.
    ///
    /// **Corners, not a size**: `(x0, y0, x1, y1)`, exactly as `label` stores them. A caller
    /// that wants to fill or paste *inside* the panel wants `label_box` instead; the two were
    /// confused once and cost every card with a game on it its boot.
    pub fn label_panel_at(&self, w: u32, h: u32) -> (u32, u32, u32, u32) {
        let p = |v: u32, of: u32| (of * v + 500) / 1000;
        (
            p(self.label[0], w),
            p(self.label[1], h),
            p(self.label[2], w),
            p(self.label[3], h),
        )
    }

    /// The panel as an origin and a size — `(x, y, w, h)`. What every caller that draws into
    /// the panel wants, because a pasted label is sized, not cornered: `label_panel`'s third
    /// and fourth numbers are the far edge, and handing those to `paste_label` as a width and
    /// a height pastes a label wider and taller than the cart it is going onto.
    pub fn label_box(&self) -> (u32, u32, u32, u32) {
        let (x0, y0, x1, y1) = self.label_panel();
        (x0, y0, x1 - x0, y1 - y0)
    }
}

/// The paper label, inset in the shell rather than covering it: 9% to 91% across and 22.8%
/// to 86.3% down. The vertical placement is the reference's, and the band it leaves above is
/// the moulded grip; that asymmetry is most of what makes the face read as a cartridge
/// rather than a bordered rectangle. The horizontal inset is deliberately tighter than the
/// reference's 14.1%, which was an icon's proportion rather than a cartridge's: a real
/// label runs nearly the full width with only a thin edge of plastic beside it.
pub const fn label_panel(w: u32, h: u32) -> (u32, u32, u32, u32) {
    (
        (w * 90 + 500) / 1000,
        (h * 228 + 500) / 1000,
        (w * 910 + 500) / 1000,
        (h * 863 + 500) / 1000,
    )
}

/// The label panel is laid out at the higher face resolution, so the title text is rasterised
/// sharp and only ever downsampled when drawn at the logical cart size.
pub const LABEL_X: u32 = label_panel(FACE_W, FACE_H).0;
pub const LABEL_Y: u32 = label_panel(FACE_W, FACE_H).1;
pub const LABEL_W: u32 = label_panel(FACE_W, FACE_H).2 - LABEL_X;
pub const LABEL_H: u32 = label_panel(FACE_W, FACE_H).3 - LABEL_Y;

const PAD: u32 = 10 * FACE_SCALE;
const MAX_LINES: usize = 3;
/// Three lines have to clear the label's height, and Open Sans Bold sets at about 1.36x
/// the em. The label is landscape now, so it runs out of height long before width.
const MAX_PX: f32 = LABEL_H as f32 / (MAX_LINES as f32 * 1.36);
const MIN_PX: f32 = 10.0 * FACE_SCALE as f32;

/// How far the translucent edge reaches in. Zero at this depth exactly, so a pixel any
/// further in is the plastic's own colour.
const RIM: u32 = 4 * FACE_SCALE;

pub struct CartFace {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

/// The cart's own shape in black. Drawn under a side cart so the dimming is a cart in shadow
/// rather than a cart you can see through: over a wallpaper a translucent face is a ghost,
/// and the shelf's carts are solid objects.
pub fn cart_shadow(size: CartSize, system: slot_store::System) -> CartFace {
    let (w, h) = (size.face_w(), size.face_h());
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for cover in cart_mask(system, w, h) {
        rgba.extend_from_slice(&[0, 0, 0, *cover]);
    }
    CartFace { rgba, w, h }
}

/// A cart with nothing on it: the default shell, its moulding, and the empty recess the label
/// would sit in — no label.
///
/// The stand-in for a cart whose own face has not been built yet. A jump across the alphabet
/// crosses hundreds of carts and they cannot all be rasterised, but they can all be *a cart*:
/// this one. Drawn where that cart is, it slides past as a cartridge rather than as the label's
/// colour alone, which read as a bar of paint going by.
/// The built-in shell as the current theme wants it. On the dark theme that is the table's own
/// colour; on the light theme a built-in shell takes the housing's — the off-white the top and
/// bottom bands are drawn in — so a cart reads as part of the same device as the furniture
/// around it. `card_face` states the same rule for a card's own Advance art; this is the
/// built-in path's half of it, and the reason the blank cart has to be re-minted when the
/// theme changes and not only when the machine does.
///
/// Note this reads the mode at *bake* time, so it is a colour that goes stale the moment
/// `palette::set_mode` runs: every caller of `cart_placeholder` / `cart_face` is expected to
/// re-bake after a theme change.
fn themed_shell(shell: &Shell) -> Shell {
    match crate::palette::mode() {
        crate::palette::Mode::Light => {
            let h = crate::slot_chrome::housing();
            Shell {
                colour: [
                    (h[0] * 255.0) as u8,
                    (h[1] * 255.0) as u8,
                    (h[2] * 255.0) as u8,
                ],
                finish: shell.finish,
            }
        }
        crate::palette::Mode::Dark => *shell,
    }
}

pub fn cart_placeholder(size: CartSize, system: slot_store::System) -> CartFace {
    // The stand-in wears the current theme's shell rather than the table's dark grey, which is
    // what makes it belong to the light theme's furniture too.
    let shell = themed_shell(&cart_shell(system, ""));
    let mut face = shell_face(&shell, size, system);
    mould_detail(&mut face, &shell, size, system);
    match system {
        // An empty plate, the way the blank Advance cart shows an empty recess: these two
        // machines' labels are plates and not recesses, so their blanks are plates too.
        slot_store::System::Sfc => sfc_plate(&mut face, size.label_box()),
        slot_store::System::Fc => fc_plate(&mut face, size.label_box()),
        _ => recess_label(&mut face, &shell, size.label_box()),
    }
    clip_to_silhouette(&mut face, size, system);
    face
}

/// The Super Famicom's own shell, which is not the game's to choose: every real one is the same
/// light grey plastic, so the header code that picks an Advance cart's colour has nothing to say
/// on this machine. Read off a photograph of the reference shell, not recalled.
const SFC_SHELL: Shell = Shell {
    colour: [0xc8, 0xc4, 0xbb],
    finish: Finish::Solid,
};

/// The Family Computer's own shell, which is the same story: the reference cart is that
/// orange-yellow plastic, and no game's header is going to change it. It is the colour half
/// the world means when it says "Famicom cart".
const FC_SHELL: Shell = Shell {
    colour: [0xe2, 0xa4, 0x2c],
    finish: Finish::Solid,
};

/// The shell a cart wears: the game's own colour where the machine's header carries a code, the
/// machine's own where it does not — which is both of the cartridge machines' case, and the
/// placeholder's.
fn cart_shell(system: slot_store::System, code: &str) -> Shell {
    match system {
        slot_store::System::Sfc => SFC_SHELL,
        slot_store::System::Fc => FC_SHELL,
        _ => shell_for(code),
    }
}

pub fn cart_face(cart: &Cart) -> CartFace {
    let system = cart.system();
    face_at(cart, size_for(system), crate::cart_art::art_for(system))
}

/// The cart as the machine itself shells it: **the built-in outline and the machine's own
/// plastic, with no border on it at all.** What every cart wears until somebody picks one, and
/// the first thing in the picker's ring for exactly that reason — a cart put back on its
/// machine's own shell is a cart with nothing written down about it.
pub fn plain_face(cart: &Cart, k: u32) -> CartFace {
    let system = cart.system();
    face_at(cart, scaled(system, k), None)
}

/// The cart wearing the border numbered `number`, drawn `k` times the size the shelf stands it
/// at. `None` when the card carries no such border: a card edited on a PC may hold fewer borders
/// than the choice written on it, and the honest drawing of a border that is not there is none.
pub fn border_face(root: &Path, cart: &Cart, number: u32, k: u32) -> Option<CartFace> {
    let system = cart.system();
    let art = crate::cart_art::border_art(root, system, number)?;
    let (art_w, art_h) = art.extent();
    Some(face_at(cart, sized(system, k, art_w, art_h), Some(&art)))
}

/// The cart at `k` times the size the shelf stands it at: the picker shows one large enough to
/// read the difference between two shells, and a face built at the shelf's size and stretched is
/// a face whose label is stretched along with it.
fn scaled(system: slot_store::System, k: u32) -> CartSize {
    let size = size_for(system);
    let k = k.max(1);
    // Only the two that decide the face: `pitch`, `middle` and `side` place a cart in a row of
    // them, and there is no row here. The label panel is per mille of the face, so it scales
    // with it and is left alone.
    CartSize {
        w: size.w * k,
        h: size.h * k,
        ..size
    }
}

/// The cart at `k` times the size the shelf stands it at, **and in the shape of the picture being
/// drawn**: `art_w`×`art_h` is the border's own size, and the face is built to its proportions.
///
/// One machine's borders are not one shape. The Family Computer carries three — the yellow cart
/// is half again as wide as it is tall, the Subor wider still, and the NES shell is very nearly
/// square — and building all three at the machine's own width and height is drawing three carts
/// that are not the carts in the pictures: a yellow Family Computer cart squeezed into the NES
/// shell's proportions is neither of the two, and the sticker on it is squeezed along with it.
///
/// **The height is what is held**, because a row stands its carts on a line and that line is the
/// one thing a row of different carts has in common; the width takes the art's own proportions.
/// `pitch`, `middle` and `side` stay the machine's: a cart is placed by the row it stands in, and
/// a cart wider than its neighbour is not given more room than one to walk through.
fn sized(system: slot_store::System, k: u32, art_w: u32, art_h: u32) -> CartSize {
    let size = size_for(system);
    let k = k.max(1);
    let h = size.h * k;
    let w = ((h as f32 * art_w as f32 / art_h.max(1) as f32).round() as u32).max(1);
    CartSize { w, h, ..size }
}

/// One cart's face, from the art it is wearing or from the machine's own shell when it is wearing
/// none. `art` is passed in rather than looked up because the picker asks for a face the shelf
/// never asked for — one particular border, of the dozens the card may hold.
fn face_at(cart: &Cart, size: CartSize, art: Option<&crate::cart_art::Art>) -> CartFace {
    let system = cart.system();
    let shell = cart_shell(system, &cart.code);
    // A card that supplies its own cart art is drawn from that instead of from the built-in
    // silhouette: see `cart_art`. Nothing below changes for a card that does not.
    if let Some(art) = art {
        if let Some(face) = art_face(cart, &shell, size, art) {
            return face;
        }
    }
    // Past the card's own art the built-in shell answers to the theme as well: on the light
    // theme it takes the housing's colour, which is the same rule `card_face` has just applied
    // to an Advance cart's own art — so a card with art and a card without still read as one
    // device. On the dark theme the game's own shell colour stands.
    let shell = themed_shell(&shell);

    let (lx, ly, lw, lh) = size.label_box();

    let t = std::time::Instant::now();
    let mut face = shell_face(&shell, size, system);
    face_profile::add(0, t.elapsed());

    let t = std::time::Instant::now();
    // What the label shows, sized for where it is going: on a Super Famicom that is the
    // plate's inner panel — the plate is the machine's own, printed whether or not this cart
    // has a game in it — and everywhere else it is the label panel itself.
    let label = if system == slot_store::System::Sfc {
        let inner = plate_inner((lx, ly, lw, lh));
        match cart.label.as_deref().and_then(|p| art::cover(p, inner.2, inner.3)) {
            Some(rgba) => rgba,
            None => plate_text_label(&label_text(cart), inner.2, inner.3),
        }
    } else if system == slot_store::System::Fc {
        let inner = plate_inner((lx, ly, lw, lh));
        match cart.label.as_deref().and_then(|p| art::cover(p, inner.2, inner.3)) {
            Some(rgba) => rgba,
            None => plate_text_label(&label_text(cart), inner.2, inner.3),
        }
    } else {
        match cart.label.as_deref().and_then(|p| art::cover(p, lw, lh)) {
            Some(rgba) => rgba,
            None => sized_label(&label_text(cart), lw, lh),
        }
    };
    face_profile::add(1, t.elapsed());

    let t = std::time::Instant::now();
    mould_detail(&mut face, &shell, size, system);
    face_profile::add(2, t.elapsed());

    let t = std::time::Instant::now();
    match system {
        slot_store::System::Sfc => sfc_plate(&mut face, (lx, ly, lw, lh)),
        slot_store::System::Fc => fc_plate(&mut face, (lx, ly, lw, lh)),
        _ => recess_label(&mut face, &shell, (lx, ly, lw, lh)),
    }
    face_profile::add(3, t.elapsed());

    let t = std::time::Instant::now();
    let panel = match system {
        slot_store::System::Sfc | slot_store::System::Fc => plate_inner((lx, ly, lw, lh)),
        _ => (lx, ly, lw, lh),
    };
    paste_label(&mut face, &label, panel);
    face_profile::add(4, t.elapsed());

    let t = std::time::Instant::now();
    clip_to_silhouette(&mut face, size, system);
    face_profile::add(5, t.elapsed());

    face
}

/// Colour is left alone and only alpha is cut, because the sprite pass blends straight
/// alpha rather than premultiplied.
fn clip_to_silhouette(face: &mut CartFace, size: CartSize, system: slot_store::System) {
    for (px, cover) in face
        .rgba
        .chunks_exact_mut(4)
        .zip(cart_mask(system, size.face_w(), size.face_h()))
    {
        px[3] = ((px[3] as u32 * *cover as u32 + 127) / 255) as u8;
    }
}

/// Stable across runs, which the standard hasher is not: the same game must be the same
/// colour on every boot, or the shelf is unrecognisable from memory.
pub fn label_colour(title: &str) -> [u8; 3] {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in title.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hsv_to_rgb((h % 360) as f32, 0.52, 0.74)
}

/// The header title is capped at twelve characters, so it reads `POKEMON EMER`. The
/// filename holds the real name.
pub fn label_text(cart: &Cart) -> String {
    clean_label(&cart.stem)
}

fn shell_face(shell: &Shell, size: CartSize, system: slot_store::System) -> CartFace {
    let (w, h) = (size.face_w(), size.face_h());
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    let edge = rim_colour(shell.colour);
    for depth in cart_depth(system, w, h) {
        let c = match shell.finish {
            Finish::Solid => shell.colour,
            Finish::Translucent => lerp(edge, shell.colour, (*depth as u32).min(RIM), RIM),
        };
        rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }
    CartFace { rgba, w, h }
}

/// Light through the plastic reads as a lighter, less saturated edge. Desaturating as well
/// as lightening is what keeps it from looking like a white outline drawn on the shell.
fn rim_colour(base: [u8; 3]) -> [u8; 3] {
    let mean = (base[0] as u16 + base[1] as u16 + base[2] as u16) / 3;
    base.map(|c| {
        let grey = (3 * c as u16 + mean) / 4;
        (grey + (255 - grey) * 2 / 5) as u8
    })
}

fn lerp(a: [u8; 3], b: [u8; 3], num: u32, den: u32) -> [u8; 3] {
    let mut out = [0u8; 3];
    for c in 0..3 {
        out[c] = ((a[c] as u32 * (den - num) + b[c] as u32 * num) / den) as u8;
    }
    out
}

/// The wall of the moulded recess the label sits in. Light comes from the upper left, so
/// the top and left walls are turned away from it and fall into shadow while the bottom and
/// right walls catch it. Painted before the label, so the label sits on the floor of the
/// recess with the wall showing around it.
const BEVEL: u32 = 3 * FACE_SCALE;

/// The grip ridge and the thumb notch, cut into the shell. Darkened rather than coloured:
/// moulded plastic is the same plastic, just turned away from the light.
fn mould_detail(face: &mut CartFace, shell: &Shell, size: CartSize, system: slot_store::System) {
    let dark = [
        (shell.colour[0] as f32 * 0.62) as u8,
        (shell.colour[1] as f32 * 0.62) as u8,
        (shell.colour[2] as f32 * 0.62) as u8,
    ];
    for (px, cover) in face
        .rgba
        .chunks_exact_mut(4)
        .zip(detail_mask(system, size.face_w(), size.face_h()))
    {
        let a = *cover as u32;
        if a == 0 {
            continue;
        }
        for c in 0..3 {
            px[c] = ((dark[c] as u32 * a + px[c] as u32 * (255 - a) + 127) / 255) as u8;
        }
    }
}

fn recess_label(face: &mut CartFace, shell: &Shell, panel: (u32, u32, u32, u32)) {
    let (lx, ly, lw, lh) = panel;
    let (face_w, face_h) = (face.w, face.h);
    let shade = |c: [u8; 3], f: f32| -> [u8; 3] {
        [
            (c[0] as f32 * f).clamp(0.0, 255.0) as u8,
            (c[1] as f32 * f).clamp(0.0, 255.0) as u8,
            (c[2] as f32 * f).clamp(0.0, 255.0) as u8,
        ]
    };
    let dark = shade(shell.colour, 0.55);
    let lit = shade(shell.colour, 1.45);

    let (x0, y0) = (lx - BEVEL, ly - BEVEL);
    let (x1, y1) = (lx + lw + BEVEL, ly + lh + BEVEL);
    let mut put = |x: u32, y: u32, c: [u8; 3]| {
        if x >= face_w || y >= face_h {
            return;
        }
        let d = ((y * face_w + x) * 4) as usize;
        face.rgba[d] = c[0];
        face.rgba[d + 1] = c[1];
        face.rgba[d + 2] = c[2];
    };
    for y in y0..y1 {
        for x in x0..x1 {
            let inside = (lx..lx + lw).contains(&x) && (ly..ly + lh).contains(&y);
            if inside {
                continue;
            }
            // Which wall a pixel belongs to: the nearer of the two edges it sits between.
            let from_top = y.saturating_sub(y0);
            let from_left = x.saturating_sub(x0);
            let from_bottom = y1.saturating_sub(y + 1);
            let from_right = x1.saturating_sub(x + 1);
            let upper = from_top.min(from_left);
            let lower = from_bottom.min(from_right);
            put(x, y, if upper <= lower { dark } else { lit });
        }
    }
}

/// Source over, so a label with an alpha channel shows the shell through it rather than
/// punching a hole in the cart.
///
/// Credited rather than trusted: `panel` is `(x, y, w, h)`, the label is `w * h` pixels, and
/// both are clipped to the face rather than assumed to fit inside it. The panel used to be
/// handed in as corners, which pasted a label a corner's width too wide and a corner's height
/// too tall, and the write past the end of the face took the whole frontend down before its
/// first frame — on any card with a game on it, so the machine came up as a boot loop. A
/// labelled cart drawn slightly wrong is a bug; a panic here is a handheld that cannot start.
fn paste_label(face: &mut CartFace, label: &[u8], panel: (u32, u32, u32, u32)) {
    let (lx, ly, lw, lh) = panel;
    let face_w = face.w;
    for y in 0..lh {
        // Past the bottom of the face, so is every row below this one.
        if y + ly >= face.h {
            break;
        }
        for x in 0..lw {
            if x + lx >= face_w {
                break;
            }
            let s = ((y * lw + x) * 4) as usize;
            // The same mistake from the other side: a label buffer shorter than the panel it
            // is being pasted into. Clipped too, for the same reason.
            let Some(src) = label.get(s..s + 4) else {
                break;
            };
            let a = src[3] as u32;
            if a == 0 {
                continue;
            }
            let d = (((y + ly) * face_w + x + lx) * 4) as usize;
            if d + 4 > face.rgba.len() {
                continue;
            }
            for c in 0..3 {
                face.rgba[d + c] =
                    ((src[c] as u32 * a + face.rgba[d + c] as u32 * (255 - a) + 127) / 255) as u8;
            }
        }
    }
}

/// The card's own cart art, when it supplies any for this machine. The label it gets is set at
/// the size of the slot the art marks out rather than at the built-in one, because the art is
/// the thing that says where its label goes.
fn art_face(
    cart: &Cart,
    shell: &Shell,
    size: CartSize,
    art: &crate::cart_art::Art,
) -> Option<CartFace> {
    let (lw, lh) = art.label_size(size)?;
    let label = match cart.label.as_deref().and_then(|p| art::cover(p, lw, lh)) {
        Some(rgba) => rgba,
        None => sized_label(&label_text(cart), lw, lh),
    };
    // What the cart becomes on the light theme. A Game Boy or Game Boy Color cart keeps its own
    // colour there; the Advance cart is repainted in the housing's own colour — the off-white the
    // top and bottom bands are drawn in — so the cart belongs to the same device as the furniture
    // around it. Taken from `housing()` rather than written down, so the two cannot drift apart.
    let light = match cart.system() {
        slot_store::System::Gba => {
            let h = crate::slot_chrome::housing();
            Some([(h[0] * 255.0) as u8, (h[1] * 255.0) as u8, (h[2] * 255.0) as u8])
        }
        _ => None,
    };
    Some(art.face(shell, &label, lw, lh, size.face_w(), size.face_h(), light))
}

/// The generated label at any size. The built-in label is one call with its own constants; a
/// card-supplied cart asks for whatever its slot measures. Everything that decides how the type
/// sets — the padding, the ceiling on the point size, the floor under it — is the built-in
/// label's own figure scaled by how much taller or shorter this slot is, so a small slot gets
/// type in proportion rather than type that will not fit.
fn sized_label(title: &str, w: u32, h: u32) -> Vec<u8> {
    let k = h as f32 / LABEL_H as f32;
    let pad = (PAD as f32 * k) as u32;
    let bg = label_colour(title);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        rgba.extend_from_slice(&[bg[0], bg[1], bg[2], 255]);
    }

    if let Some(font) = text::label_font() {
        let layout = text::fit(
            font,
            title,
            w.saturating_sub(2 * pad) as f32,
            MAX_LINES,
            MAX_PX * k,
            (MIN_PX * k).max(6.0),
        );
        text::draw_centred(&mut rgba, w, h, &layout, ink(bg));
    }
    rgba
}

/// The Super Famicom's label plate: the dark field the reference shell prints its artwork on,
/// running nearly edge to edge across the top half of the shell, corners rounded, and the
/// machine's three dots in its bottom right corner. Drawn straight onto the shell rather than
/// rasterised apart from it, because it is the one label in the program that belongs to the
/// machine rather than to the game.
const SFC_PLATE_INK: [u8; 3] = [0x12, 0x12, 0x15];

/// Yellow, red and green — the three the reference shell carries, left to right, and the one
/// part of the label that says "Super Famicom" from across the room.
const SFC_DOTS: [[u8; 3]; 3] = [[0xf0, 0xc2, 0x20], [0xd8, 0x30, 0x40], [0x38, 0xa8, 0x50]];

fn sfc_plate(face: &mut CartFace, panel: (u32, u32, u32, u32)) {
    let (lx, ly, lw, lh) = panel;
    let (x0, y0) = (lx as i32, ly as i32);
    let (x1, y1) = (x0 + lw as i32, y0 + lh as i32);
    // A corner a twelfth of the short side: rounded the way a printed label is rounded, and
    // not the sharp rectangle a sticker dies cut to.
    let r = (lh.min(lw) / 12).max(6) as i32;
    fill_round(face, x0, y0, x1, y1, r, SFC_PLATE_INK);

    // The dots sit in the plate's own bottom-right corner, below the inner panel the game's
    // art pastes into — the corner is the plate's, and it stays the plate's even on a cart
    // whose cover runs the full inner panel.
    let r_dot = 5 * FACE_SCALE as i32;
    let cy = y1 - 7 * FACE_SCALE as i32;
    let mut cx = x1 - 10 * FACE_SCALE as i32;
    // Drawn right to left so the array reads left to right on the shell: the order on the
    // face is the order the reference carries, yellow red green.
    for ink in SFC_DOTS.iter().rev() {
        fill_disc(face, cx, cy, r_dot, *ink);
        cx -= 9 * FACE_SCALE as i32;
    }
}

/// The panel of a plate the game's own art or title pastes into: inset from the plate on
/// every side, and **shorter at the foot than the head**, because both machines' plates keep
/// their bottom band for themselves — the Super Famicom's for its three dots, the Family
/// Computer's for the blue it shows around its label.
fn plate_inner(panel: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
    let (lx, ly, lw, lh) = panel;
    let k = FACE_SCALE as u32;
    (lx + 13 * k, ly + 12 * k, lw - 26 * k, lh - 28 * k)
}

/// A title with no cover art on either plate machine: white type on nothing at all, because
/// the field behind it is already the ground those machines print their titles on. The
/// Advance's label tints itself per game — `sized_label`'s work — and on these machines the
/// machine carries the colour instead.
fn plate_text_label(title: &str, w: u32, h: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    if let Some(font) = text::label_font() {
        // Two lines at most in a band this short, and the band's own height as the em: a
        // single line may use all of it, since there is no second line to reserve for.
        let layout = text::fit(
            font,
            title,
            (w - 2 * PAD) as f32,
            2,
            h as f32 / 1.36,
            MIN_PX.max(12.0),
        );
        let cover = text::coverage(w, h, &layout);
        for (d, c) in rgba.chunks_exact_mut(4).zip(cover) {
            d[0] = 0xf5;
            d[1] = 0xf4;
            d[2] = 0xf0;
            d[3] = c;
        }
    }
    rgba
}

/// A filled rounded rectangle, straight into a face. Corner pixels test against the quarter
/// disc whose centre is `r` in from each edge; everything else is plain fill.
fn fill_round(face: &mut CartFace, x0: i32, y0: i32, x1: i32, y1: i32, r: i32, ink: [u8; 3]) {
    let (w, h) = (face.w as i32, face.h as i32);
    for y in y0.max(0)..y1.min(h) {
        for x in x0.max(0)..x1.min(w) {
            let cx = x.clamp(x0 + r, x1 - r);
            let cy = y.clamp(y0 + r, y1 - r);
            let dx = (x - cx) * (x - cx);
            let dy = (y - cy) * (y - cy);
            if dx + dy > r * r {
                continue;
            }
            put(face, x, y, ink);
        }
    }
}

/// A filled disc: the machine's three dots, and nothing else in the program paints one.
fn fill_disc(face: &mut CartFace, cx: i32, cy: i32, r: i32, ink: [u8; 3]) {
    let (w, h) = (face.w as i32, face.h as i32);
    for y in (cy - r).max(0)..(cy + r + 1).min(h) {
        for x in (cx - r).max(0)..(cx + r + 1).min(w) {
            if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r {
                put(face, x, y, ink);
            }
        }
    }
}

fn put(face: &mut CartFace, x: i32, y: i32, ink: [u8; 3]) {
    let d = ((y as u32 * face.w + x as u32) * 4) as usize;
    face.rgba[d] = ink[0];
    face.rgba[d + 1] = ink[1];
    face.rgba[d + 2] = ink[2];
}

/// The Family Computer's label: one big blue panel pasted over the shell's face, corners
/// barely rounded — a sticker's rounding, not a printed label's — and nothing else on it,
/// because the reference cart's panel carries the game's art and that is what pastes here.
/// The blue is the reference shell's own: greyer than the sky a screenshot of it would
/// suggest, which is what keeps it sitting on the yellow rather than fighting it.
const FC_PLATE_INK: [u8; 3] = [0x93, 0xa7, 0xc4];

fn fc_plate(face: &mut CartFace, panel: (u32, u32, u32, u32)) {
    let (lx, ly, lw, lh) = panel;
    let (x0, y0) = (lx as i32, ly as i32);
    let (x1, y1) = (x0 + lw as i32, y0 + lh as i32);
    let r = (lh.min(lw) / 24).max(3) as i32;
    fill_round(face, x0, y0, x1, y1, r, FC_PLATE_INK);
}

/// Hue rotation alone puts yellow and blue at very different luminance, so the ink flips
/// rather than sitting at one fixed value.
fn ink(bg: [u8; 3]) -> [u8; 3] {
    let luma = 0.2126 * bg[0] as f32 + 0.7152 * bg[1] as f32 + 0.0722 * bg[2] as f32;
    if luma > 140.0 {
        [0x1a, 0x18, 0x16]
    } else {
        [0xf4, 0xf1, 0xea]
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}
