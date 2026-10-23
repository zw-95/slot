use slot::core_picker::{
    swap_pose, CorePicker, Outcome, Press, CLOSE_MS, HOP_MS, LIFT_MS, OPEN_MS, SLIDE_MS,
};
use slot_store::{Core, System};
use slot_ui::Millis;

/// A picker whose faces were ready the moment it opened, on the Advance's cores.
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
    // Nothing is changing: the name on it is the cart's own, and has been since it opened.
    assert!(chip.swap.is_none(), "a name is moving on a chip at rest");
}

/// The chip does not travel any more and does not go blank: it is the same chip for every core,
/// and what moves is the name written on it. Pressing right sends the old name out to the right
/// and brings the new one in from the left.
#[test]
fn right_from_mgba_sends_one_name_out_and_another_in() {
    let mut p = opened(Core::Mgba, 0);
    assert_eq!(p.press(Press::Right, 500), Outcome::Nothing);
    assert_eq!(p.seat(), Core::Gpsp);

    let mid = p.chip(500 + HOP_MS / 2);
    assert_eq!(mid.seated, Some(Core::Gpsp), "the chip went blank in flight");
    let swap = mid.swap.expect("no name is changing");
    assert_eq!(swap.from, Core::Mgba);
    assert_eq!(swap.to, Core::Gpsp);
    assert_eq!(swap.dir, 1.0, "a press to the right sends the name right");
    // Halfway through, and the two of them are the same distance either side of the middle and
    // equally solid — one leaving at exactly the rate the other arrives.
    let (out_x, out_a, in_x, in_a) = swap_pose(&swap);
    assert!(
        out_x < 0.0 && in_x > 0.0,
        "the arriving name did not come in from the right the hand pointed at"
    );
    assert!((out_a - in_a).abs() < 0.02, "they are not changing hands evenly");

    let landed = p.chip(500 + HOP_MS);
    assert!(landed.swap.is_none(), "the name is still moving");
    assert_eq!(landed.seated, Some(Core::Gpsp));
}

/// At the end of the machine's list there is nowhere to go. The chip shakes, and nothing else
/// about the picker changes. Spelled against the last of the Advance's cores rather than naming
/// one: what is at the end is a machine's business, and the Advance has grown past two.
#[test]
fn toward_the_end_of_the_list_is_refused_and_only_shakes() {
    let last = *System::Gba.cores().last().unwrap();
    let mut p = opened(last, 0);
    assert_eq!(p.press(Press::Right, 400), Outcome::Refused);
    assert_eq!(p.seat(), last);
    assert_ne!(p.chip(400).shake, 0.0, "a refusal with no shake");
    assert_eq!(p.chip(400).seated, Some(last));
    assert_eq!(p.chip(700).shake, 0.0, "the shake outlived its 300 ms");
}

/// Back the other way mid-change, the name picks up from where it is rather than jumping to the
/// start of a new travel. The one arriving is still most of the way out to the left, so it turns
/// round and goes back out the way it came — it does not slide to the middle first, and it does
/// not start again from the far side.
#[test]
fn back_mid_change_turns_the_name_round_from_where_it_is() {
    let mut p = opened(Core::Mgba, 0);
    p.press(Press::Right, 400);
    let at = 400 + HOP_MS / 4;
    // Where gpSP's name is: the one arriving, still out to the right and not yet in the middle.
    let before = swap_pose(&p.chip(at).swap.expect("no name is moving")).2;
    assert!(before > 0.0, "the arriving name had not come from the right");
    assert_eq!(p.press(Press::Left, at), Outcome::Nothing);
    assert_eq!(p.seat(), Core::Mgba);
    let turned = p.chip(at).swap.expect("the name stopped moving");
    assert_eq!(turned.dir, -1.0, "a press to the left sends the name left");
    // The same name, now the one leaving, from the same place it had got to.
    assert_eq!(
        turned.from,
        Core::Gpsp,
        "a different name is on its way out than the one that was on its way in"
    );
    assert!(
        (swap_pose(&turned).0 - before).abs() < 0.05,
        "the name jumped from {before} to {} as it turned",
        swap_pose(&turned).0
    );
    assert_eq!(p.chip(at + HOP_MS).seated, Some(Core::Mgba));
}

