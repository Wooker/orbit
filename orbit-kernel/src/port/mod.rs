use crate::port::uart_v208::{Config, Uart};
use chip::PortPeripheral;

mod ringbuf;
use ringbuf::RingBuf;

enum Role {
    Leader,
    Follower,
    Candidate,
}

enum Message {
    Hell0,
    Bye,
    Invoke(u8),
    Unknown,
}

pub(crate) struct Port<'p> {
    role: Role,
    peripheral: Uart<'p>,
    rbuf: RingBuf<32, u8>,
}

impl<'p> Port<'p> {
    pub(crate) fn new(p: &'p PortPeripheral) -> Self {
        Self {
            peripheral: Uart::new(p, Config::default()),
            role: Role::Candidate,
            rbuf: RingBuf::new(),
        }
    }

    pub(crate) fn read(&mut self) {
        self.rbuf.push(self.peripheral.read());
    }

    pub(crate) fn write(&mut self, ch: u8) {
        self.peripheral.blocking_write_char(ch);
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
