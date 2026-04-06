use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;

const ARENA_SIZE: usize = chip::RAM_SIZE - 0x1500 - 16;

#[repr(C, align(4))]
struct BlockHeader {
    size: usize,
    next: *mut BlockHeader,
}

#[repr(C, align(4))]
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

        unsafe {
            (*block).size = ARENA_SIZE;
            (*block).next = null_mut();

            *self.free_list.get() = block;
        }
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            if (*self.free_list.get()).is_null() {
                self.init();
            }
        }

        if layout.size() == 0 {
            return null_mut();
        }

        let header_size = size_of::<BlockHeader>();
        let header_align = align_of::<BlockHeader>();

        // Payload alignment
        let req_align = layout.align().max(header_align);

        // Total size needed
        let mut total_size = layout.size() + header_size;

        // Ensure block size alignment
        total_size = (total_size + req_align - 1) & !(req_align - 1);

        let mut prev: *mut BlockHeader = null_mut();
        let mut current = unsafe { *self.free_list.get() };

        while !current.is_null() {
            if unsafe { (*current).size } >= total_size {
                let remaining = unsafe { (*current).size } - total_size;

                if remaining >= header_size {
                    // Split block
                    let new_block = (current as usize + total_size) as *mut BlockHeader;

                    unsafe {
                        (*new_block).size = remaining;
                        (*new_block).next = (*current).next;
                        if prev.is_null() {
                            *self.free_list.get() = new_block;
                        } else {
                            (*prev).next = new_block;
                        }

                        (*current).size = total_size;
                    }
                } else {
                    // Use entire block

                    unsafe {
                        if prev.is_null() {
                            *self.free_list.get() = (*current).next;
                        } else {
                            (*prev).next = (*current).next;
                        }
                    }
                }

                let payload = (current as usize + header_size) as *mut u8;

                return payload;
            }

            prev = current;
            current = unsafe { (*current).next };
        }

        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        use core::mem::{align_of, size_of};
        use core::ptr::{null_mut, write_bytes};

        if ptr.is_null() {
            return;
        }

        let header_size = size_of::<BlockHeader>();
        let header_align = align_of::<BlockHeader>();

        let ptr_addr = ptr as usize;

        // Recover header
        let mut header_addr = ptr_addr;

        // Walk backwards to header
        header_addr -= header_size;

        let header = header_addr as *mut BlockHeader;

        debug_assert_eq!(header_addr % header_align, 0);

        let block_size = unsafe { (*header).size };

        // Zero payload (optional but safe)
        let payload_size = block_size - header_size;
        unsafe { write_bytes((header_addr + header_size) as *mut u8, 0, payload_size) };

        let mut prev: *mut BlockHeader = null_mut();
        let mut current = unsafe { *self.free_list.get() };

        // Insert in sorted order
        while !current.is_null() && (current as usize) < header_addr {
            prev = current;
            current = unsafe { (*current).next };
        }

        unsafe {
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

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        use core::cmp;
        use core::ptr::{copy_nonoverlapping, null_mut};

        if ptr.is_null() {
            return unsafe {
                self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()))
            };
        }

        if new_size == 0 {
            unsafe { self.dealloc(ptr, layout) };
            return null_mut();
        }

        let header_size = core::mem::size_of::<BlockHeader>();
        let header = (ptr as usize - header_size) as *mut BlockHeader;

        let old_total_size = unsafe { (*header).size };
        let old_payload_size = old_total_size - header_size;

        let align = layout.align().max(core::mem::align_of::<BlockHeader>());

        let mut new_total_size = new_size + header_size;
        new_total_size = (new_total_size + align - 1) & !(align - 1);

        // Shrink in place
        if new_total_size <= old_total_size {
            let remaining = old_total_size - new_total_size;

            if remaining >= header_size {
                unsafe {
                    (*header).size = new_total_size;

                    let new_block = (header as usize + new_total_size) as *mut BlockHeader;

                    (*new_block).size = remaining;

                    // Insert split part into free list
                    self.dealloc(
                        (new_block as usize + header_size) as *mut u8,
                        Layout::from_size_align_unchecked(remaining - header_size, align),
                    );
                }
            }

            return ptr;
        }

        // try to grow in place (coalesce with next block)
        let next_block_addr = header as usize + old_total_size;

        let mut prev: *mut BlockHeader = null_mut();
        let mut current = unsafe { *self.free_list.get() };

        while !current.is_null() {
            if current as usize == next_block_addr {
                let combined_size = old_total_size + unsafe { (*current).size };

                if combined_size >= new_total_size {
                    // Remove next block from free list
                    unsafe {
                        if prev.is_null() {
                            *self.free_list.get() = (*current).next;
                        } else {
                            (*prev).next = (*current).next;
                        }

                        (*header).size = combined_size;
                    }

                    // Split if extra space
                    let remaining = combined_size - new_total_size;
                    if remaining >= header_size {
                        unsafe { (*header).size = new_total_size };

                        let split_block = (header as usize + new_total_size) as *mut BlockHeader;

                        unsafe {
                            (*split_block).size = remaining;

                            self.dealloc(
                                (split_block as usize + header_size) as *mut u8,
                                Layout::from_size_align_unchecked(remaining - header_size, align),
                            );
                        }
                    }

                    return ptr;
                }
            }

            prev = current;
            current = unsafe { (*current).next };
        }

        // fallback: allocate new block
        let new_ptr = unsafe { self.alloc(Layout::from_size_align_unchecked(new_size, align)) };

        if new_ptr.is_null() {
            return null_mut();
        }

        unsafe { copy_nonoverlapping(ptr, new_ptr, cmp::min(old_payload_size, new_size)) };

        unsafe { self.dealloc(ptr, layout) };

        new_ptr
    }
}

#[global_allocator]
pub(crate) static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();
