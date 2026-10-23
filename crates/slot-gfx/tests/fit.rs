//! The three ways a machine's frame is fitted to the panel, and the arithmetic that puts it there.
//!
//! Nothing here needs a context: the fits are a rectangle, and the panel is two numbers.

use slot_gfx::{Fit, System, OUT_H, OUT_W};

/// Centred, and inside the panel: the two things every fit owes whatever else it does.
fn centred_and_inside(system: System, fit: Fit) -> (f32, f32, f32, f32) {
    let (x, y, w, h) = system.screen_rect(fit);
    assert!(w > 0.0 && h > 0.0, "{system:?} at {fit:?} is {w}x{h}");
    assert!(x >= -0.01 && y >= -0.01, "{system:?} at {fit:?} hangs off the top left");
    assert!(
        x + w <= OUT_W as f32 + 0.01 && y + h <= OUT_H as f32 + 0.01,
        "{system:?} at {fit:?} hangs off the bottom right"
    );
    assert!(
        (x - (OUT_W as f32 - w) / 2.0).abs() < 0.01 && (y - (OUT_H as f32 - h) / 2.0).abs() < 0.01,
        "{system:?} at {fit:?} is not centred"
    );
    (x, y, w, h)
}

/// The whole-number fit is the one the panel was built around, and the one `screen_base` has
/// always answered: an Advance fills it, and a Game Boy is short of it in both directions.
#[test]
fn the_integer_fit_is_the_scale_the_panel_was_built_around() {
    for system in [System::Gba, System::Gb, System::Sfc, System::Fc] {
        let (_, _, w, h) = centred_and_inside(system, Fit::Integer);
        assert_eq!(
            (w, h),
            (
                system.src_w() as f32 * system.scale() as f32,
                system.src_h() as f32 * system.scale() as f32
            ),
            "{system:?}'s integer fit is not its own whole number of source pixels"
        );
        // And it is the same rectangle the one function there was before returns.
        assert_eq!(system.screen_rect(Fit::Integer), system.screen_base());
    }
    // The Advance's own case, which is the case the panel's two numbers came from.
    let (_, _, w, h) = System::Gba.screen_rect(Fit::Integer);
    assert_eq!((w, h), (OUT_W as f32, OUT_H as f32), "240x160 at 3 is the whole panel");
}

/// Grown until one pair of edges is on the panel's, and not a pixel further: the frame's own
/// shape, at the largest size the panel allows.
#[test]
fn the_fill_fit_puts_one_pair_of_edges_on_the_panel() {
    for system in [System::Gba, System::Gb, System::Sfc, System::Fc] {
        let (_, _, w, h) = centred_and_inside(system, Fit::Fill);
        let (sw, sh) = (system.src_w() as f32, system.src_h() as f32);
        // The picture's own shape, which is the whole of what this fit keeps.
        assert!(
            (w / sw - h / sh).abs() < 0.001,
            "{system:?} at the fill fit is {w}x{h}, which is not {sw}x{sh}'s shape"
        );
        // One pair of edges on the panel's, and the other pair inside them.
        assert!(
            (w - OUT_W as f32).abs() < 0.01 || (h - OUT_H as f32).abs() < 0.01,
            "{system:?} at the fill fit touches neither edge: {w}x{h}"
        );
        // And never smaller than the whole-number one: filling the panel is the point, and a fit
        // that came out smaller than the integer scale would be a step backwards wearing a new
        // name.
        let (_, _, iw, ih) = system.screen_rect(Fit::Integer);
        assert!(
            w >= iw - 0.01 && h >= ih - 0.01,
            "{system:?}'s fill fit is smaller than its integer one"
        );
    }
    // A Game Boy is shorter than the panel is shaped, so it is the height that gives out first:
    // 160x144 at 3.33 is 533x480, and the sides are what is left over.
    let (_, _, w, h) = System::Gb.screen_rect(Fit::Fill);
    assert!((h - OUT_H as f32).abs() < 0.01, "a Game Boy's fill fit is capped by height");
    assert!(w < OUT_W as f32, "and is letterboxed down the sides");
}

/// The panel's own shape, which is the one fit that changes the picture's proportions.
#[test]
fn the_full_fit_is_the_panels_own_shape() {
    for system in [System::Gba, System::Gb, System::Sfc, System::Fc] {
        let (x, y, w, h) = centred_and_inside(system, Fit::Fill);
        assert!((x, y) != (0.0, 0.0) || (w, h) == (OUT_W as f32, OUT_H as f32));
        let (x, y, w, h) = system.screen_rect(Fit::Full);
        assert_eq!(
            (x, y, w, h),
            (0.0, 0.0, OUT_W as f32, OUT_H as f32),
            "{system:?} stretched is not the panel"
        );
    }
}

/// A ring of three, walked with one key: the last comes back to the first, because a preference
/// stepped through one press at a time has to be gettable back to the front of without counting.
#[test]
fn the_three_fits_are_a_ring() {
    let mut f = Fit::Integer;
    let mut seen = vec![f];
    for _ in 0..2 {
        f = f.next();
        seen.push(f);
    }
    assert_eq!(seen.len(), 3, "the ring is three long, not {seen:?}");
    assert!(seen.iter().all(|f| Fit::ALL.contains(f)));
    assert_eq!(
        f.next(),
        Fit::Integer,
        "the walk does not come back to the front"
    );
}

/// The lattice is one RGB triad per source pixel, which is a statement about a whole number of
/// panel pixels under each of them — so it is drawn at one fit and stands down at the other two.
#[test]
fn the_lattice_is_only_whole_at_the_integer_fit() {
    assert!(System::whole(Fit::Integer));
    assert!(!System::whole(Fit::Fill));
    assert!(!System::whole(Fit::Full));
}
