#![allow(unused)]

use crate::{
    message::Message,
    ringbuf::{RingBuf, TraitBound},
    RingbufType, RINGBUF_SIZE,
};

pub(crate) struct Action {
    pub(crate) message: Message,
    pub rbuf: RingBuf<RINGBUF_SIZE, RingbufType>,
}

impl Action {
    pub fn new(message: Message, termination: RingbufType) -> Self {
        Self {
            message,
            rbuf: RingBuf::default(),
        }
    }
}

impl From<&[u8]> for Action {
    fn from(value: &[u8]) -> Self {
        let message: Message = value[0].into();
        let mut rbuf = RingBuf::default();
        match message {
            Message::Invoke => {
                for ch in value[1..].iter() {
                    rbuf.push(*ch);
                }
                rbuf.push(<RingbufType as TraitBound>::termination());
                Self {
                    message: Message::Invoke,
                    rbuf,
                }
            }
            message => {
                for ch in value[1..].iter() {
                    rbuf.push(*ch);
                }
                Self { message, rbuf }
            }
        }
    }
}
