use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null_mut,
};

const ARENA_SIZE: usize = 1024 * 10;
const MAX_LAYOUT_SIZE: usize = 1024;
pub(crate) struct SimpleAllocator {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    remaining: UnsafeCell<usize>,
}

unsafe impl Sync for SimpleAllocator {}
unsafe impl GlobalAlloc for SimpleAllocator {
    #[inline(never)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if align > MAX_LAYOUT_SIZE || size == 0 {
            return null_mut();
        }

        let remaining = &mut *self.remaining.get();

        let arena_start = self.arena.get().cast::<u8>() as usize;

        let current_top = arena_start + *remaining;

        let new_top = current_top.checked_sub(size).unwrap();
        let aligned_top = new_top & !(align - 1);

        if aligned_top < arena_start {
            return null_mut();
        }

        *remaining = aligned_top - arena_start;

        aligned_top as *mut u8
    }

    #[inline(never)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let remaining = &mut *self.remaining.get();

        let arena_start = self.arena.get().cast::<u8>() as usize;
        let arena_end = arena_start + ARENA_SIZE;
        let ptr = ptr as usize;

        // Only free if it's the last allocation (LIFO)
        if ptr >= arena_start && ptr < arena_end {
            if ptr == arena_start + *remaining {
                *remaining += size;
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    arena: UnsafeCell::new([0xff; ARENA_SIZE]),
    remaining: UnsafeCell::new(ARENA_SIZE),
};
