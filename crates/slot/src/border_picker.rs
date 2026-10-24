//! The border picker's own clock: which border is in hand, what a press does to it, and how far
//! through opening and closing it is. No drawing and no card — `App` asks it what to show and
//! what to write, so the whole of its behaviour can be stated against a number of milliseconds.
//!
//! The first thing in the ring is **the machine's own shell** — the cart with no border on it,
//! which is what every cart wears until somebody picks one and what a cart goes back to when its
//! border is taken away. It is `None`, and it is a choice like any other rather than a way out:
//! a cart is either wearing a border or it is not, and both are answers worth writing down.

use slot_ui::{ease, Millis};

/// The cart grows out of the shelf and the row behind it dims; the close runs the same progress
/// back, quicker, because a panel the player is leaving should not keep them waiting.
pub const OPEN_MS: Millis = 220;
pub const CLOSE_MS: Millis = 200;
/// How long a border takes to change: the one being left goes out as the one being arrived at
/// comes in, and both travel the way the hand went.
pub const HOP_MS: Millis = 200;

/// The most borders one machine may offer, plus the machine's own shell. The picker is `Copy`
/// and `App` holds it frame to frame, so its list is a fixed array rather than a `Vec`.
pub const BORDERS_MAX: usize = 64;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Press {
    Left,
    Right,
    Keep,
    Back,
}

/// What a press asks of `App`, beyond the picker's own state.
///
/// `Write(None)` is the machine's own shell, which is written as *no line at all* — a cart put
/// back on its machine's shell leaves nothing behind it to go stale.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Nothing,
    Write(Option<u32>),
}

/// Where the row of borders stands for one frame: how far it has slid, and how open the panel is.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pose {
    /// How far the row has slid, **in items**: `0.0` at rest, one whole item the moment a press
    /// lands, easing back to nothing. The row slides the way the hand went, so a press to the
    /// right brings the next border in from the right.
    pub shift: f32,
    /// `0.0` shut on the shelf, `1.0` open at rest.
    pub openness: f32,
}

#[derive(Copy, Clone)]
struct Hop {
    dir: f32,
    started: Millis,
}

#[derive(Copy, Clone)]
struct Close {
    started: Millis,
    /// How open the panel was when the close began.
    from: f32,
}

#[derive(Copy, Clone)]
pub struct BorderPicker {
    /// Which of the ring the hand is on. An index into `items` and not a number, because the
    /// ring's own behaviour — what a press moves to, when it comes round again — is about
    /// position in the row, and the border's number is only what that position is standing for.
    seat: usize,
    /// The ring: the machine's own shell first, then the card's borders in the order they are
    /// numbered. Filled from the left; the rest of the array is never read.
    items: [Option<u32>; BORDERS_MAX],
    count: usize,
    /// When the picker was opened.
    opened: Millis,
    /// When the open's clock began, once the borders' faces were ready.
    started: Option<Millis>,
    hop: Option<Hop>,
    close: Option<Close>,
}

impl BorderPicker {
    /// Opens on `selected`, which is the border the cart already wears: the picker answers
    /// "which is this?" before it asks "which do you want?". `None` — a cart on its machine's own
    /// shell, or one nobody has chosen for — opens on that shell, which is the first thing in
    /// the ring.
    ///
    /// `numbers` are the borders the card actually holds, in the order they are numbered. One
    /// border is still a choice of two, because the machine's own shell is always the other one;
    /// a machine with no borders at all is not a question and `App` does not open the picker.
    pub fn open(selected: Option<u32>, numbers: &[u32], now: Millis) -> Self {
        let mut items = [None; BORDERS_MAX];
        let mut count = 1;
        for n in numbers.iter().take(BORDERS_MAX - 1) {
            items[count] = Some(*n);
            count += 1;
        }
        let seat = match selected {
            Some(number) => items[..count]
                .iter()
                .position(|i| *i == Some(number))
                .unwrap_or(0),
            None => 0,
        };
        BorderPicker {
            seat,
            items,
            count,
            opened: now,
            started: None,
            hop: None,
            close: None,
        }
    }

    /// Starts the open's clock. The first start is the one that counts.
    pub fn start(&mut self, now: Millis) {
        self.started.get_or_insert(now);
    }

    /// Up, but not yet moving: the borders' faces are not on the GPU yet.
    pub fn waiting(&self) -> bool {
        self.started.is_none()
    }

    /// How long ago the picker was opened.
    pub fn waited(&self, now: Millis) -> Millis {
        now.saturating_sub(self.opened)
    }

    /// The border in hand. `None` is the machine's own shell, which is what `A` writes when it
    /// is chosen — as no line at all.
    pub fn seat(&self) -> Option<u32> {
        self.items[self.seat]
    }

    /// Where in the ring that is, for a row of lamps and for the row's own drawing.
    pub fn index(&self) -> usize {
        self.seat
    }

    /// How many things are in the ring: the machine's own shell, and one per border.
    pub fn count(&self) -> usize {
        self.count
    }

