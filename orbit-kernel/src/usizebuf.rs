use core::fmt::{self, Write};

/// A simple stack-based writer
pub struct UsizeBuf<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> Write for UsizeBuf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > self.buf.len() {
            return Err(fmt::Error);
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

/// Convert a usize into a &str using a user-provided buffer.
/// Returns `None` if the buffer is too small.
#[allow(unused)]
fn usize_to_str<'a>(n: usize, buf: &'a mut [u8]) -> Option<&'a str> {
    let mut writer = UsizeBuf { buf, len: 0 };
    write!(writer, "{}", n).ok()?;
    core::str::from_utf8(&writer.buf[..writer.len]).ok()
}
