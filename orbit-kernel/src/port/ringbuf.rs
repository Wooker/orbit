#![allow(unused)]

pub trait TraitBound
where
    Self: Sized + Default + Copy + PartialEq,
{
}
impl TraitBound for u8 {}

pub struct RingBuf<const SIZE: usize, T: TraitBound> {
    pub start: usize,
    pub end: usize,
    pub buf: [T; SIZE],
    pub termination: T,
}

impl<const SIZE: usize, T: TraitBound> RingBuf<SIZE, T> {
    pub(super) fn new(termination: T) -> Self {
        let buf = [T::default(); SIZE];
        Self {
            start: 0,
            end: 0,
            buf,
            termination,
        }
    }

    pub(super) fn push(&mut self, value: T) {
        self.buf[self.end] = value;

        if self.end + 1 == SIZE {
            self.end = 0;
        } else if self.end + 1 == self.start {
            if self.start + 1 == SIZE {
                self.start = 0;
            } else {
                self.start += 1;
            }
        } else {
            self.end += 1;
        }
    }

    pub(super) fn read(&mut self) -> Option<&[T]> {
        let last = if self.end == 0 {
            SIZE - 1
        } else {
            self.end - 1
        };
        if self.buf[last] == self.termination {
            if self.start <= self.end {
                let out = Some(&self.buf[self.start..self.end]);
                self.start = self.end;
                out
            } else {
                let offset = SIZE - self.start;
                for i in 0..self.end {
                    self.buf.swap(i, i + offset);
                }
                for i in 0..offset {
                    self.buf.swap(self.start + i, i);
                }
                self.start = 0;
                self.end = self.end + offset;
                Some(&self.buf[self.start..self.end])
            }
        } else {
            None
        }
    }

    pub(super) fn at(&self, index: usize) -> T {
        self.buf[index]
    }
}
