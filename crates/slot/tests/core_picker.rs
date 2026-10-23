use slot::core_picker::{
    CorePicker, Outcome, Press, CLOSE_MS, HOP_MS, LIFT_MS, OPEN_MS, SLIDE_MS, SOCKETS,
};
use slot_store::{Core, System};
use slot_ui::Millis;

/// A picker whose faces were ready the moment it opened, on the Advance's two sockets — which
/// is what every one of these tests is about until the last, which is about three.
fn opened(seat: Core, at: Millis) -> CorePicker {
    let mut p = CorePicker::open(seat, System::Gba.cores(), at);
    p.start(at);
    p
}

/// Linear, and held to it a quarter of the way in: smootherstep passes through a half at the
/// midpoint too, so the midpoint alone cannot tell the two apart.
#[test]
fn it_opens_over_the_open_time_and_then_rests() {
    let p = opened(Core::Mgba, 1000);
    assert_eq!(p.openness(1000), 0.0);
    assert_eq!(p.openness(1000 + OPEN_MS / 4), 0.25);
    assert_eq!(p.openness(1000 + OPEN_MS), 1.0);
    assert_eq!(p.openness(1000 + 10 * OPEN_MS), 1.0);
}

/// The chip is already where the cart runs, so the board says what is true before it asks.
#[test]
fn the_chip_starts_seated_in_the_carts_own_core() {
    let chip = opened(Core::Gpsp, 0).chip(0);
    assert_eq!(chip.seated, Some(Core::Gpsp));
    assert_eq!((chip.across, chip.lift, chip.tip), (1.0, 0.0, 0.0));
}

#[test]
fn right_from_mgba_hops_blank_and_lands_named_in_gpsp() {
    let mut p = opened(Core::Mgba, 0);
    assert_eq!(p.press(Press::Right, 500), Outcome::Nothing);
    assert_eq!(p.seat(), Core::Gpsp);

    let mid = p.chip(500 + HOP_MS / 2);
    assert_eq!(mid.seated, None, "the chip wears a name in flight");
    assert!((mid.across - 0.5).abs() < 0.01, "across {}", mid.across);
    assert!((mid.lift - 1.0).abs() < 0.01, "lift {}", mid.lift);
    assert!(mid.tip > 0.0, "a chip moving right should lean right");

    let landed = p.chip(500 + HOP_MS);
    assert_eq!(landed.seated, Some(Core::Gpsp));
    assert_eq!((landed.across, landed.lift), (1.0, 0.0));
}

/// Toward the socket it is already in there is nowhere to go. The chip shakes, and nothing
/// else about the picker changes.
#[test]
fn toward_the_socket_it_is_in_is_refused_and_only_shakes() {
    let mut p = opened(Core::Gpsp, 0);
    assert_eq!(p.press(Press::Right, 400), Outcome::Refused);
    assert_eq!(p.seat(), Core::Gpsp);
    assert_ne!(p.chip(400).shake, 0.0, "a refusal with no shake");
    assert_eq!(p.chip(400).seated, Some(Core::Gpsp));
    assert_eq!(p.chip(700).shake, 0.0, "the shake outlived its 300 ms");
}

/// Back toward the socket it is leaving, the chip retraces its arc from where it is rather
/// than jumping to the start of a new one.
#[test]
fn back_mid_hop_turns_the_chip_round_from_where_it_is() {
    let mut p = opened(Core::Mgba, 0);
    p.press(Press::Right, 400);
    let at = 400 + HOP_MS / 4;
    let before = p.chip(at).across;
    assert_eq!(p.press(Press::Left, at), Outcome::Nothing);
    assert_eq!(p.seat(), Core::Mgba);
    assert!(
        (p.chip(at).across - before).abs() < 0.02,
        "the chip jumped from {before} to {} as it turned",
        p.chip(at).across
    );
    assert_eq!(p.chip(at + HOP_MS).seated, Some(Core::Mgba));
}

#[test]
fn onward_mid_hop_does_nothing_and_is_not_a_refusal() {
    let mut p = opened(Core::Mgba, 0);
    p.press(Press::Right, 400);
    assert_eq!(p.press(Press::Right, 450), Outcome::Nothing);
    assert_eq!(p.chip(450).shake, 0.0);
    assert_eq!(p.seat(), Core::Gpsp);
}

#[test]
fn keep_writes_where_the_chip_is_heading_and_closes() {
    let mut p = opened(Core::Mgba, 0);
    p.press(Press::Right, 400);
    assert_eq!(p.press(Press::Keep, 450), Outcome::Write(Core::Gpsp));
    assert!(p.closing());
    assert!(!p.finished(450));
    assert!(p.finished(450 + CLOSE_MS));
}

