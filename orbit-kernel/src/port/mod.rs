#![allow(unused)]

use alloc::vec::Vec;
use chip::PortPeripheral;

use orbit_arch::interface::timer::Timer;
use orbit_common::{feature_mod_use, feature_mod_use_mutual};
use spaceport::{
    constants::{self, EOF, PROTOCOL_VERSION},
    message::Message,
    packet::{HEADER_LEN, MAX_BUFFER_LENGTH, MAX_PACKET_LENGTH, MAX_PAYLOAD_LENGTH, Packet},
    transport::Transport,
    types::Flags,
};

use crate::ringbuf::RingBuf;
use crate::{RINGBUF_SIZE, kernel::PACKET_ID};

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
    buf: [u8; MAX_BUFFER_LENGTH],
    packet_buf: [u8; MAX_PACKET_LENGTH],
    pub fragments: Vec<u8>,
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
            buf: [0; RINGBUF_SIZE],
            packet_buf: [0; MAX_PACKET_LENGTH],
            fragments: Vec::with_capacity(MAX_PAYLOAD_LENGTH),
            rbuf: RingBuf::new(),
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
        PACKET_ID.set(PACKET_ID.get_id() + 1);
        Ok(buf.len())
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn handle(&mut self) -> Option<Packet> {
        if let Ok(size) = self.peripheral.read(&mut self.buf)
            && size > 0
        {
            let packet = Packet::decode(&self.buf[..size], &mut self.packet_buf);

            if let Ok(mut p) = packet {
                // Reply ACK if the flag is present
                if p.flags.contains(Flags::ACK_REQUIRED) {
                    let mut buf = [0; MAX_BUFFER_LENGTH];
                    if let Ok(size) = p.ack(&[]).encode(&mut buf) {
                        self.peripheral
                            .write(&mut buf[..size])
                            .map_err(|_| PortError::Send);
                        PACKET_ID.set(PACKET_ID.get_id() + 1);
                    }
                }

                if p.flags.contains(Flags::FRAGMENTED) {
                    self.fragments.append(&mut p.payload.to_vec());
                    None
                } else {
                    if self.fragments.is_empty() {
                        Some(p)
                    } else {
                        self.fragments.append(&mut p.payload.to_vec());
                        p.payload = &self.fragments;
                        Some(p)
                    }
                }
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
