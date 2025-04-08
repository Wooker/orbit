use crate::{
    arch::interface::timer::Timer,
    kernel::KERNEL,
    port::uart_v208::{Config, Uart},
};

use action::Action;
use chip::PortPeripheral;

pub(crate) mod action;
pub(crate) mod message;

pub(crate) mod ringbuf;
use ringbuf::RingBuf;

pub(crate) enum Role {
    Leader,
    Follower,
    Candidate,
}

pub(crate) struct Port<'p> {
    role: Role,
    peripheral: Uart<'p>,
    pub rbuf: RingBuf<32, u8>,
}

impl<'p> Port<'p> {
    pub(crate) fn new(p: &'p PortPeripheral) -> Self {
        Self {
            peripheral: Uart::new(p, Config::default()),
            role: Role::Candidate,
            rbuf: RingBuf::new(0x0),
        }
    }

    pub(crate) fn push(&mut self) {
        self.rbuf.push(self.peripheral.read());
    }
    pub(crate) fn read_buf(&mut self, index: usize) -> u8 {
        self.rbuf.at(index)
    }

    pub(crate) fn write(&mut self, ch: u8) {
        self.peripheral.blocking_write_char(ch);
        for i in 5..=9 {
            self.peripheral.clear_int(i);
        }
        unsafe { KERNEL.assume_init_read().core.timer.delay(100) };
    }

    pub(crate) fn handle(&mut self) -> Action {
        self.push();
        if let Some(slice) = self.rbuf.read() {
            slice.into()
        } else {
            Action::Nothing
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