#[test]
fn back_closes_without_a_write() {
    let mut p = opened(Core::Mgba, 0);
    assert_eq!(p.press(Press::Back, 400), Outcome::Nothing);
    assert!(p.closing());
    assert!(p.finished(400 + CLOSE_MS));
}

/// A close that begins while the lid is still lifting starts from where the lid is. Starting
/// from rest would snap it up before bringing it back down.
#[test]
fn a_close_during_the_lift_reverses_from_where_the_lid_had_got_to() {
    let mut p = opened(Core::Mgba, 0);
    let at = OPEN_MS / 2;
    let open = p.openness(at);
    p.press(Press::Back, at);
    assert!(
        (p.openness(at) - open).abs() < 1e-6,
        "the lid snapped as the close began"
    );
    assert!(
        p.finished(at + CLOSE_MS),
        "a partial close outlasted a whole one"
    );
}

/// A refusal decays on its own clock and must not survive into a hop that actually happens: two
/// things shaking at once reads as two separate failures.
#[test]
fn a_hop_started_soon_after_a_refusal_does_not_carry_its_shake() {
    let mut p = opened(Core::Gpsp, 0);
    assert_eq!(p.press(Press::Right, 0), Outcome::Refused);
    assert_eq!(p.press(Press::Left, 100), Outcome::Nothing);
    assert_eq!(p.chip(100).shake, 0.0, "the refusal rode along on the hop");
    assert_eq!(p.chip(150).shake, 0.0, "the refusal rode along on the hop");
}

#[test]
fn presses_during_the_close_do_nothing() {
    let mut p = opened(Core::Mgba, 0);
    p.press(Press::Back, 400);
    assert_eq!(p.press(Press::Right, 420), Outcome::Nothing);
    assert_eq!(p.press(Press::Keep, 430), Outcome::Nothing);
    assert_eq!(p.seat(), Core::Mgba);
}

#[test]
fn the_open_is_a_slide_then_a_lift_and_the_board_agrees() {
    assert_eq!(OPEN_MS, SLIDE_MS + LIFT_MS);
    assert!((slot_ui::SLIDE_SHARE - SLIDE_MS as f32 / OPEN_MS as f32).abs() < 1e-6);
}

/// Opened before its faces are uploaded, the cart stands on the shelf until it is started.
#[test]
fn a_picker_waits_until_it_is_started() {
    let mut p = CorePicker::open(Core::Mgba, System::Gba.cores(), 100);
    assert!(p.waiting());
    assert_eq!(p.openness(5_000), 0.0);
    assert_eq!(p.waited(350), 250);
    p.start(400);
    assert!(!p.waiting());
    assert_eq!(p.openness(400), 0.0);
    assert_eq!(p.openness(400 + OPEN_MS), 1.0);
}

#[test]
fn a_second_start_keeps_the_first() {
    let mut p = CorePicker::open(Core::Mgba, System::Gba.cores(), 0);
    p.start(300);
    p.start(500);
    assert_eq!(p.openness(300 + OPEN_MS), 1.0);
}

/// Backing out before anything moved has nothing to put back.
#[test]
fn backing_out_while_waiting_finishes_at_once() {
    let mut p = CorePicker::open(Core::Mgba, System::Gba.cores(), 0);
    p.press(Press::Back, 50);
    assert!(p.finished(50));
}

/// The close runs the progress backwards at the close's speed, from wherever it was, and
/// linearly: a quarter of the close in, a quarter of the progress is gone. At the midpoint an
/// eased close would read a half as well.
#[test]
fn a_close_from_rest_takes_the_close_time() {
    let mut p = opened(Core::Mgba, 0);
    p.press(Press::Back, OPEN_MS);
    assert_eq!(p.openness(OPEN_MS + CLOSE_MS / 4), 0.75);
    assert!(p.finished(OPEN_MS + CLOSE_MS));
}

/// Until the cart opens the chip is not on screen, so the arrows have nothing to move: a hop run
/// unseen would open the cart with the chip already in the other socket for an `A` to write, and
/// a refusal would shake a chip nobody can see.
#[test]
fn the_arrows_do_nothing_while_the_picker_waits() {
    let mut p = CorePicker::open(Core::Mgba, System::Gba.cores(), 0);
    assert_eq!(p.press(Press::Right, 100), Outcome::Nothing);
    assert_eq!(p.seat(), Core::Mgba, "a hop started while the cart waited");
    let chip = p.chip(100);
    assert_eq!((chip.seated, chip.shake), (Some(Core::Mgba), 0.0));
    assert_eq!(
        p.press(Press::Left, 120),
        Outcome::Nothing,
        "toward its own socket while the cart waited was refused"
    );
    assert_eq!(p.chip(120).shake, 0.0);

    p.start(200);
    assert_eq!(p.press(Press::Right, 300), Outcome::Nothing);
    assert_eq!(
        p.seat(),
        Core::Gpsp,
        "the arrow did nothing once the cart had opened"
    );
    assert_eq!(
        p.chip(300 + HOP_MS / 2).seated,
        None,
        "no hop once the cart had opened"
    );
}

