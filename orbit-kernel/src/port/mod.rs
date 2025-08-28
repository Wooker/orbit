#![allow(unused)]

use chip::PortPeripheral;
use orbit_arch::interface::timer::Timer;
use orbit_common::{feature_mod_use, feature_mod_use_mutual};

pub(crate) mod action;
use action::Action;

pub mod message;
use message::Message;

pub mod port_kind;
pub(crate) use port_kind::PortKinds;

pub mod ringbuf;
use ringbuf::RingBuf;

feature_mod_use_mutual!(uart_v208, "ch32v208wbu6", "ch32v003");
feature_mod_use_mutual!(uart_x035, "ch32x035");

#[derive(Clone, Copy)]
pub(crate) enum Role {
    Leader,
    Follower,
    Candidate,
}

pub const RINGBUF_SIZE: usize = 32;
pub type RingbufType = u8;

pub trait ConfigureGPIO {
    fn configure(&self);
}

pub(crate) struct Port<'p> {
    pub awaiting: bool,
    pub msg: usize,
    role: Role,
    peripheral: Uart<'p>,
    pub rbuf: RingBuf<RINGBUF_SIZE, RingbufType>,
}

impl<'p> Port<'p> {
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn new(peripheral: &'p PortPeripheral, kind: PortKinds) -> Self {
        Self {
            awaiting: false,
            msg: 0,
            peripheral: Uart::new(peripheral, kind, Config::default()),
            role: Role::Candidate,
            rbuf: RingBuf::new(0x0),
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn push(&mut self) {
        self.rbuf.push(self.peripheral.read());
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn read_buf(&mut self, index: usize) -> RingbufType {
        self.rbuf.at(index)
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn write(&mut self, ch: RingbufType) {
        self.peripheral.blocking_write_char(ch);
        for i in 5..=9 {
            self.peripheral.clear_int(i);
        }
        orbit_arch::delay(300);
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn write_str<'a>(&'a mut self, buf: &[RingbufType]) {
        for ch in buf.iter() {
            self.write(*ch);
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn write_self<'a>(&'a mut self) {
        for ch in self.rbuf.buf.into_iter() {
            self.write(ch);
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(crate) fn handle(&mut self) -> Option<Action> {
        let b = self.peripheral.read();
        self.rbuf.push(b);
        if let Some(slice) = self.rbuf.read() {
            Some(slice.into())
        } else {
            None
        }
    }

    pub(crate) fn respond(&mut self) {}
}

// use orbit_common::feature_mod_use;
//
// feature_mod_use!("ch592", pub);
// feature_mod_use!("ch32v208wbu6", pub);
// feature_mod_use!("ch32v003", pub);
// feature_mod_use!("bl702", pub);
