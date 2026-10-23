//! The core picker's own clock: how open the cart is, which core is in hand, and what a press
//! does to either. No drawing and no card — `App` asks it what to show and what to write, and
//! the whole of its behaviour can be stated against a number of milliseconds.

use slot_store::Core;
use slot_ui::{ease, Millis, Refusal};

/// The front half slides off the back, then lifts away; the close runs the same progress back,
/// quicker. `slot_ui::SLIDE_SHARE` is the slide's share of the open, held to these by a test.
pub const SLIDE_MS: Millis = 160;
pub const LIFT_MS: Millis = 260;
pub const OPEN_MS: Millis = SLIDE_MS + LIFT_MS;
pub const CLOSE_MS: Millis = 320;
/// How long a core's name takes to change: the one the player is leaving goes out as the one
/// they are coming to arrives, and the chip under both of them stays where it is.
pub const HOP_MS: Millis = 180;

/// The most cores one machine may offer. The picker is `Copy` and `App` holds it frame to frame,
/// so its list is a fixed array rather than a `Vec`. Sixteen is far past any machine today; a
/// machine with more is a machine `System::cores` has to answer for.
pub const CORES_MAX: usize = 16;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Press {
    Left,
    Right,
    Keep,
    Back,
}

/// What a press asks of `App`, beyond the picker's own state.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Nothing,
    Refused,
    Write(Core),
}

/// A core's name changing on the chip: the one leaving, the one arriving, which way they are
/// going, and how far through the change they are.
///
/// `dir` is the way the hand moved — `1.0` for a press to the right. **The name being left goes
/// that way and the one being arrived at comes from the other side**, so the words travel with
/// the hand rather than against it: a press to the left sends the old name out to the left.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Swap {
    pub from: Core,
    pub to: Core,
    pub dir: f32,
    /// 0.0 the moment the press lands, 1.0 when the new name has settled in the middle.
    pub q: f32,
}

/// The chip's pose for one frame, which is now mostly a matter of what its name is doing.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Chip {
    /// The core the chip stands for. Always one: the chip is the same chip whichever core is in
    /// hand, and it is the name on it that changes, so there is no moment when it is blank.
    pub seated: Option<Core>,
    /// A name changing on it, if one is.
    pub swap: Option<Swap>,
    /// Panel pixels the refusal puts on its x. Both the chip and its name carry it, so the
    /// whole thing shakes rather than the words sliding out from under a still chip.
    pub shake: f32,
}

#[derive(Copy, Clone)]
struct Hop {
    /// The core whose name is leaving. Kept as a core rather than as a position, because there
    /// are no positions any more: the chip does not travel, the name does.
    from: Core,
    /// Which way the hand moved, and so which way the leaving name goes.
    dir: f32,
    started: Millis,
}

#[derive(Copy, Clone)]
struct Close {
    started: Millis,
    /// How open the cart was when the close began.
    from: f32,
}

#[derive(Copy, Clone)]
pub struct CorePicker {
    /// Which of the machine's cores the chip is on. An index and not a `Core`, because the whole
    /// of the picker's own behaviour — which core is in hand, which way a press moves it, when it
    /// is at an end — is about position in the machine's list, and the core is only what that
    /// position is standing for.
    seat: usize,
    /// The cores the machine offers, in the order they are travelled, and how many there are.
    /// Filled from the left; the rest of the array is never read.
    cores: [Core; CORES_MAX],
    count: usize,
    /// When START was pressed.
    opened: Millis,
    /// When the open's clock began, once the cart's faces were ready.
    started: Option<Millis>,
    hop: Option<Hop>,
    close: Option<Close>,
    /// The chip's own. The shelf does not shake while the picker is up.
    refusal: Option<Refusal>,
}

impl CorePicker {
    /// Opens on `seat`, which is the core the cart already uses: the board answers "which is
    /// this?" before it asks "which do you want?". `cores` is the machine's own list, in the
    /// order it is travelled, and must hold at least two — one core is not a question, and `App`
    /// does not open the picker for it.
    ///
    /// A `seat` the list does not hold opens on the first instead, which is that machine's
    /// default: the same answer `core_for` would have given for a cart with no line.
    pub fn open(seat: Core, cores: &[Core], now: Millis) -> Self {
        let count = cores.len().min(CORES_MAX).max(1);
        let mut list = [Core::Mgba; CORES_MAX];
        list[..count].copy_from_slice(&cores[..count]);
        let seat = cores
            .iter()
            .position(|c| *c == seat)
            .unwrap_or(0)
            .min(count - 1);
        CorePicker {
            seat,
            cores: list,
            count,
            opened: now,
            started: None,
            hop: None,
            close: None,
            refusal: None,
        }
    }