/// Onward again mid-change, with somewhere to go: the walk carries on. Not a refusal — there is
/// room — and so not a chip shaking over nothing.
#[test]
fn onward_mid_change_keeps_walking_and_is_not_a_refusal() {
    let cores = System::Gba.cores();
    let mut p = CorePicker::open(cores[0], cores, 0);
    p.start(0);
    p.press(Press::Right, 400);
    assert_eq!(p.press(Press::Right, 450), Outcome::Nothing);
    assert_eq!(
        p.chip(450).shake,
        0.0,
        "a refusal where there is room to walk"
    );
    assert_eq!(p.seat(), cores[2], "the walk stopped one core along");
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
fn a_change_started_soon_after_a_refusal_does_not_carry_its_shake() {
    let mut p = opened(*System::Gba.cores().last().unwrap(), 0);
    assert_eq!(p.press(Press::Right, 0), Outcome::Refused);
    assert_eq!(p.press(Press::Left, 100), Outcome::Nothing);
    assert_eq!(p.chip(100).shake, 0.0, "the refusal rode along on the change");
    assert_eq!(p.chip(150).shake, 0.0, "the refusal rode along on the change");
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
    // And once the cart is open the arrow does work, so a name is changing — with the chip under
    // it already the new core's. The chip is never blank: there is nothing in flight to stand
    // in for, the name being the only thing that travels.
    let mid = p.chip(300 + HOP_MS / 2);
    assert!(
        mid.swap.is_some(),
        "the arrow did nothing once the cart had opened"
    );
    assert_eq!(
        mid.seated,
        Some(Core::Gpsp),
        "the chip went blank while the name was changing"
    );
}

/// There is no board to fill and no window to roll any more — one chip, one name — so a machine
/// may offer as many cores as it likes and the picker walks them in the order it was given, one
/// press at a time, refusing at both ends rather than wrapping to the far side.
#[test]
fn a_long_list_is_walked_one_core_at_a_time_and_stops_at_both_ends() {
    let cores = System::Sfc.cores();
    assert!(cores.len() > 3, "this machine is too short to show it");
    let mut p = CorePicker::open(cores[0], cores, 0);
    p.start(0);
    for (i, _) in cores.iter().enumerate().skip(1) {
        assert_eq!(
            p.press(Press::Right, 100 + i as Millis * 1_000),
            Outcome::Nothing,
            "the walk stopped at {i}"
        );
        assert_eq!(p.seat(), cores[i], "the walk is out of step at {i}");
        // A card that drew its own board put a lamp per place on it, and the lit one has to be
        // the place the chip is at, not the place it started.
        assert_eq!(p.index(), i);
    }
    assert_eq!(
        p.press(Press::Right, 1_000_000),
        Outcome::Refused,
        "the arrow wrapped round to the far end"
    );
    assert_eq!(p.seat(), cores[cores.len() - 1]);
}

/// Opening on a core at the far end of a long list opens on that core, with nothing beyond it in
/// that direction — and with the rest of the machine's list behind it, which a picker that had
/// forgotten where it opened would not be able to get back to.
#[test]
fn opening_at_an_end_leaves_nothing_beyond_it_and_the_rest_behind_it() {
    let cores = System::Sfc.cores();
    let last = cores[cores.len() - 1];
    let mut p = CorePicker::open(last, cores, 0);
    p.start(0);
    assert_eq!(p.seat(), last);
    assert_eq!(p.press(Press::Right, 100), Outcome::Refused);
    assert_eq!(p.press(Press::Left, 200), Outcome::Nothing);
    assert_eq!(p.seat(), cores[cores.len() - 2]);
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
