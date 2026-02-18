#![allow(unused)]

use core::{alloc::Layout, cell::UnsafeCell, pin::Pin};

use alloc::{boxed::Box, vec::Vec};

use crate::{context::Context, id::ID};

pub(crate) static TASK_ID: ID<usize> = ID::new(0);
const STACK_SIZE: usize = 32;

#[repr(C, align(4))]
pub(crate) struct TaskMeta<'tm> {
    pin: Pin<Box<Task<'tm>>>,
    priority: usize,
    packet_id: u16,
}

impl<'tm> TaskMeta<'tm> {
    pub fn new(pin: Pin<Box<Task<'tm>>>, priority: usize, packet_id: u16) -> Self {
        Self {
            pin,
            priority,
            packet_id,
        }
    }

    pub fn priority(&self) -> usize {
        self.priority
    }

    pub fn pin(self) -> Pin<Box<Task<'tm>>> {
        self.pin
    }

    pub fn pin_ref(&self) -> &Pin<Box<Task<'tm>>> {
        &self.pin
    }

    pub fn packet_id(self) -> u16 {
        self.packet_id
    }
}

#[repr(C, align(4))]
pub(crate) struct Task<'t> {
    pub(crate) context: Context,
    pub(crate) task_id: usize,
    pub(crate) buf: &'t mut [u8],
    pub(crate) stack: [usize; STACK_SIZE],
    // pmp: [PmpEntry; PMP_REGS],
    end: [u8; 8],
}

impl<'t> Task<'t> {
    pub fn new(payload: &[u8]) -> Option<Pin<Box<Self>>> {
        let layout = Layout::new::<Context>()
            .extend(Layout::new::<usize>())
            .unwrap()
            .0
            .extend(Layout::array::<u8>(payload.len()).unwrap())
            .unwrap()
            .0
            .extend(Layout::new::<&[u8]>())
            .unwrap()
            .0
            .extend(Layout::array::<usize>(STACK_SIZE).unwrap())
            .unwrap()
            .0
            .extend(Layout::array::<u8>(8).unwrap())
            .unwrap()
            .0
            .pad_to_align();

        unsafe {
            let ptr = alloc::alloc::alloc(layout) as *mut Task;

            if ptr.is_null() {
                // alloc::alloc::handle_alloc_error(layout);
                None
            } else {
                (*ptr).task_id = TASK_ID.get_id();
                TASK_ID.set((*ptr).task_id + 1);

                (*ptr).context.ra = 0x12345678;
                (*ptr).context.mepc = 0x87654321;

                let mut buf = core::slice::from_raw_parts_mut(
                    (ptr as usize + size_of::<Task>() as usize) as *mut usize as *mut u8,
                    payload.len(),
                );

                buf.copy_from_slice(payload);
                (*ptr).buf = buf;
                (*ptr).end = [0, 0, 0, 0, 15, 15, 15, 15];

                Some(Pin::new(Box::from_raw(ptr)))
            }
        }
    }
}

impl<'t> Drop for Task<'t> {
    fn drop(&mut self) {
        let layout = Layout::new::<Context>()
            .extend(Layout::new::<usize>())
            .unwrap()
            .0
            .extend(Layout::array::<u8>(self.buf.len()).unwrap())
            .unwrap()
            .0
            .extend(Layout::new::<&[u8]>())
            .unwrap()
            .0
            .extend(Layout::array::<usize>(STACK_SIZE).unwrap())
            .unwrap()
            .0
            .extend(Layout::array::<u8>(8).unwrap())
            .unwrap()
            .0
            .pad_to_align();
        unsafe { alloc::alloc::dealloc(self as *mut Task as *mut u8, layout) }
    }
}
