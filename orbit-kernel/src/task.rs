#![allow(unused)]

use core::{
    alloc::Layout,
    cell::UnsafeCell,
    mem::size_of,
    pin::Pin,
    ptr::{null, null_mut},
};

use alloc::{boxed::Box, vec::Vec};
use spaceport::packet::Packet;

use crate::{context::Context, id::ID};

pub(crate) static TASK_ID: ID<usize> = ID::new(0);
const STACK_SIZE: usize = 512;

#[repr(C, align(4))]
pub(crate) struct TaskHeader<'t> {
    pub(crate) context: Context,
    pub(crate) task_id: usize,
    pub(crate) packet: Packet<'t>,
    pub(crate) priority: usize,
    pub(crate) state: TaskState,
    pub(crate) stack: [usize; STACK_SIZE],
}

#[derive(PartialEq)]
pub(crate) enum TaskState {
    Ready,
    Waiting,
    Blocked(usize),
}

#[repr(C, align(4))]
pub(crate) struct Task<'t> {
    pub(crate) header: TaskHeader<'t>,
    payload: [u8],
}

impl<'t> Task<'t> {
    pub fn new<'p>(packet: Packet<'p>, priority: usize, addr: usize) -> Option<Pin<Box<Self>>>
    where
        't: 'p,
    {
        let payload_len = packet.payload.len();

        // layout = header + payload
        let (layout, payload_offset) = Layout::new::<TaskHeader>()
            .extend(Layout::array::<u8>(payload_len).ok()?)
            .ok()?;

        let layout = layout.pad_to_align();

        unsafe {
            let raw = alloc::alloc::alloc(layout);
            if raw.is_null() {
                return None;
            }

            let header_ptr = raw as *mut TaskHeader;
            let task_id = TASK_ID.get_id();

            // initialize header
            core::ptr::write(
                header_ptr,
                TaskHeader {
                    context: Context::new(),
                    task_id: TASK_ID.get_id(),
                    stack: [0; STACK_SIZE],
                    packet: packet,
                    priority,
                    state: TaskState::Waiting,
                },
            );
            TASK_ID.set(task_id + 1);

            // payload pointer (correctly aligned!)
            let payload_ptr = raw.add(payload_offset);

            core::ptr::copy_nonoverlapping(packet.payload.as_ptr(), payload_ptr, payload_len);

            (*header_ptr).packet.payload =
                core::slice::from_raw_parts_mut(payload_ptr, payload_len);

            (*header_ptr).context.mepc = addr;
            (*header_ptr).context.gp = raw as usize;
            // SP must point to the end of the stack buffer in bytes.
            // Using `+ STACK_SIZE` here underflows stack capacity because STACK_SIZE is
            // number of `usize` elements, not number of bytes.
            (*header_ptr).context.sp = (&(*header_ptr).stack as *const [usize; STACK_SIZE]
                as usize)
                + (STACK_SIZE * size_of::<usize>());
            (*header_ptr).context.a0 = payload_ptr as usize;
            (*header_ptr).context.a1 = payload_len;

            // construct fat pointer properly
            let task_ptr = core::ptr::from_raw_parts_mut(raw as *mut (), payload_len) as *mut Task;

            Some(Pin::new_unchecked(Box::from_raw(task_ptr)))
        }
    }

    pub fn for_driver<'p>(self: &mut Pin<Box<Self>>, driver_addr: usize) {
        self.header.context.a2 = self.header.context.a1;
        self.header.context.a1 = self.header.context.a0;
        self.header.context.a0 = driver_addr;
    }
}
