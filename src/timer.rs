use core::sync::atomic::{AtomicU64, Ordering};

static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    // SeqCst on init guarantees visibility of the reset before any tick() call.
    TICK_COUNT.store(0, Ordering::SeqCst);
}

pub fn tick() {
    // Relaxed is sufficient here: we only need the counter to increase
    // monotonically and we read it back in the same execution context
    // (single-core, no interrupts). When interrupts are introduced this
    // should be revisited (AcqRel at minimum).
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn ticks() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}
