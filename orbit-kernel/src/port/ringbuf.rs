#![allow(unused)]

pub trait TraitBound
where
    Self: Sized + Default + Copy + PartialEq,
{
}
impl TraitBound for u8 {}

#[derive(Clone, Copy)]
pub struct RingBuf<const SIZE: usize, T: TraitBound> {
    pub start: usize,
    pub end: usize,
    pub buf: [T; SIZE],
    pub termination: T,
}

impl<const SIZE: usize, T: TraitBound> RingBuf<SIZE, T> {
    #[repr(align(4))]
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn new(termination: T) -> Self {
        let buf = [T::default(); SIZE];
        Self {
            start: 0,
            end: 0,
            buf,
            termination,
        }
    }

    #[repr(align(4))]
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn push(&mut self, value: T) {
        self.buf[self.end] = value;

        if self.end + 1 != SIZE {
            self.end += 1;
        }
    }

    #[repr(align(4))]
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn read(&mut self) -> Option<&[T]> {
        if self.end != self.start && self.buf[self.end - 1] == self.termination {
            let out = Some(&self.buf[self.start..self.end]);
            self.start = 0;
            self.end = 0;
            out
        } else {
            None
        }
    }

    #[repr(align(4))]
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn flush(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    #[repr(align(4))]
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub(super) fn at(&self, index: usize) -> T {
        self.buf[index]
    }
}
