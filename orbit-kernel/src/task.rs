#![allow(unused)]

use core::{alloc::Layout, cell::UnsafeCell, pin::Pin};

use alloc::{boxed::Box, vec::Vec};

use crate::{context::Context, id::ID};

pub(crate) static TASK_ID: ID<usize> = ID::new(0);
const STACK_SIZE: usize = 32;

#[repr(C, align(4))]
pub(crate) struct TaskMeta {
    pin: Pin<Box<Task>>,
    priority: usize,
}

impl TaskMeta {
    pub fn new(pin: Pin<Box<Task>>, priority: usize) -> Self {
        Self { pin, priority }
    }

    pub fn priority(&self) -> usize {
        self.priority
    }

    pub fn pin(&self) -> &Pin<Box<Task>> {
        &self.pin
    }
}

#[repr(C, align(4))]
#[derive(Clone)]
pub(crate) struct Task {
    pub(crate) context: Context,
    pub(crate) task_id: usize,
    pub(crate) buf: Vec<u8>,
    pub(crate) stack: [u8; STACK_SIZE],
    // pmp: [PmpEntry; PMP_REGS],
    end: [u8; 8],
}

impl Task {
    pub fn new(payload: &[u8]) -> Option<Pin<Box<Self>>> {
        let layout = Layout::new::<Context>()
            .extend(Layout::new::<usize>())
            .unwrap()
            .0
            .extend(Layout::array::<u8>(payload.len()).unwrap())
            .unwrap()
            .0
            .extend(Layout::array::<u8>(STACK_SIZE).unwrap())
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

                (*ptr).buf = Vec::new();
                (*ptr).buf.resize(payload.len(), 0);

                (*ptr).buf.copy_from_slice(payload);
                (*ptr).end = [0, 0, 0, 0, 15, 15, 15, 15];

                Some(Pin::new(Box::from_raw(ptr)))
            }
        }
    }
}
