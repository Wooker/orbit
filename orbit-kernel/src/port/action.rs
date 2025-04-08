use core::mem::MaybeUninit;

use super::message::Message;

pub(crate) enum Action {
    Invoke(u8),
    Nothing,
}

impl From<&[u8]> for Action {
    fn from(value: &[u8]) -> Self {
        let mut message: Message = value[0].into();
        match message {
            Message::Invoke(uninit) => Action::Invoke(value[1]),
            _ => Action::Nothing,
        }
    }
}
