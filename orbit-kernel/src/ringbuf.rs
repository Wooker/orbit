pub trait Terminate
where
    Self: Sized + Default + Copy + PartialEq,
{
    fn termination() -> Self;
}
impl Terminate for u8 {
    fn termination() -> Self {
        32
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RingBuf<const SIZE: usize, T: Terminate> {
    pub start: usize,
    pub end: usize,
    pub buf: [T; SIZE],
    pub termination: T,
}

impl<const SIZE: usize, T: Terminate> RingBuf<SIZE, T> {
    #[rustc_align(4)]
    #[inline(never)]
    pub fn new(termination: T) -> Self {
        let buf = [termination; SIZE];
        Self {
            start: 0,
            end: 0,
            buf,
            termination,
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    pub fn push(&mut self, value: T) {
        if self.end < SIZE {
            self.buf[self.end] = value;
            self.end += 1;
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    pub fn read(&mut self) -> Option<&[T]> {
        if self.end == SIZE {
            // if self.end != self.start && self.buf[self.end - 1] == self.termination {
            // let out = Some(&self.buf[self.start..self.end]);
            let out = Some(&self.buf[..]);
            self.start = 0;
            self.end = 0;
            out
        } else {
            None
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    pub fn fill(&mut self) {
        while self.end != SIZE {
            self.buf[self.end] = T::termination();
            self.end += 1;
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    pub fn flush(&mut self) {
        self.start = 0;
        self.end = 0;
        for i in self.buf.iter_mut() {
            *i = self.termination;
        }
    }

    #[allow(unused)]
    #[rustc_align(4)]
    #[inline(never)]
    pub(super) fn at(&self, index: usize) -> T {
        self.buf[index]
    }
}

impl<const SIZE: usize, T: Terminate> Default for RingBuf<SIZE, T> {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            buf: [T::termination(); SIZE],
            termination: T::termination(),
        }
    }
}
