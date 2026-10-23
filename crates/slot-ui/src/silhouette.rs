use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use slot_store::System;

const CART_GBA_SVG: &str = include_str!("../assets/cart.svg");
const CART_SFC_SVG: &str = include_str!("../assets/cart_sfc.svg");
const CART_GB_SVG: &str = include_str!("../assets/cart_gb.svg");
const CART_GBC_SVG: &str = include_str!("../assets/cart_gbc.svg");
const CART_FC_SVG: &str = include_str!("../assets/cart_fc.svg");
const DETAIL_GBA_SVG: &str = include_str!("../assets/cart_detail.svg");
const DETAIL_SFC_SVG: &str = include_str!("../assets/cart_sfc_detail.svg");
/// Shared by class A, B and C, which is why there is one of it rather than two.
const DETAIL_GB_SVG: &str = include_str!("../assets/cart_gb_detail.svg");
const DETAIL_FC_SVG: &str = include_str!("../assets/cart_fc_detail.svg");

/// Which of the shells a machine's carts are: the Advance cart's landscape outline, the Super
/// Nintendo's, the Game Boy family's portrait one — carrying the top notch for class A and B
/// (the grey and black carts) and not for class C (Game Boy Color only), which is the physical
/// lockout — and the Family Computer's.
fn outline(system: System) -> &'static str {
    match system {
        System::Gba => CART_GBA_SVG,
        // Its own outline, and the reason is one shape detail: a Super Nintendo shell has a
        // finger groove stepped down each side, which is the most recognisable thing about it
        // and nothing the Advance's shell has. The two still share a row, a size and a label
        // box — only the outline differs.
        System::Sfc => CART_SFC_SVG,
        System::Gb => CART_GB_SVG,
        System::Gbc => CART_GBC_SVG,
        // The plainest outline here — a rounded rectangle, no grooves, no lockout — which is
        // honest: the Family Computer shell is a box the label does the talking for. It is
        // its own file rather than a shared rectangle because its proportions are its own.
        System::Fc => CART_FC_SVG,
    }
}

fn detail(system: System) -> &'static str {
    match system {
        System::Gba => DETAIL_GBA_SVG,
        // The groove across the top of the shell and the boss below it where it steps out
        // towards the connector, rather than the Advance's grip ridge and thumb notch.
        System::Sfc => DETAIL_SFC_SVG,
        System::Gb | System::Gbc => DETAIL_GB_SVG,
        // The mould lines across the foot of the shell, below the label panel.
        System::Fc => DETAIL_FC_SVG,
    }
}

fn index(system: System) -> usize {
    match system {
        // One slot per outline: the masks are keyed on what was rasterised, not on who asked.
        // The Super Nintendo used to share the Advance's slot because it shared its outline,
        // and no longer does — a shared slot over two different outlines is whichever one
        // rasterised first, which is not a bug you can see in the code.
        System::Gba => 0,
        System::Sfc => 3,
        System::Gb => 1,
        System::Gbc => 2,
        System::Fc => 4,
    }
}

/// Everything one machine's shell needs: shaped, depth-mapped, detailed, at one size.
struct Masks {
    mask: Vec<u8>,
    depth: Vec<u8>,
    detail: Vec<u8>,
}

/// Keyed on machine **and size**. A machine's shell is drawn at one size in this program and at
/// several in the tests, and a cache keyed on the machine alone kept whichever size rasterised
/// first: every face built on it afterwards was shorter than its own `w`×`h` said it was, and
/// read past its end. On the device the size never varies, so nothing crashed out there — but
/// ten cart tests died in that gap, and the cache is the reason.
fn masks(system: System, w: u32, h: u32) -> &'static Masks {
    static CACHE: OnceLock<Mutex<HashMap<(usize, u32, u32), &'static Masks>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (index(system), w, h);
    let mut map = cache.lock().unwrap();
    if let Some(&m) = map.get(&key) {
        return m;
    }
    let mask =
        rasterise_svg(outline(system), w, h).unwrap_or_else(|| vec![255; (w * h) as usize]);
    let depth = depth_map(&mask, w as usize, h as usize);
    let detail =
        rasterise_svg(detail(system), w, h).unwrap_or_else(|| vec![0; (w * h) as usize]);
    let m: &'static Masks = Box::leak(Box::new(Masks { mask, depth, detail }));
    map.insert(key, m);
    m
}

/// Coverage of the cart outline, one byte per pixel, row major.
pub fn silhouette(system: System, w: u32, h: u32) -> &'static [u8] {
    &masks(system, w, h).mask
}

/// The outline and its depth map, rasterised fresh at whatever size is asked, and cached by
/// nobody.
///
/// The shelf's cart faces go through `silhouette`, which keeps one raster per machine at the
/// cart-face size. The picker's board is a **different** size, and borrowing that cache for it is
/// how a board came out wearing a cart-sized depth map: the cache kept whichever size asked
/// first, and the face built on the shorter buffer read past its end — the panic ten cart tests
/// died in, unseen for as long as nothing ever ran. So the board rasterises its own, every time:
/// it is built once per cart at most, and resvg is fast.
pub(crate) fn shell_at(system: System, w: u32, h: u32) -> (Vec<u8>, Vec<u8>) {
    let mask =
        rasterise_svg(outline(system), w, h).unwrap_or_else(|| vec![255; (w * h) as usize]);
    let depth = depth_map(&mask, w as usize, h as usize);
    (mask, depth)
}

/// Every cart of a machine is the same shape, so the mask is rasterised once per machine and
/// multiplied into its faces. Rasterised at the face resolution so the cart stays crisp at the
/// largest size the shelf draws it.
pub(crate) fn cart_mask(system: System, w: u32, h: u32) -> &'static [u8] {
    silhouette(system, w, h)
}

/// How far inside the outline each pixel sits, in city block steps, saturating at 255. A
/// translucent shell fades from its edge inward and needs the distance, not the coverage.
pub(crate) fn cart_depth(system: System, w: u32, h: u32) -> &'static [u8] {
    &masks(system, w, h).depth
}

/// The moulded detail: the grip ridge above the label and the thumb notch below it. Shaded into
/// the shell rather than drawn in a fixed colour, so it belongs to whatever colour the cart is.
pub(crate) fn detail_mask(system: System, w: u32, h: u32) -> &'static [u8] {
    &masks(system, w, h).detail
}

/// Two pass chamfer. Everything off the edge of the buffer counts as outside, so a pixel on the
/// top row is one step in rather than unreachable.
fn depth_map(mask: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut d: Vec<u8> = mask
        .iter()
        .map(|c| if *c > 127 { 255 } else { 0 })
        .collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if d[i] == 0 {
                continue;
            }
            let up = if y == 0 { 0 } else { d[i - w] };
            let left = if x == 0 { 0 } else { d[i - 1] };
            d[i] = d[i].min(up.saturating_add(1)).min(left.saturating_add(1));
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if d[i] == 0 {
                continue;
            }
            let down = if y + 1 == h { 0 } else { d[i + w] };
            let right = if x + 1 == w { 0 } else { d[i + 1] };
            d[i] = d[i]
                .min(down.saturating_add(1))
                .min(right.saturating_add(1));
        }
    }
    d
}

fn rasterise_svg(svg: &str, w: u32, h: u32) -> Option<Vec<u8>> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
    let size = tree.size();
    let scale =
        resvg::tiny_skia::Transform::from_scale(w as f32 / size.width(), h as f32 / size.height());
    resvg::render(&tree, scale, &mut pixmap.as_mut());
    Some(pixmap.data().iter().skip(3).step_by(4).copied().collect())
}
