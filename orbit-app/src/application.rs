use orbit_kernel::application::Context;
// const STACK_SIZE: u32 = 1024;

pub trait Application {
    fn main(&mut self);
    #[allow(unsafe_code)]
    fn context(&self) -> Context;
    extern "C" fn ecall();
}

pub trait AsBytes {
    type Output;
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for () {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}
