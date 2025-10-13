#![no_std]

use core::ops::Add;

#[inline(never)]
pub fn add(a: usize, b: usize) -> usize {
    a.add(b)
}

#[inline(never)]
pub fn add_t<T: Add<Output = T>>(a: T, b: T) -> T {
    a.add(b)
}
