#![allow(unused)]

use crate::{
    message::Message,
    ringbuf::RingBuf,
    {RingbufType, RINGBUF_SIZE},
};

pub(crate) struct Action {
    pub(crate) message: Message,
    pub rbuf: RingBuf<RINGBUF_SIZE, RingbufType>,
}

impl Action {
    pub fn new(message: Message, termination: RingbufType) -> Self {
        Self {
            message,
            rbuf: RingBuf::new(termination),
        }
    }
}

impl From<&[u8]> for Action {
    fn from(value: &[u8]) -> Self {
        let message: Message = value[0].into();
        let mut rbuf = RingBuf::new(0);
        match message {
            Message::Invoke => {
                for ch in value[1..].iter() {
                    rbuf.push(*ch);
                }
                rbuf.push(0);
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
