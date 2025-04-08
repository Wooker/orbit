use core::mem::MaybeUninit;

pub(crate) enum Message {
    Invoke(MaybeUninit<u8>),
    Ok,
    Unknown,
}

impl From<u8> for Message {
    fn from(value: u8) -> Self {
        match value {
            1 => Message::Invoke(MaybeUninit::uninit()),
            2 => Message::Ok,
            _ => Message::Unknown,
        }
    }
}

impl Into<u8> for Message {
    fn into(self) -> u8 {
        match self {
            Message::Invoke(_) => 1,
            Message::Ok => 2,
            Message::Unknown => u8::MAX,
        }
    }
}
