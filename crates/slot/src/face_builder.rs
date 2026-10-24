//! The open cart's faces, built off the render thread. On the H700 a board takes the better part
//! of half a second to rasterise, which on the frame loop is half a second of frozen shelf, so
//! the frontend asks for the highlighted cart's faces as the caret lands and uploads them when
//! they come back. Only the newest request matters: a caret that has moved on has no use for the
//! cart it passed.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use slot_store::Cart;
use slot_ui::{board_face, border_face, cart_face, padded, CartFace, FACE_SCALE, TURN_PAD};

pub struct BuiltFaces {
    /// **The cart, not the cart's stem**: a machine and a stem, which is the only form that is
    /// safe to say "these faces were built for this cart" in — see `App::faces_key`, which is the
    /// other half of this and has to agree with it exactly.
    ///
    /// Called `key` and not `stem` because a stem is not a cart: `Tank (C)` sits on the Game Boy
    /// shelf and on the Family Computer's, and a lid built for one machine's cart opened over the
    /// other's is the wrong plastic on the screen with the right name on it. Two builders that
    /// named this differently is a board that is never found and a picker that opens as a black
    /// rectangle with no core on it, which is what a mismatch here looks like.
    pub key: String,
    pub board: CartFace,
    pub lid: CartFace,
}

/// What the open cart is built from: **the card, the cart, and the border the card says it is
/// wearing**, which is the same three things the shelf's own filler is given and for the same
/// reason — a worker is a thread with a channel, and it has no reach to the card's choices. See
/// `Dressed`, which this mirrors.
type Opened = (PathBuf, Cart, Option<u32>);

pub struct FaceBuilder {
    requests: Sender<Opened>,
    built: Receiver<BuiltFaces>,
}

impl FaceBuilder {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<Opened>();
        let (outbox, built) = mpsc::channel();
        // Named and built the way every other worker in this crate is (`link_start`,
        // `rewind`, `emu`, `audio/host`): a `Builder` rather than the bare `thread::spawn`, so
        // a `ps`/`top` on the device names this thread instead of just another anonymous one.
        let spawned = thread::Builder::new()
            .name("slot-faces".into())
            .spawn(move || {
                while let Ok(mut job) = inbox.recv() {
                    // Straight to the newest: the requests before it were for carts the caret
                    // has already left.
                    while let Ok(newer) = inbox.try_recv() {
                        job = newer;
                    }
                    let (root, cart, dressed) = job;
                    // **The lid is this cart wearing what the card says it wears.** It used to be
                    // `cart_face` — the machine's own shell, which is what a cart is before
                    // anybody picks anything — and a cart that had been given a cover therefore
                    // lifted the plastic it was born in, at *that* plastic's proportions rather
                    // than the ones it was wearing on the shelf. The cover comes from the card
                    // the way the shelf's does, because the cart itself cannot be asked: only the
                    // card knows what was given.
                    let worn = match dressed {
                        Some(number) => {
                            border_face(&root, &cart, number, FACE_SCALE).unwrap_or_else(|| cart_face(&cart))
                        }
                        None => cart_face(&cart),
                    };
                    let faces = BuiltFaces {
                        // **The cover is part of what these faces are**, and so part of the key: a
                        // lid wearing another cover is another lid. It was left out until now,
                        // which is why giving a cart a cover could leave the picker lifting the
                        // bare shell indefinitely — the key had not changed, so nothing was
                        // rebuilt, and only walking the caret away and back made it look again.
                        key: format!(
                            "{}/{}#{}",
                            cart.system().dir_name(),
                            cart.stem,
                            dressed
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        ),
                        // The back half is this cart's own machine's shell: a Game Boy cart
                        // opens as a portrait cart, and not as an Advance one.
                        board: board_face(&cart, cart.system()),
                        lid: padded(&worn, TURN_PAD),
                    };
                    if outbox.send(faces).is_err() {
                        return;
                    }
                }
            });
        // A thread that never started leaves both ends of `inbox`/`outbox` dropped with it, so
        // `request` below sends into a channel nobody drains and `take` only ever sees it
        // disconnected — the same shape as a worker too far behind to answer in time. `App`
        // already waits on that and gives up after `FACES_WAIT_MS`, so this is reported rather
        // than turned into a panic that would take the whole frontend down with it.
        if let Err(e) = spawned {
            eprintln!("slot: faces: worker thread failed to start: {e}");
        }
        FaceBuilder { requests, built }
    }

    pub fn request(&self, root: PathBuf, cart: Cart, dressed: Option<u32>) {
        // A worker that has gone has nothing to build with; the picker's own wait gives up.
        let _ = self.requests.send((root, cart, dressed));
    }

    /// The newest build finished since the last call, if any.
    pub fn take(&self) -> Option<BuiltFaces> {
        let mut newest = None;
        while let Ok(faces) = self.built.try_recv() {
            newest = Some(faces);
        }
        newest
    }
}

