use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null_mut,
};

pub const ARENA_SIZE: usize = 1024 * 4;
const MAX_LAYOUT_SIZE: usize = 1024;
#[repr(C, align(4))]
pub(crate) struct SimpleAllocator {
    pub(crate) arena: UnsafeCell<[u8; ARENA_SIZE]>,
    pub(crate) remaining: UnsafeCell<usize>,
}

impl SimpleAllocator {
    pub const fn new() -> Self {
        Self {
            arena: UnsafeCell::new([0; ARENA_SIZE]),
            remaining: UnsafeCell::new(ARENA_SIZE),
        }
    }

    #[inline]
    fn arena_start(&self) -> usize {
        self.arena.get() as usize
    }
}
unsafe impl Sync for SimpleAllocator {}
unsafe impl GlobalAlloc for SimpleAllocator {
    #[inline(never)]
    #[unsafe(no_mangle)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if size == 0 {
            return null_mut();
        }

        let remaining = &mut *self.remaining.get();

        // Compute new aligned position (downward bump)
        let mut new_remaining = *remaining;

        if size > new_remaining {
            return null_mut();
        }

        new_remaining -= size;

        // Align downward
        new_remaining &= !(align - 1);

        if new_remaining > *remaining {
            return null_mut();
        }

        *remaining = new_remaining;

        (self.arena_start() + new_remaining) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let size = layout.size();
        let align = layout.align();

        let remaining = &mut *self.remaining.get();
        let arena_start = self.arena_start();
        let ptr_addr = ptr as usize;
        let arena = &mut *self.arena.get();

        // Only free if LIFO
        if ptr_addr == arena_start + *remaining {
            // Recompute the aligned size the same way alloc did
            let mut new_remaining = *remaining + size;

            // Align upward to undo the downward align
            let mask = align - 1;
            if (new_remaining & mask) != 0 {
                new_remaining = (new_remaining + mask) & !mask;
            }

            if new_remaining <= ARENA_SIZE {
                while *remaining < new_remaining {
                    arena[*remaining] = 0;
                    *remaining += 1;
                }
                *remaining = new_remaining;
            }
        }
    }
}

#[global_allocator]
pub(crate) static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    arena: UnsafeCell::new([0x00; ARENA_SIZE]),
    remaining: UnsafeCell::new(ARENA_SIZE),
};
