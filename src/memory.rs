use core::ptr::addr_of_mut;

const HEAP_SIZE: usize = 16 * 1024;

// SAFETY: accessed only through the `GLOBAL_ALLOCATOR` wrapper below, which
// is guarded by a single-core, no-interrupts contract. When SMP or interrupts
// are introduced this must be replaced with a proper mutex-protected allocator.
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[derive(Debug)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

// SAFETY: single-core, no interrupts — the Scheduler has the same contract.
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    pub fn init(&mut self) {
        // SAFETY: HEAP_MEMORY is only mutated through this allocator.
        // addr_of_mut! avoids creating a reference to the static, preventing UB.
        let start = addr_of_mut!(HEAP_MEMORY) as usize;
        self.heap_start = start;
        self.heap_end = start + HEAP_SIZE;
        self.next = start;
    }

    /// Bump-allocate `size` bytes with the given power-of-two `align`.
    /// Returns `None` on OOM.
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<&'static mut [u8]> {
        let aligned = align_up(self.next, align);
        let new_next = aligned.checked_add(size)?;
        if new_next > self.heap_end {
            return None;
        }

        self.next = new_next;
        let offset = aligned - self.heap_start;
        // SAFETY: offset is within bounds and no alias exists for this range.
        unsafe { Some(&mut HEAP_MEMORY[offset..offset + size]) }
    }
}

/// Align `addr` up to the nearest multiple of `align` (must be a power of two).
fn align_up(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "align must be a power of two");
    (addr + align - 1) & !(align - 1)
}

// SAFETY: same single-core, no-interrupts contract as BumpAllocator.
static mut GLOBAL_ALLOCATOR: BumpAllocator = BumpAllocator::new();

pub fn init() {
    // SAFETY: called once at kernel boot, before any allocation.
    // addr_of_mut! avoids creating a reference to the static.
    unsafe {
        (*addr_of_mut!(GLOBAL_ALLOCATOR)).init();
    }
}

/// Allocate `size` bytes with `align` alignment from the global bump allocator.
/// Returns `None` on OOM.
#[allow(dead_code)] // will be used by future subsystems
pub fn allocate(size: usize, align: usize) -> Option<&'static mut [u8]> {
    // SAFETY: single-core, no interrupts.
    unsafe { (*addr_of_mut!(GLOBAL_ALLOCATOR)).allocate(size, align) }
}

/// Bytes consumed so far from the heap.
pub fn used_bytes() -> usize {
    // SAFETY: single-core read — no race possible.
    // addr_of_mut! avoids creating a shared reference to the mutable static.
    unsafe {
        let alloc = &*addr_of_mut!(GLOBAL_ALLOCATOR);
        alloc.next - alloc.heap_start
    }
}

/// Total heap capacity in bytes.
pub fn total_bytes() -> usize {
    HEAP_SIZE
}
