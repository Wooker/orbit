#![no_std]

#[inline(never)]
pub fn add() -> usize {
    1
}

#[inline(never)]
pub fn nested() -> usize {
    my_lib2::add()
}