    /// What sits `offset` places from the one in hand — `-1` the border to its left, `1` the one
    /// to its right — **round the ring**, so the left of the first is the last and the right of
    /// the last is the first. `None` only when the ring has fewer than three things in it and a
    /// place is being asked for twice.
    pub fn item(&self, offset: isize) -> Option<Option<u32>> {
        if self.count == 0 {
            return None;
        }
        let at = (self.seat as isize + offset).rem_euclid(self.count as isize) as usize;
        if offset != 0 && self.count < 3 && at == self.seat {
            return None;
        }
        Some(self.items[at])
    }

    /// How lit the `i`th lamp is: `1.0` for the border in hand, and — while a change is in
    /// flight — somewhere between the two it is travelling between, so the light moves along the
    /// row rather than going out and coming back on.
    pub fn dot_lit(&self, i: usize, now: Millis) -> f32 {
        let rest = || if i == self.seat { 1.0 } else { 0.0 };
        let (Some(hop), Some(q)) = (self.hop, self.progress(now)) else {
            return rest();
        };
        // Where the light is coming from: one place back the way the hand went.
        let from = (self.seat as isize - hop.dir as isize).rem_euclid(self.count as isize) as usize;
        let e = ease(q);
        if i == self.seat {
            e
        } else if i == from {
            1.0 - e
        } else {
            0.0
        }
    }

    pub fn closing(&self) -> bool {
        self.close.is_some()
    }

    /// `0.0` standing on the shelf, `1.0` open at rest.
    pub fn openness(&self, now: Millis) -> f32 {
        match (self.close, self.started) {
            (Some(Close { started, from }), _) => {
                (from - now.saturating_sub(started) as f32 / CLOSE_MS as f32).max(0.0)
            }
            (None, Some(started)) => (now.saturating_sub(started) as f32 / OPEN_MS as f32).min(1.0),
            (None, None) => 0.0,
        }
    }

    /// The close has run out, and `App` can let the picker go.
    pub fn finished(&self, now: Millis) -> bool {
        self.close.is_some() && self.openness(now) <= 0.0
    }

    /// Where the row stands, for the drawing: how far it has slid and how open the panel is.
    pub fn pose(&self, now: Millis) -> Pose {
        let shift = match (self.hop, self.progress(now)) {
            (Some(hop), Some(q)) => (1.0 - ease(q)) * hop.dir,
            _ => 0.0,
        };
        Pose {
            shift,
            openness: self.openness(now),
        }
    }

    /// What a press does. The arrows walk the ring and **the ring has no end**: a press at either
    /// end comes back round to the other, which is the only shape that makes sense of a machine
    /// with thirty-two borders. `A` writes the border in hand and closes; `B` closes and writes
    /// nothing. While the borders' faces are still being drawn the arrows answer nothing at all —
    /// there is nothing on screen to move, and a change run unseen would be confirmed by an `A`
    /// the player never saw what it was confirming.
    pub fn press(&mut self, press: Press, now: Millis) -> Outcome {
        if self.close.is_some() {
            return Outcome::Nothing;
        }
        let step = match press {
            Press::Keep => {
                self.begin_close(now);
                return Outcome::Write(self.seat());
            }
            Press::Back => {
                self.begin_close(now);
                return Outcome::Nothing;
            }
            Press::Left => -1,
            Press::Right => 1,
        };
        if self.waiting() {
            return Outcome::Nothing;
        }
        // A second press before the first has settled takes the row up where the last one's arc
        // had got to, so a run of presses reads as one movement rather than as a row that jumps
        // back to the start of its travel every time.
        let started = match self.progress(now) {
            Some(q) => now.saturating_sub(((1.0 - q) * HOP_MS as f32) as Millis),
            None => now,
        };
        self.hop = Some(Hop {
            dir: step as f32,
            started,
        });
        self.seat = (self.seat as isize + step).rem_euclid(self.count as isize) as usize;
        Outcome::Nothing
    }

    fn begin_close(&mut self, now: Millis) {
        self.close = Some(Close {
            started: now,
            from: self.openness(now),
        });
    }

