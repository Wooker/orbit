#![allow(unused)]

use alloc::vec::Vec;
use chip::PortPeripheral;

use orbit_arch::interface::timer::Timer;
use orbit_common::{feature_mod_use, feature_mod_use_mutual};
use spaceport::{
    constants::{self, EOF, PROTOCOL_VERSION},
    error::EncodeError,
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
    Allocation,
}

pub(crate) struct Port<'p> {
    pub awaiting: bool,
    pub msg: usize,
    role: Role,
    peripheral: Uart<'p>,
    buf: [u8; MAX_BUFFER_LENGTH],
    packet_buf: [u8; MAX_PACKET_LENGTH],
    pub fragments: Vec<u8>,
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
            buf: [0; MAX_BUFFER_LENGTH],
            packet_buf: [0; MAX_PACKET_LENGTH],
            fragments: Vec::new(),
        }
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

    fn try_append_fragment(fragments: &mut Vec<u8>, payload: &[u8]) -> Result<(), PortError> {
        if fragments
            .try_reserve_exact(fragments.len() + payload.len())
            .is_err()
        {
            *fragments = Vec::new();
            return Err(PortError::Allocation);
        }

        fragments.append(&mut payload.to_vec());
        Ok(())
    }

    fn send_allocation_error(peripheral: &mut Uart, fragments: &mut Vec<u8>, p: &Packet) {
        let mut buf = [0; MAX_BUFFER_LENGTH];

        if let Ok(size) = p
            .reply_with_flags(Flags::ERROR, b"Could not allocate more memory")
            .encode(&mut buf)
        {
            let _ = peripheral.write(&mut buf[..size]);
            PACKET_ID.set(PACKET_ID.get_id() + 1);
        }
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub(crate) fn handle<'h>(&'h mut self) -> Option<Packet<'h>> {
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
                    if Self::try_append_fragment(&mut self.fragments, p.payload).is_err() {
                        Self::send_allocation_error(&mut self.peripheral, &mut self.fragments, &p);
                    }
                    None
                } else {
                    if self.fragments.is_empty() {
                        Some(p)
                    } else {
                        if Self::try_append_fragment(&mut self.fragments, p.payload).is_ok() {
                            p.payload = &self.fragments;
                            Some(p)
                        } else {
                            Self::send_allocation_error(
                                &mut self.peripheral,
                                &mut self.fragments,
                                &p,
                            );
                            None
                        }
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn respond<'r>(&'r mut self, packet: Packet<'r>) {
        let mut out = [0u8; MAX_BUFFER_LENGTH];
        packet
            .encode(&mut out)
            .and_then(|size| {
                self.send(&out[..size])
                    .map_err(|e| EncodeError::BufferTooSmall)
            })
            .unwrap_or(0);
    }
}

// use orbit_common::feature_mod_use;
//
// feature_mod_use!("ch592", pub);
// feature_mod_use!("ch32v208wbu6", pub);
// feature_mod_use!("ch32v003", pub);
// feature_mod_use!("bl702", pub);
