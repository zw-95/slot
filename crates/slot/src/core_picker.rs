//! The core picker's own clock: how open the cart is, where the chip is, and what a press does
//! to either. No drawing and no card — `App` asks it what to show and what to write, and the
//! whole of its behaviour can be stated against a number of milliseconds.

use slot_store::Core;
use slot_ui::{ease, Millis, Refusal, CHIP_TIP};

/// The front half slides off the back, then lifts away; the close runs the same progress back,
/// quicker. `slot_ui::SLIDE_SHARE` is the slide's share of the open, held to these by a test.
pub const SLIDE_MS: Millis = 160;
pub const LIFT_MS: Millis = 260;
pub const OPEN_MS: Millis = SLIDE_MS + LIFT_MS;
pub const CLOSE_MS: Millis = 320;
pub const HOP_MS: Millis = 180;

/// How many sockets the board has room for, and how many it draws however long the machine's
/// list is. Three, because that is what the board art has space for: a fourth socket would sit
/// on plastic that was drawn for three, and a name small enough to fit in the space left would
/// be too small to read.
///
/// A machine offering more than this rolls a window of `SOCKETS` along its list instead — the
/// chip walks to the end of what is on the board and the sockets under it change, which is how
/// a list longer than the board is travelled without the board changing.
pub const SOCKETS: usize = 3;

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

/// The chip's pose for one frame.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Chip {
    /// 0.0 over the leftmost socket on the board, 1.0 over the rightmost. The sockets on it are
    /// evenly spaced, so a fraction of that run lands either on one of them or in the air
    /// between two.
    pub across: f32,
    /// Which socket that is, and how many the board is showing. Given rather than measured back
    /// off `across`: rounding a fraction that came from an even spacing is how a chip in flight
    /// would be reported as sitting in the socket it is only passing over.
    pub index: usize,
    pub visible: usize,
    /// 0.0 seated, 1.0 at the top of the hop.
    pub lift: f32,
    /// Radians, clockwise, leaning into the direction of travel. Nothing to lean into when the
    /// hop is a window moving under a chip that is staying put, so that case is upright.
    pub tip: f32,
    /// The socket it sits in, and so the name it wears. `None` in flight.
    pub seated: Option<Core>,
    /// Panel pixels the refusal puts on its x.
    pub shake: f32,
}

#[derive(Copy, Clone)]
struct Hop {
    /// Where it left from, as a position on the board rather than as a core: the arc is drawn
    /// from a socket to a socket, and a window rolling under a chip leaves it where it was, so
    /// the same hop can begin and end on the same socket.
    from: usize,
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
    /// of the picker's own behaviour — where the chip is, which way a press moves it, when it is
    /// at an end — is about position on the board, and the core is only what that position is
    /// standing for.
    seat: usize,
    /// The cores the machine offers, in the order they are travelled, and how many there are.
    /// Filled from the left; the rest of the array is never read.
    cores: [Core; CORES_MAX],
    count: usize,
    /// The first of the machine's cores that is on the board. Zero while the whole list fits;
    /// on a longer list it moves so the chip's socket always has a core under it.
    window: usize,
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
    /// order it is travelled, and must hold at least two — a board with one socket is not a
    /// question, and `App` does not open the picker for it.
    ///
    /// A `seat` the list does not hold opens on the first socket instead, which is that
    /// machine's default: the same answer `core_for` would have given for a cart with no line.
    pub fn open(seat: Core, cores: &[Core], now: Millis) -> Self {
        let count = cores.len().min(CORES_MAX).max(1);
        let mut list = [Core::Mgba; CORES_MAX];
        list[..count].copy_from_slice(&cores[..count]);
        let seat = cores
            .iter()
            .position(|c| *c == seat)
            .unwrap_or(0)
            .min(count - 1);
        let mut picker = CorePicker {
            seat,
            cores: list,
            count,
            window: 0,
            opened: now,
            started: None,
            hop: None,
            close: None,
            refusal: None,
        };
        // Opening on a cart's own core can put the chip anywhere in a long list, and the board
        // has to be showing it. Not on the last press, but now, before anything has been drawn.
        picker.follow();
        picker
    }

    /// How many sockets the board is showing: the whole list if it fits, `SOCKETS` if not.
    pub fn visible(&self) -> usize {
        self.count.min(SOCKETS)
    }

