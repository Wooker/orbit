const STACK_SIZE: u32 = 1024;

pub trait Application {
    #[inline(never)]
    fn main(&mut self);
    #[allow(unsafe_code)]
    unsafe fn stack_top() -> usize;
}
