use std::ops::Deref;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Video handoff from the emulator thread to the renderer. Buffers are moved, never copied,
/// and the lock is only ever held for a pointer swap, so neither side waits on the other.
/// Three buffers are enough: one being written, one published, one being read.
pub struct Frames {
    inner: Mutex<Inner>,
    /// How big a frame the pool holds room for, in bytes. Grown when a machine with a larger
    /// frame than the one the pool was built for is loaded — see `grow` — and never shrunk, so
    /// the usual case reads it without touching the lock at all.
    size: AtomicUsize,
    /// Every successful `latest`. `latest` consumes, so anything that calls it outside the
    /// render path silently steals a frame the renderer would have drawn.
    taken: AtomicU64,
}

struct Inner {
    ready: Option<Vec<u8>>,
    /// The size the picture in `ready` is, in pixels. It travels with the buffer rather than
    /// being read back off the machine: a core may change its frame size between frames, and
    /// the pool outlives any one machine anyway.
    ready_dims: (u32, u32),
    spare: Vec<Vec<u8>>,
    allocated: usize,
}

impl Frames {
    pub fn new(size: usize) -> Arc<Self> {
        Arc::new(Frames {
            inner: Mutex::new(Inner {
                ready: None,
                ready_dims: (0, 0),
                spare: Vec::new(),
                allocated: 0,
            }),
            size: AtomicUsize::new(size),
            taken: AtomicU64::new(0),
        })
    }

    /// Grow the pool to hold frames of `size` bytes. Called once, when the core reports a
    /// machine whose frame is larger than the one the pool was built for; never per frame, and
    /// never to shrink. Every buffer in flight is dropped rather than recycled, because a
    /// recycled one is the old size — which is exactly what made this call necessary.
    pub fn grow(&self, size: usize) {
        if size <= self.size.load(Ordering::Relaxed) {
            return;
        }
        let mut i = self.lock();
        // Re-read under the lock: the size is written from the emulator thread, and this is
        // the one place it changes.
        if size <= self.size.load(Ordering::Relaxed) {
            return;
        }
        self.size.store(size, Ordering::Relaxed);
        i.spare.clear();
        i.ready = None;
        i.ready_dims = (0, 0);
    }

    /// How big a frame the pool currently holds room for, in bytes.
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn take_write(&self) -> Vec<u8> {
        let mut i = self.lock();
        match i.spare.pop() {
            Some(buf) => buf,
            None => {
                i.allocated += 1;
                Vec::with_capacity(self.size.load(Ordering::Relaxed))
            }
        }
    }

    /// A frame the renderer never picked up is overwritten rather than queued. Presenting a
    /// stale frame late is worse than never presenting it.
    pub fn publish(&self, buf: Vec<u8>, w: u32, h: u32) {
        let mut i = self.lock();
        if let Some(dropped) = i.ready.replace(buf) {
            i.spare.push(dropped);
        }
        i.ready_dims = (w, h);
    }

    pub fn latest(self: &Arc<Self>) -> Option<FrameRef> {
        let mut i = self.lock();
        let buf = i.ready.take()?;
        let (w, h) = i.ready_dims;
        i.ready_dims = (0, 0);
        self.taken.fetch_add(1, Ordering::Relaxed);
        Some(FrameRef {
            frames: self.clone(),
            buf,
            w,
            h,
        })
    }

    /// Non consuming. The only safe way to ask whether a frame is waiting.
    pub fn is_ready(&self) -> bool {
        self.lock().ready.is_some()
    }

    pub fn taken(&self) -> u64 {
        self.taken.load(Ordering::Relaxed)
    }

    pub fn allocated(&self) -> usize {
        self.lock().allocated
    }

    fn recycle(&self, buf: Vec<u8>) {
        self.lock().spare.push(buf);
    }

    /// A poisoned lock means one side panicked mid swap. The worst case is a lost frame,
    /// which is better than the renderer giving up for the rest of the session.
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub struct FrameRef {
    frames: Arc<Frames>,
    buf: Vec<u8>,
    w: u32,
    h: u32,
}

impl FrameRef {
    /// How big the picture is, in pixels. Not the buffer's length: that is room for the
    /// largest frame this machine can produce, and a frame is only ever as big as the one the
    /// core just handed back.
    pub fn dims(&self) -> (u32, u32) {
        (self.w, self.h)
    }
}

impl Deref for FrameRef {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buf
    }
}

impl Drop for FrameRef {
    fn drop(&mut self) {
        self.frames.recycle(std::mem::take(&mut self.buf));
    }
}
