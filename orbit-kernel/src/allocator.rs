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

unsafe impl Sync for SimpleAllocator {}
unsafe impl GlobalAlloc for SimpleAllocator {
    #[inline(never)]
    #[unsafe(no_mangle)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let remaining = &mut *self.remaining.get();

        let arena_start = self.arena.get() as usize;
        let arena_end = arena_start + ARENA_SIZE;

        let mut new_remaining = *remaining;

        if size > new_remaining {
            return null_mut();
        }

        // subtract first
        new_remaining -= size;

        // align down
        new_remaining &= !(align - 1);

        let ptr = arena_start + new_remaining;

        *remaining = new_remaining;

        ptr as *mut u8
    }

    #[inline(never)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let remaining = &mut unsafe { *self.remaining.get() };

        let arena_start = self.arena.get().cast::<u8>() as usize;
        let arena_end = arena_start + ARENA_SIZE;
        let ptr = ptr as usize;

        let arena = &mut unsafe { *self.arena.get() };

        // Only free if it's the last allocation (LIFO)
        if ptr >= arena_start && ptr < arena_end {
            if ptr == arena_start + *remaining {
                while *remaining != *remaining + size && *remaining < ARENA_SIZE {
                    arena[*remaining] = 0xff;
                    *remaining += 1;
                }
            }
        }
    }
}

#[global_allocator]
pub(crate) static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    arena: UnsafeCell::new([0x00; ARENA_SIZE]),
    remaining: UnsafeCell::new(ARENA_SIZE),
};
