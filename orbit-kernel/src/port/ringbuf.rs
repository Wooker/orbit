pub(super) struct RingBuf<const SIZE: usize, T: Sized + Default + Copy> {
    start: usize,
    end: usize,
    buf: [T; SIZE],
}

impl<const SIZE: usize, T: Sized + Default + Copy> RingBuf<SIZE, T> {
    pub(super) fn new() -> Self {
        let buf = [T::default(); SIZE];
        Self {
            start: 0,
            end: 0,
            buf,
        }
    }

    pub(super) fn push(&mut self, value: T) {
        self.buf[self.end] = value;
        /*
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
        */
    }
    pub(super) fn at(&self, index: usize) -> T {
        self.buf[index]
    }
}
