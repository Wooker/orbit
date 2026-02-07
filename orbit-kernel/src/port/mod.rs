#![allow(unused)]

use chip::PortPeripheral;

use orbit_arch::interface::timer::Timer;
use orbit_common::{feature_mod_use, feature_mod_use_mutual};
use spaceport::{
    constants::{self, EOF},
    message::Message,
    packet::{HEADER_LEN, Packet},
    transport::Transport,
};

use crate::ringbuf::RingBuf;
use crate::{RINGBUF_SIZE, RingbufType};

pub mod port_kind;
pub(crate) use port_kind::PortKinds;

pub enum UartError {
    Config,
    BufTooSmall,
}

feature_mod_use_mutual!(uart_v208, "ch32v208wbu6", "ch32v003");
feature_mod_use_mutual!(uart_x035, "ch32x035");

#[derive(Clone, Copy)]
pub(crate) enum Role {
    Leader,
    Follower,
    Candidate,
}

pub trait ConfigureGPIO {
    fn configure(&self);
}

#[derive(Debug)]
pub enum PortError {
    Send,
}

pub(crate) struct Port<'p> {
    pub awaiting: bool,
    pub msg: usize,
    role: Role,
    peripheral: Uart<'p>,
    payload_buf: [u8; RINGBUF_SIZE - HEADER_LEN],
    buf: [u8; RINGBUF_SIZE],
    pub rbuf: RingBuf<RINGBUF_SIZE>,
}

impl<'p> Port<'p> {
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn new(peripheral: &'p PortPeripheral, kind: PortKinds) -> Self {
        Self {
            awaiting: false,
            msg: 0,
            peripheral: Uart::new(peripheral, kind, Config::default()),
            role: Role::Candidate,
            payload_buf: [0; RINGBUF_SIZE - HEADER_LEN],
            buf: [0; RINGBUF_SIZE],
            rbuf: RingBuf::default(),
        }
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn push(&mut self) {
        self.rbuf.push(self.peripheral.read_byte());
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn write_byte(&mut self, b: u8) {
        self.peripheral.blocking_write_byte(b);
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn write<'a>(&'a mut self, buf: &[u8]) {
        buf.iter().for_each(|ch| self.write_byte(*ch));
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn send<'a>(&'a mut self, buf: &[u8]) -> Result<usize, PortError> {
        self.peripheral.write(buf).map_err(|_| PortError::Send);
        Ok(buf.len())
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn handle(&mut self) -> Option<Packet<'_>> {
        if let Ok(size) = self.peripheral.read(&mut self.buf)
            && size > 0
        {
            if let Ok(p) = Packet::decode(&self.buf[..size], &mut self.payload_buf) {
                Some(p)
            } else {
                None
            }
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