/// The border picker's faces: **one cart wearing every border the card holds**, built off the
/// render thread when the picker opens.
///
/// A cart face costs about 70 ms, and the ring holds one per border — thirty-three on a card
/// with thirty-two of them. Built on the frame loop that is two seconds of frozen shelf, so the
/// whole ring is built in one go off it, and held until the panel closes: the arrows then move
/// through the whole ring without waiting for anything, which is the only way a press can answer
/// at once.
/// One face of the border picker's ring, **sent the moment it is drawn** rather than with the rest
/// of the ring behind it.
pub struct BuiltBorder {
    /// Which cart this face was built for, as `App::faces_key` spells one — machine *and* stem,
    /// because a stem is not a cart. Checked when it lands rather than when it was asked for: a
    /// caret that moved while the worker was drawing is a face for a cart that is no longer under
    /// it, and two machines with a game of the same name is two rings.
    pub key: String,
    /// Which place in the ring this is: 0 the machine's own shell, then one per border in the
    /// order the card numbers them.
    pub index: usize,
    pub face: CartFace,
}

/// The places of a ring of `count`, **from `seat` outward** — the seat, one to its right, one to
/// its left, and so on until the ring is exhausted.
///
/// What lets the panel open on what it is looking at instead of on whatever happens to be first:
/// the ring is built in the order it will be *read*, so the three faces under the eye are the
/// first three out of the worker and the panel is up in the time three faces take rather than the
/// time the whole card's worth of them does.
fn ring_order(seat: usize, count: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let seat = seat.min(count - 1);
    let mut out = vec![seat];
    for step in 1..count {
        out.push((seat + step) % count);
        if out.len() >= count {
            break;
        }
        out.push((seat + count - step) % count);
    }
    out.truncate(count);
    out
}

pub struct BorderFaceBuilder {
    requests: Sender<(std::path::PathBuf, Cart, Vec<u32>, u32, usize)>,
    built: Receiver<BuiltBorder>,
}

impl BorderFaceBuilder {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<(std::path::PathBuf, Cart, Vec<u32>, u32, usize)>();
        let (outbox, built) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("slot-border-faces".into())
            .spawn(move || {
                while let Ok((root, cart, numbers, k, seat)) = inbox.recv() {
                    let key = format!("{}/{}", cart.system().dir_name(), cart.stem);
                    let count = numbers.len();
                    for place in ring_order(seat, count) {
                        // Every place is one of the card's own borders: the machine's shell is
                        // not an entry in this ring, it is what a cart wears until one is picked.
                        let face = slot_ui::border_face(&root, &cart, numbers[place], k);
                        // A border the card no longer carries draws nothing rather than a hole in
                        // the row: a card edited on a PC can hold fewer than it did.
                        if let Some(face) = face {
                            let done = BuiltBorder {
                                key: key.clone(),
                                index: place,
                                face,
                            };
                            if outbox.send(done).is_err() {
                                return;
                            }
                        }
                    }
                }
            });
        if let Err(e) = spawned {
            eprintln!("slot: border faces: worker thread failed to start: {e}");
        }
        BorderFaceBuilder { requests, built }
    }

    /// Build the ring for one cart, starting at `seat` — the place the picker opened on.
    pub fn request(
        &self,
        root: std::path::PathBuf,
        cart: Cart,
        numbers: Vec<u32>,
        k: u32,
        seat: usize,
    ) {
        let _ = self.requests.send((root, cart, numbers, k, seat));
    }

    /// Everything that has finished since the last call, in the order it was drawn.
    pub fn take(&self) -> Vec<BuiltBorder> {
        let mut out = Vec::new();
        while let Ok(done) = self.built.try_recv() {
            out.push(done);
        }
        out
    }
}

