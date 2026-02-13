#[derive(Clone, Copy)]
#[repr(C)]
pub struct RingBuf<const SIZE: usize> {
    pub start: usize,
    pub end: usize,
    pub buf: [u8; SIZE],
}

impl<const SIZE: usize> RingBuf<SIZE> {
    #[rustc_align(4)]
    #[inline(never)]
    pub fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            buf: [0; SIZE],
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    pub fn push(&mut self, value: u8) {
        if self.end < SIZE {
            self.buf[self.end] = value;
            self.end += 1;
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    pub fn read(&mut self) -> &[u8] {
        let out = &self.buf[self.start..self.end];
        self.start = 0;
        self.end = 0;
        out
    }
}

impl<const SIZE: usize> Default for RingBuf<SIZE> {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            buf: [0; SIZE],
        }
    }
}