    /// The cores on the board, left to right — a window of the machine's list, and the whole of
    /// it on a machine short enough to fit. `App` asks for this to draw the sockets, and for the
    /// chip's own face out of the same list.
    pub fn window(&self) -> &[Core] {
        let end = (self.window + self.visible()).min(self.count);
        &self.cores[self.window..end]
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

    /// What a press does to the chip or the lid, and what it asks of `App`. While the cart still
    /// waits on its faces the arrows do nothing: the chip is not on screen, and a hop run unseen
    /// would open the cart with the chip already in the other socket, for an `A` to write a core
    /// the player never saw chosen. `A` and `B` still close it.
    ///
    /// The arrows step one core along the machine's list and do not wrap: at either end the
    /// board refuses rather than sending the chip round to the far side, because a chip that
    /// appeared at the other end would read as a second, longer hop rather than as "there is
    /// nothing over there". At the end of what the board is *showing* they step past it instead,
    /// and the window follows.
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
        // Where it is leaving from, on the board as it stands now: a window that is about to
        // roll changes what that socket is called, not where it is.
        let from = self.seat - self.window;
        if let Some(progress) = self.hop_progress(now) {
            if target == self.seat {
                return Outcome::Nothing;
            }
            // Back toward the socket it is leaving: the new hop starts as far through as the
            // old one had left to go, which puts the chip exactly where it already was.
            let done = ((1.0 - progress) * HOP_MS as f32) as Millis;
            self.hop = Some(Hop {
                from,
                started: now.saturating_sub(done),
            });
            self.seat = target;
            self.follow();
            // A hop is happening now, so a shake left over from an earlier refusal is not
            // this chip's any more.
            self.refusal = None;
            return Outcome::Nothing;
        }
        self.hop = Some(Hop { from, started: now });
        self.seat = target;
        self.follow();
        // Same rule as the turn above: a hop starting now has nothing to do with whatever was
        // refused before it, however recently.
        self.refusal = None;
        Outcome::Nothing
    }

    pub fn chip(&self, now: Millis) -> Chip {
        let shake = self.refusal.map_or(0.0, |r| r.offset(now));
        let to = self.seat - self.window;
        let (across, lift, tip, seated) = match (self.hop, self.hop_progress(now)) {
            (Some(hop), Some(q)) => {
                let from = self.pos(hop.from);
                let to = self.pos(to);
                let lean = if to > from {
                    1.0
                } else if to < from {
                    -1.0
                } else {
                    // A window rolled under it: the chip hops on the spot, which is the only
                    // way to say "this is a different core now" without moving to a socket
                    // that is not where it was.
                    0.0
                };
                let arc = (std::f32::consts::PI * q).sin();
                (from + (to - from) * ease(q), arc, lean * CHIP_TIP * arc, None)
            }
            _ => (self.pos(to), 0.0, 0.0, Some(self.seat())),
        };
        Chip {
            across,
            index: to,
            visible: self.visible(),
            lift,
            tip,
            seated,
            shake,
        }
    }

    /// Rolls the window by the least that puts the chip's socket back on the board. Called after
    /// every move and once at open, so the board can never be showing a list the chip is not on.
    fn follow(&mut self) {
        let visible = self.visible();
        if self.seat < self.window {
            self.window = self.seat;
        } else if self.seat >= self.window + visible {
            self.window = self.seat + 1 - visible;
        }
    }

    /// Where socket `i` falls between the two ends of the board: 0.0 at the leftmost, 1.0 at the
    /// rightmost, evenly spaced between. One socket is 0.0 whatever it is, there being no run to
    /// be a fraction of.
    fn pos(&self, i: usize) -> f32 {
        let visible = self.visible();
        if visible < 2 {
            return 0.0;
        }
        i as f32 / (visible - 1) as f32
    }

    fn begin_close(&mut self, now: Millis) {
        self.close = Some(Close {
            started: now,
            from: self.openness(now),
        });
    }

    /// How far through a hop in flight, or `None` once the chip is seated.
    fn hop_progress(&self, now: Millis) -> Option<f32> {
        let hop = self.hop?;
        let q = now.saturating_sub(hop.started) as f32 / HOP_MS as f32;
        (q < 1.0).then_some(q)
    }
}