/// The shelf's faces, rasterised off the frame loop one cart at a time.
///
/// A cart face costs about 70 ms here, measured, and the shelf draws three carts either side
/// of the caret. Rasterising the whole card before the first frame is therefore seven seconds
/// on a hundred games — all of it spent on carts nobody is looking at yet. The shelf is born
/// with the seven it draws and this fills in the rest while the user reads them, nearest the
/// caret first, so a card of any size opens in the time seven faces take.
///
/// One request in flight at a time: the answer is always wanted, and a queue would only let
/// the worker fall further behind the caret.
/// What one shelf face is drawn *with*: the card, and the border this cart was last given, if it
/// was given one.
///
/// Carried rather than looked up in the worker, because the worker has no reach to the card's
/// choices — it is a thread with a channel — and a face is the one thing that has to be drawn
/// from both. `None` is the machine's own shell, which is every cart's until somebody picks.
type Dressed = (usize, Cart, Option<(std::path::PathBuf, u32)>);

pub struct ShelfFaceFiller {
    requests: Sender<Dressed>,
    built: Receiver<(usize, CartFace)>,
    outstanding: Option<usize>,
}

impl ShelfFaceFiller {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<Dressed>();
        let (outbox, built) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("slot-shelf-faces".into())
            .spawn(move || {
                while let Ok((i, cart, dressed)) = inbox.recv() {
                    // The cart wearing the border it was given, and the machine's own shell when
                    // it was given none. A border the card no longer carries falls back to the
                    // shell rather than to nothing: a cart with a stale number on it still has a
                    // shape, and the picker is where the number gets put right.
                    let face = match dressed {
                        Some((root, number)) => {
                            border_face(&root, &cart, number, FACE_SCALE)
                                .unwrap_or_else(|| cart_face(&cart))
                        }
                        None => cart_face(&cart),
                    };
                    if outbox.send((i, face)).is_err() {
                        return;
                    }
                }
            });
        if let Err(e) = spawned {
            eprintln!("slot: shelf faces: worker thread failed to start: {e}");
        }
        ShelfFaceFiller {
            requests,
            built,
            outstanding: None,
        }
    }

    pub fn busy(&self) -> bool {
        self.outstanding.is_some()
    }

    /// The index being built, if any. Used to keep the request weighted toward the caret.
    pub fn outstanding(&self) -> Option<usize> {
        self.outstanding
    }

    /// Ask for one, wearing `border` if it has one. Refused while another is in flight.
    pub fn request(
        &mut self,
        i: usize,
        cart: Cart,
        border: Option<(std::path::PathBuf, u32)>,
    ) -> bool {
        if self.outstanding.is_some() {
            return false;
        }
        if self.requests.send((i, cart, border)).is_err() {
            return false;
        }
        self.outstanding = Some(i);
        true
    }

    /// Everything that finished since the last call, in completion order.
    pub fn drain(&mut self) -> Vec<(usize, CartFace)> {
        let mut out = Vec::new();
        while let Ok(done) = self.built.try_recv() {
            self.outstanding = None;
            out.push(done);
        }
        out
    }
}

/// How many carts either side of the caret the shelf can draw at once. `Shelf::SLOTS` in
/// `slot-ui`; duplicated rather than exported because it is a drawing decision there and a
/// boot-budget decision here, and the two are allowed to disagree.
pub const SHELF_RADIUS: usize = 3;

/// The indices the shelf can currently show, as a contiguous window centred on `index` and
/// clipped to the ends of the list. Fewer than `SHELF_RADIUS * 2 + 1` only when there are
/// fewer carts than that.
pub fn shelf_window(index: usize, count: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let span = (SHELF_RADIUS * 2 + 1).min(count);
    let start = index.saturating_sub(SHELF_RADIUS).min(count - span);
    (start..start + span).collect()
}

/// Stops `boot.log` from reporting a number nobody can act on: `28.1` ms is the caret's own
/// face, which is built ahead of the row.
pub fn ring_distance(a: usize, b: usize, count: usize) -> usize {
    let d = a.abs_diff(b);
    d.min(count.saturating_sub(d))
}