    /// Starts the open's clock. The first start is the one that counts.
    pub fn start(&mut self, now: Millis) {
        self.started.get_or_insert(now);
    }

    /// Up, but not yet moving: the cart's faces are not on the GPU yet.
    pub fn waiting(&self) -> bool {
        self.started.is_none()
    }

    /// How long ago START was pressed.
    pub fn waited(&self, now: Millis) -> Millis {
        now.saturating_sub(self.opened)
    }

    /// Where the chip is, or where it is going: the core `A` writes.
    pub fn seat(&self) -> Core {
        self.cores[self.seat]
    }

    /// How far along the machine's own list that core is. A card that drew its own board drew a
    /// lamp per position on it, and one of them has to be lit for the one in hand.
    pub fn index(&self) -> usize {
        self.seat
    }

    pub fn closing(&self) -> bool {
        self.close.is_some()
    }

    /// 0.0 is the cart standing on the shelf, 1.0 open at rest. Linear progress through both
    /// beats, which `slot_ui::slide_of` and `lift_of` ease on their own shares.
    pub fn openness(&self, now: Millis) -> f32 {
        match (self.close, self.started) {
            // Reversed from wherever the open had got to, at the close's speed.
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

    /// What a press does to the name or the lid, and what it asks of `App`. While the cart still
    /// waits on its faces the arrows do nothing: the chip is not on screen, and a change run
    /// unseen would open the cart with another core's name already on it, for an `A` to write a
    /// core the player never saw chosen. `A` and `B` still close it.
    ///
    /// The arrows step one core along the machine's list and do not wrap: at either end the
    /// board refuses rather than sending the player round to the far core, because arriving at
    /// the other end would read as a long way round rather than as "there is nothing over there".
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
        let target = self.seat as isize + step;
        if target < 0 || target >= self.count as isize {
            self.refusal = Some(Refusal::started(now));
            return Outcome::Refused;
        }
        let target = target as usize;
        if self.waiting() {
            return Outcome::Nothing;
        }
        // The name the player is leaving is the one on the chip now, whatever the last change
        // was doing: a second press before the first has settled takes the name it was arriving
        // at and sends that one on its way, which is what makes a run of presses read as one
        // movement rather than as names shoving each other out of the middle.
        let from = self.seat();
        let started = match self.progress(now) {
            // Already mid-change: pick the new one up where the old one's arc had got to, so
            // the names do not jump back to the start of their travel.
            Some(q) => now.saturating_sub(((1.0 - q) * HOP_MS as f32) as Millis),
            None => now,
        };
        self.hop = Some(Hop {
            from,
            dir: step as f32,
            started,
        });
        self.seat = target;
        // A change is happening now, so a shake left over from an earlier refusal is not this
        // chip's any more.
        self.refusal = None;
        Outcome::Nothing
    }

    pub fn chip(&self, now: Millis) -> Chip {
        Chip {
            seated: Some(self.seat()),
            swap: self.progress(now).and_then(|q| {
                self.hop.map(|hop| Swap {
                    from: hop.from,
                    to: self.seat(),
                    dir: hop.dir,
                    q,
                })
            }),
            shake: self.refusal.map_or(0.0, |r| r.offset(now)),
        }
    }

    fn begin_close(&mut self, now: Millis) {
        self.close = Some(Close {
            started: now,
            from: self.openness(now),
        });
    }

    /// How far through a change of name, or `None` when there is none in flight.
    fn progress(&self, now: Millis) -> Option<f32> {
        let hop = self.hop?;
        let q = now.saturating_sub(hop.started) as f32 / HOP_MS as f32;
        (q < 1.0).then_some(q)
    }
}

/// How far a name has travelled at `q`, and how solid it is: `ease`d, because a name sliding
/// out linearly reads as a thing being pulled rather than as type being set.
///
/// `out` is the name on its way off, `in` the one arriving. Both are given as a fraction of the
/// travel and an alpha, so the caller only has to know which way the hand went.
pub fn swap_pose(swap: &Swap) -> (f32, f32, f32, f32) {
    let e = ease(swap.q);
    (
        swap.dir * e,                  // leaving: towards the side the hand went
        1.0 - e,                       // and fading out as it goes
        -swap.dir * (1.0 - e),         // arriving: from the other side
        e,                             // and coming up to full as it settles
    )
}
