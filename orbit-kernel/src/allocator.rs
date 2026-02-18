use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull, null_mut};

const ARENA_SIZE: usize = 16 * 1024;

#[repr(C)]
struct BlockHeader {
    size: usize,
    next: *mut BlockHeader,
}

pub struct SimpleAllocator {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    free_list: UnsafeCell<*mut BlockHeader>,
}

unsafe impl Sync for SimpleAllocator {}

impl SimpleAllocator {
    pub const fn new() -> Self {
        Self {
            arena: UnsafeCell::new([0; ARENA_SIZE]),
            free_list: UnsafeCell::new(null_mut()),
        }
    }

    unsafe fn init(&self) {
        let arena_ptr = self.arena.get() as *mut u8;
        let block = arena_ptr as *mut BlockHeader;

        (*block).size = ARENA_SIZE;
        (*block).next = null_mut();

        *self.free_list.get() = block;
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    #[inline(never)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if (*self.free_list.get()).is_null() {
            self.init();
        }

        let size = layout.size();
        let align = layout.align().max(align_of::<BlockHeader>());

        let mut prev: *mut BlockHeader = null_mut();
        let mut current = *self.free_list.get();

        while !current.is_null() {
            let block_start = current as usize;
            let block_size = (*current).size;

            // Space needed:
            // header + alignment padding + header back-pointer + user data
            let header_size = size_of::<BlockHeader>();
            let backptr_size = size_of::<*mut BlockHeader>();

            let mut user_start = block_start + header_size + backptr_size;

            user_start = (user_start + align - 1) & !(align - 1);

            let total_needed = user_start + size - block_start;

            if block_size >= total_needed {
                let remaining = block_size - total_needed;

                if remaining > size_of::<BlockHeader>() {
                    let new_block = (block_start + total_needed) as *mut BlockHeader;

                    (*new_block).size = remaining;
                    (*new_block).next = (*current).next;

                    if prev.is_null() {
                        *self.free_list.get() = new_block;
                    } else {
                        (*prev).next = new_block;
                    }
                } else {
                    if prev.is_null() {
                        *self.free_list.get() = (*current).next;
                    } else {
                        (*prev).next = (*current).next;
                    }
                }

                (*current).size = total_needed;

                // Store back-pointer to header
                let backptr_location = (user_start - backptr_size) as *mut *mut BlockHeader;

                *backptr_location = current;

                return user_start as *mut u8;
            }

            prev = current;
            current = (*current).next;
        }

        null_mut()
    }

    #[inline(never)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let backptr_size = size_of::<*mut BlockHeader>();

        // Recover header
        let backptr_location = (ptr as usize - backptr_size) as *mut *mut BlockHeader;

        let header = *backptr_location;

        // Zero payload
        let payload_size = (*header).size - ((ptr as usize) - (header as usize));

        ptr::write_bytes(ptr, 0, payload_size);

        let mut prev: *mut BlockHeader = null_mut();
        let mut current = *self.free_list.get();

        let header_addr = header as usize;

        // Insert in sorted order
        while !current.is_null() && (current as usize) < header_addr {
            prev = current;
            current = (*current).next;
        }

        (*header).next = current;

        if prev.is_null() {
            *self.free_list.get() = header;
        } else {
            (*prev).next = header;
        }

        // ---- Coalesce with next ----
        if !current.is_null() {
            let header_end = header_addr + (*header).size;
            if header_end == current as usize {
                (*header).size += (*current).size;
                (*header).next = (*current).next;
            }
        }

        // ---- Coalesce with previous ----
        if !prev.is_null() {
            let prev_end = (prev as usize) + (*prev).size;
            if prev_end == header_addr {
                (*prev).size += (*header).size;
                (*prev).next = (*header).next;
            }
        }
    }
}

#[global_allocator]
pub(crate) static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();
