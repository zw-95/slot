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

/// The list is a **ring**: off the end comes back round to the other end, and the name travels
/// the same way it does for any other step. Spelled against the last of the Advance's cores
/// rather than naming one: what is at the end is a machine's business, and the Advance has grown
/// past two.
///
/// It was refused here until 2026-10-24, and a wall is the wrong shape for a list of thirteen the
/// board shows one of at a time: getting from the last core to the first meant walking back
/// through twelve, which is not "there is nothing over there" — it is a long way round.
#[test]
fn off_the_end_of_the_list_comes_back_round() {
    let cores = System::Gba.cores();
    let last = *cores.last().unwrap();
    let mut p = opened(last, 0);
    assert_eq!(p.press(Press::Right, 400), Outcome::Nothing);
    assert_eq!(p.seat(), cores[0], "the ring did not come back to the front");
    assert_eq!(p.index(), 0);
    // And the name travels, so it reads as a movement and not as a jump: the one being left is
    // the old core and the one arriving is the new one, from the side the hand went.
    let moving = p.chip(400 + HOP_MS / 2).swap.expect("no name is travelling");
    assert_eq!((moving.from, moving.to), (last, cores[0]));
    assert_eq!(moving.dir, 1.0, "a press to the right brings the name in from the right");
    assert_eq!(p.chip(400 + HOP_MS).seated, Some(cores[0]));
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

/// Onward again mid-change: the walk carries on from where the name had got to.
#[test]
fn onward_mid_change_keeps_walking() {
    let cores = System::Gba.cores();
    let mut p = CorePicker::open(cores[0], cores, 0);
    p.start(0);
    p.press(Press::Right, 400);
    assert_eq!(p.press(Press::Right, 450), Outcome::Nothing);
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

/// A change started a moment after another one still runs from where the first had got to, and
/// neither of them leaves anything behind for the next: the chip has one name on it at a time and
/// one travel at a time, so a run of presses reads as one movement.
#[test]
fn a_change_started_soon_after_another_picks_up_where_that_one_was() {
    let cores = System::Gba.cores();
    let mut p = opened(cores[0], 0);
    assert_eq!(p.press(Press::Right, 0), Outcome::Nothing);
    // Only a quarter of the way through the first travel, and already turning round: the name
    // goes back the way it came rather than starting again from the far side.
    assert_eq!(p.press(Press::Left, HOP_MS / 4), Outcome::Nothing);
    assert_eq!(p.seat(), cores[0]);
    let turned = p.chip(HOP_MS / 4).swap.expect("the name stopped moving");
    assert_eq!(turned.dir, -1.0);
    assert!(p.chip(HOP_MS / 2).swap.is_some(), "the turn did not travel");
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
    assert_eq!(chip.seated, Some(Core::Mgba));
    assert_eq!(
        p.press(Press::Left, 120),
        Outcome::Nothing,
        "the arrows moved a chip nobody can see"
    );

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
/// press at a time, and round again from the last to the first.
#[test]
fn a_long_list_is_walked_one_core_at_a_time_and_round_again() {
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
    // And off the end, **back round to the front**: a ring, which is what gets a player from the
    // last of thirteen to the first without walking the eleven between them.
    assert_eq!(
        p.press(Press::Right, 1_000_000),
        Outcome::Nothing,
        "the arrow at the end of the list answered nothing"
    );
    assert_eq!(p.seat(), cores[0], "the walk did not come back round");
    assert_eq!(p.index(), 0);
    // And the other way off the front, to the back.
    assert_eq!(p.press(Press::Left, 1_000_001), Outcome::Nothing);
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
    // Beyond the last is the first, and behind the last is the one before it: both ends of a ring
    // are somewhere, which is the difference between a ring and a wall.
    assert_eq!(p.press(Press::Right, 100), Outcome::Nothing);
    assert_eq!(p.seat(), cores[0]);
    assert_eq!(p.press(Press::Left, 200), Outcome::Nothing);
    assert_eq!(p.seat(), cores[cores.len() - 1]);
    assert_eq!(p.press(Press::Left, 300), Outcome::Nothing);
    assert_eq!(p.seat(), cores[cores.len() - 2]);
}

/// A machine with one core is not asked: there is nowhere for the chip to hop to, and `App`
/// does not open the board for it at all.
#[test]
fn one_core_is_not_a_choice() {
    // Spelled out rather than taken off a machine: every machine on the card offers more than
    // one now, so the one-socket board has to be made by hand. It is still a shape a machine
    // could have — the Game Boy had it until this week — and a ring of one is the one core there
    // is, pressed at from either side and never moved.
    let cores = [Core::Mgba];
    assert_eq!(cores.len(), 1);
    let mut p = CorePicker::open(cores[0], &cores, 0);
    p.start(0);
    assert_eq!(p.press(Press::Right, 100), Outcome::Nothing);
    assert_eq!(p.press(Press::Left, 200), Outcome::Nothing);
    assert_eq!(p.seat(), cores[0]);
    assert_eq!(p.index(), 0);
}
