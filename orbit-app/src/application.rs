use orbit_kernel::application::Context;
// const STACK_SIZE: u32 = 1024;

pub trait Application {
    fn main(&mut self);
    #[allow(unsafe_code)]
    fn context(&self) -> Context;
    extern "C" fn ecall();
}
