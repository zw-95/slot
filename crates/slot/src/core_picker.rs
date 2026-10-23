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

/// The most cores one machine can put on the board. The picker is `Copy` and `App` holds it
/// frame to frame, so its sockets are a fixed array rather than a `Vec`. Three is what fits
/// across the board art; a machine offering more than this is a machine `System::cores` has
/// to answer for, because the picker cannot lay a fourth socket out.
pub const CORES_MAX: usize = 3;

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
    /// 0.0 over the leftmost socket, 1.0 over the rightmost, for whichever machine this is.
    /// The sockets are evenly spaced, so a fraction of the run lands either on one of them or
    /// in the air between two.
    pub across: f32,
    /// Which socket that is, left to right, and how many the board has. Given rather than
    /// measured back off `across`: rounding a fraction that came from an even spacing is how
    /// a chip in flight would be reported as sitting in the socket it is only passing over.
    pub index: usize,
    pub count: usize,
    /// 0.0 seated, 1.0 at the top of the hop.
    pub lift: f32,
    /// Radians, clockwise, leaning into the direction of travel.
    pub tip: f32,
    /// The socket it sits in, and so the name it wears. `None` in flight.
    pub seated: Option<Core>,
    /// Panel pixels the refusal puts on its x.
    pub shake: f32,
}

#[derive(Copy, Clone)]
struct Hop {
    /// Socket it left, by position on the board rather than by which core it is: two hops can
    /// run through the same core in opposite directions, and the arc is drawn from where the
    /// chip was to where it is going, not from what it was called.
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
    /// Which socket the chip is in, left to right. An index and not a `Core`, because the
    /// whole of the picker's own behaviour — where the chip is, which way a press moves it,
    /// when it is at an end — is about position on the board, and the core is only what that
    /// position is standing for.
    seat: usize,
    /// The cores the machine offers, left to right, and how many of them there are. Filled
    /// from the left; the rest of the array is never read.
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
    /// order the sockets are laid out, and must hold at least two — a board with one socket is
    /// not a question, and `App` does not open the picker for it.
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

    /// The sockets on the board, left to right. `App` asks for it to draw them: which faces to
    /// put down and how far apart, and the chip's own face comes out of the same list.
    pub fn cores(&self) -> &[Core] {
        &self.cores[..self.count]
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
    /// The arrows step one socket and do not wrap: at either end the board refuses rather than
    /// sending the chip round to the far side, because a chip that appeared at the other end
    /// would read as a second, longer hop rather than as "there is nothing over there".
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
        if let Some(progress) = self.hop_progress(now) {
            if target == self.seat {
                return Outcome::Nothing;
            }
            // Back toward the socket it is leaving: the new hop starts as far through as the
            // old one had left to go, which puts the chip exactly where it already was.
            let done = ((1.0 - progress) * HOP_MS as f32) as Millis;
            self.hop = Some(Hop {
                from: self.seat,
                started: now.saturating_sub(done),
            });
            self.seat = target;
            // A hop is happening now, so a shake left over from an earlier refusal is not
            // this chip's any more.
            self.refusal = None;
            return Outcome::Nothing;
        }
        self.hop = Some(Hop {
            from: self.seat,
            started: now,
        });
        self.seat = target;
        // Same rule as the turn above: a hop starting now has nothing to do with whatever was
        // refused before it, however recently.
        self.refusal = None;
        Outcome::Nothing
    }

    pub fn chip(&self, now: Millis) -> Chip {
        let shake = self.refusal.map_or(0.0, |r| r.offset(now));
        let (across, lift, tip, seated) = match (self.hop, self.hop_progress(now)) {
            (Some(hop), Some(q)) => {
                let from = self.pos(hop.from);
                let to = self.pos(self.seat);
                let lean = if to > from { 1.0 } else { -1.0 };
                let arc = (std::f32::consts::PI * q).sin();
                (from + (to - from) * ease(q), arc, lean * CHIP_TIP * arc, None)
            }
            _ => (self.pos(self.seat), 0.0, 0.0, Some(self.seat())),
        };
        Chip {
            across,
            index: self.seat,
            count: self.count,
            lift,
            tip,
            seated,
            shake,
        }
    }

    /// Where socket `i` falls between the two ends: 0.0 at the leftmost, 1.0 at the rightmost,
    /// evenly spaced between. One socket is 0.0 whatever it is, there being no run to be a
    /// fraction of.
    fn pos(&self, i: usize) -> f32 {
        if self.count < 2 {
            return 0.0;
        }
        i as f32 / (self.count - 1) as f32
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
