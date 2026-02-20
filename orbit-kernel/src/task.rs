#![allow(unused)]

use core::{
    alloc::Layout,
    cell::UnsafeCell,
    pin::Pin,
    ptr::{null, null_mut},
};

use alloc::{boxed::Box, vec::Vec};
use spaceport::packet::Packet;

use crate::{context::Context, id::ID};

pub(crate) static TASK_ID: ID<usize> = ID::new(0);
const STACK_SIZE: usize = 32;

#[repr(C, align(4))]
pub(crate) struct Task<'t> {
    pub(crate) context: Context,
    pub(crate) task_id: usize,
    pub(crate) stack: [usize; STACK_SIZE],
    pub(crate) packet: Packet<'t>,
    // pmp: [PmpEntry; PMP_REGS],
    pub(crate) priority: usize,
}

impl<'t> Task<'t> {
    pub fn new<'p>(packet: Packet<'p>, priority: usize, addr: usize) -> Option<Pin<Box<Self>>>
    where
        't: 'p,
    {
        let layout = Layout::new::<Task>()
            .extend(Layout::array::<u8>(packet.payload.len()).unwrap())
            .unwrap()
            .0
            .pad_to_align();

        unsafe {
            let ptr = alloc::alloc::alloc(layout) as *mut Task;

            if ptr.is_null() {
                // alloc::alloc::handle_alloc_error(layout);
                None
            } else {
                (*ptr).context.mepc = addr;
                (*ptr).context.gp = ptr as usize;
                (*ptr).context.sp =
                    &(*ptr).stack as *const [usize; STACK_SIZE] as usize + STACK_SIZE;

                let mut buf = core::slice::from_raw_parts_mut(
                    (ptr as usize + size_of::<Task>() as usize) as *mut usize as *mut u8,
                    packet.payload.len(),
                );
                buf.copy_from_slice(packet.payload);

                (*ptr).packet = Packet {
                    version: packet.version,
                    flags: packet.flags,
                    packet_id: packet.packet_id,
                    src: packet.src,
                    dst: packet.dst,
                    ttl: packet.ttl,
                    msg_type: packet.msg_type,
                    payload: buf,
                };
                (*ptr).context.a0 = buf.as_ptr() as usize;

                (*ptr).priority = priority;

                let task_id = TASK_ID.get_id();
                (*ptr).task_id = task_id;
                TASK_ID.set(task_id + 1);

                Some(Pin::new(Box::from_raw(ptr)))
            }
        }
    }
}

impl<'t> Drop for Task<'t> {
    fn drop(&mut self) {
        let layout = Layout::new::<Task>()
            .extend(Layout::array::<u8>(self.packet.payload.len()).unwrap())
            .unwrap()
            .0
            .pad_to_align();
        unsafe { alloc::alloc::dealloc(self as *mut Task as *mut u8, layout) }
    }
}