    fn progress(&self, now: Millis) -> Option<f32> {
        let hop = self.hop?;
        let q = now.saturating_sub(hop.started) as f32 / HOP_MS as f32;
        (q < 1.0).then_some(q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(numbers: &[u32]) -> BorderPicker {
        BorderPicker::open(None, numbers, 0)
    }

    /// The ring is the machine's own shell and then the card's borders: a cart nobody has chosen
    /// for opens on the shell, and that is a place in the ring like any other rather than a way
    /// out of it.
    #[test]
    fn it_opens_on_the_machine_s_own_shell() {
        let p = ring(&[1, 2, 3]);
        assert_eq!(p.count(), 4, "the shell and three borders");
        assert_eq!(p.seat(), None);
        assert_eq!(p.index(), 0);
    }

    /// A cart that already wears one opens on that one, because the picker answers which it is
    /// before it asks which you want.
    #[test]
    fn it_opens_on_the_border_the_cart_wears() {
        let p = BorderPicker::open(Some(3), &[1, 2, 3], 0);
        assert_eq!(p.seat(), Some(3));
        assert_eq!(p.index(), 3);
    }

    /// A number the card no longer holds — a card whose borders were edited on a PC — opens on
    /// the shell rather than on a border that is not there.
    #[test]
    fn a_border_that_is_not_on_the_card_opens_on_the_shell() {
        let p = BorderPicker::open(Some(9), &[1, 2], 0);
        assert_eq!(p.seat(), None);
    }

    /// The ring has no end: a press left off the shell lands on the last border, and a press
    /// right off the last lands back on the shell. Thirty-two borders and no way to the first
    /// but walking back through thirty-one is not a list, it is a corridor.
    #[test]
    fn the_ring_comes_round() {
        let mut p = ring(&[1, 2, 3]);
        p.start(0);
        assert_eq!(p.press(Press::Left, 10), Outcome::Nothing);
        assert_eq!(p.seat(), Some(3), "left of the shell is the last border");
        assert_eq!(p.press(Press::Left, 300), Outcome::Nothing);
        assert_eq!(p.seat(), Some(2));
        p = ring(&[1, 2, 3]);
        p.start(0);
        p.press(Press::Right, 10);
        p.press(Press::Right, 300);
        p.press(Press::Right, 600);
        assert_eq!(p.seat(), Some(3), "three steps right from the shell");
        p.press(Press::Right, 900);
        assert_eq!(p.seat(), None, "and one more comes back to the shell");
    }

    /// `A` writes what is in hand, including the shell — which is written as no line at all, so
    /// a cart put back on its machine's own shell leaves nothing to go stale.
    #[test]
    fn a_writes_what_is_in_hand() {
        let mut p = BorderPicker::open(Some(2), &[1, 2, 3], 0);
        p.start(0);
        assert_eq!(p.press(Press::Keep, 10), Outcome::Write(Some(2)));
        let mut plain = ring(&[1]);
        plain.start(0);
        assert_eq!(plain.press(Press::Keep, 10), Outcome::Write(None));
    }

    /// `B` writes nothing, and after either of them the arrows are spent: a press on a panel that
    /// is on its way out is a press on nothing.
    #[test]
    fn b_writes_nothing_and_the_arrows_are_spent_after_it() {
        let mut p = ring(&[1, 2]);
        p.start(0);
        assert_eq!(p.press(Press::Back, 10), Outcome::Nothing);
        assert_eq!(p.press(Press::Right, 20), Outcome::Nothing);
        assert_eq!(p.seat(), None, "and the ring did not move");
    }

    /// While the faces are still being drawn the arrows do nothing, but `A` and `B` still close:
    /// a change run unseen could be confirmed by a press the player never saw the result of.
    #[test]
    fn the_arrows_wait_for_the_faces() {
        let mut p = ring(&[1, 2]);
        assert!(p.waiting());
        assert_eq!(p.press(Press::Right, 10), Outcome::Nothing);
        assert_eq!(p.seat(), None);
        assert_eq!(p.press(Press::Keep, 10), Outcome::Write(None));
    }

    /// The row slides the way the hand went and comes to rest: a press right brings the next
    /// border in from the right, which is the whole of what makes the row read as a hand moving
    /// along it rather than as pictures being swapped.
    #[test]
    fn the_row_slides_the_way_the_hand_went() {
        let mut p = ring(&[1, 2, 3]);
        p.start(0);
        p.press(Press::Right, 0);
        let early = p.pose(0).shift;
        assert!(early > 0.0, "the row starts a whole item to the right");
        assert!(p.pose(HOP_MS).shift.abs() < 0.01, "and eases to rest");
        p.press(Press::Left, 1_000);
        assert!(p.pose(1_000).shift < 0.0, "a press left starts to the left");
    }

    /// The neighbours are the ones either side, round the ring: the left of the first is the
    /// last, and a ring of two has no third place to show.
    #[test]
    fn the_neighbours_come_round_the_ring() {
        let p = BorderPicker::open(Some(1), &[1, 2, 3], 0);
        assert_eq!(p.item(-1), Some(None), "left of border 1 is the shell");
        assert_eq!(p.item(1), Some(Some(2)));
        let last = BorderPicker::open(Some(3), &[1, 2, 3], 0);
        assert_eq!(last.item(1), Some(None), "right of the last is the shell");
    }

    /// Open, then close, and the close runs out so `App` can let it go.
    #[test]
    fn it_opens_and_closes_and_is_finished() {
        let mut p = ring(&[1, 2]);
        p.start(0);
        assert_eq!(p.openness(OPEN_MS), 1.0);
        p.press(Press::Back, OPEN_MS);
        assert!(p.openness(OPEN_MS) > 0.0);
        assert!(p.finished(OPEN_MS + CLOSE_MS));
    }
}