/// More than two sockets, which is the whole of what a machine with three to choose between
/// changes: `across` is a fraction of the run of sockets, so the middle one is halfway along
/// it, and both ends are still ends — the arrow refuses rather than wrapping to the far side.
#[test]
fn a_run_of_three_puts_the_middle_socket_in_the_middle() {
    // Spelled out rather than taken from a machine, because a machine's list can grow past what
    // the board holds and this is about three that fit.
    let cores = [
        Core::Supafaust,
        Core::Snes9x2010,
        Core::Snes9x2005Plus,
    ];
    assert_eq!(cores.len(), SOCKETS, "this list fills the board");
    let mut p = CorePicker::open(Core::Supafaust, &cores, 0);
    p.start(0);
    assert_eq!(p.chip(0).across, 0.0);

    p.press(Press::Right, 100);
    assert_eq!(p.seat(), cores[1]);
    assert_eq!(p.chip(100 + HOP_MS).across, 0.5, "the middle is halfway");

    p.press(Press::Right, 200);
    assert_eq!(p.seat(), cores[2]);
    assert_eq!(p.chip(200 + HOP_MS).across, 1.0);

    assert_eq!(
        p.press(Press::Right, 300),
        Outcome::Refused,
        "the arrow wrapped round to the far socket"
    );
    assert_eq!(p.seat(), cores[2]);
}

/// A machine may offer more cores than the board has sockets for. The board shows a window of
/// them and rolls it along: the chip walks to the end of what is showing, and the next press
/// brings the next core onto the board without the chip leaving the socket it is standing on.
#[test]
fn a_list_longer_than_the_board_rolls_under_the_chip() {
    let cores = System::Sfc.cores();
    assert!(cores.len() > SOCKETS, "this machine fits, so nothing rolls");
    let mut p = CorePicker::open(cores[0], cores, 0);
    p.start(0);
    assert_eq!(p.window(), &cores[..SOCKETS]);
    assert_eq!(p.chip(0).visible, SOCKETS);

    // To the far end of what is showing.
    p.press(Press::Right, 100);
    assert_eq!(p.chip(100 + HOP_MS).across, 0.5);
    p.press(Press::Right, 200);
    assert_eq!(p.chip(200 + HOP_MS).across, 1.0);
    assert_eq!(p.window(), &cores[..SOCKETS]);

    // One more, and it is the window that moves: the chip is still on the right-hand socket,
    // and the core under it is the next one along.
    p.press(Press::Right, 300);
    assert_eq!(p.seat(), cores[3]);
    assert_eq!(p.chip(300 + HOP_MS).across, 1.0, "the chip left the board");
    assert_eq!(p.window(), &cores[1..1 + SOCKETS]);
    // It hopped on the spot, and upright: a window rolling under it is not a direction.
    assert_eq!(p.chip(300 + HOP_MS / 2).tip, 0.0);
}

/// Opening on a core at the far end of a long list has to bring the board to it, or the chip
/// would be drawn standing on a socket that is not showing the core it is standing for.
#[test]
fn opening_on_a_core_at_the_end_brings_the_window_to_it() {
    let cores = System::Sfc.cores();
    let last = cores[cores.len() - 1];
    let mut p = CorePicker::open(last, cores, 0);
    p.start(0);
    assert_eq!(p.seat(), last);
    assert_eq!(p.window().last().copied(), Some(last));
    // And it is the end: nothing to the right of it.
    assert_eq!(p.press(Press::Right, 100), Outcome::Refused);
}

/// A machine with one core is not asked: there is nowhere for the chip to hop to, and `App`
/// does not open the board for it at all.
#[test]
fn one_core_is_not_a_choice() {
    // Spelled out rather than taken off a machine: every machine on the card offers more than
    // one now, so the one-socket board has to be made by hand. It is still a shape a machine
    // could have — the Game Boy had it until this week — and the picker still has to refuse
    // rather than wrap.
    let cores = [Core::Mgba];
    assert_eq!(cores.len(), 1);
    let mut p = CorePicker::open(cores[0], &cores, 0);
    p.start(0);
    assert_eq!(p.press(Press::Right, 100), Outcome::Refused);
    assert_eq!(p.press(Press::Left, 200), Outcome::Refused);
    assert_eq!(p.seat(), cores[0]);
}
