#![allow(unused)]

use crate::{
    arch::interface::timer::Timer,
    kernel::KERNEL,
    port::uart_v208::{Config, Uart},
};

use action::Action;
use chip::PortPeripheral;

pub(crate) mod action;
pub(crate) mod message;

pub mod ringbuf;
use message::Message;
use ringbuf::RingBuf;

pub(crate) enum Role {
    Leader,
    Follower,
    Candidate,
}

pub const RINGBUF_SIZE: usize = 32;
pub type RingbufType = u8;

pub(crate) struct Port<'p> {
    role: Role,
    peripheral: Uart<'p>,
    pub rbuf: RingBuf<RINGBUF_SIZE, RingbufType>,
}

impl<'p> Port<'p> {
    #[inline(never)]
    pub(crate) fn new(p: &'p PortPeripheral) -> Self {
        Self {
            peripheral: Uart::new(p, Config::default()),
            role: Role::Candidate,
            rbuf: RingBuf::new(0x0),
        }
    }

    #[inline(never)]
    pub(crate) fn push(&mut self) {
        self.rbuf.push(self.peripheral.read());
    }

    #[inline(never)]
    pub(crate) fn read_buf(&mut self, index: usize) -> RingbufType {
        self.rbuf.at(index)
    }

    #[inline(never)]
    pub(crate) fn write(&mut self, ch: RingbufType) {
        self.peripheral.blocking_write_char(ch);
        for i in 5..=9 {
            self.peripheral.clear_int(i);
        }
        unsafe { KERNEL.assume_init_read().core.timer.delay(400) };
    }

    #[inline(never)]
    pub(crate) fn write_str<'a>(&'a mut self, buf: &[RingbufType]) {
        for ch in buf.iter() {
            self.write(*ch);
        }
    }

    #[inline(never)]
    pub(crate) fn handle(&mut self) -> Option<Action> {
        self.push();
        if let Some(slice) = self.rbuf.read() {
            Some(slice.into())
        } else {
            None
        }
    }

    pub(crate) fn respond(&mut self) {}
}

#[cfg(any(feature = "ch32v208wbu6", feature = "ch32v003"))]
mod uart_v208;

// use orbit_common::feature_mod_use;
//
// feature_mod_use!("ch592", pub);
// feature_mod_use!("ch32v208wbu6", pub);
// feature_mod_use!("ch32v003", pub);
// feature_mod_use!("bl702", pub);
