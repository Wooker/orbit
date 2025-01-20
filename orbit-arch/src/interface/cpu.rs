#[const_trait]
pub trait Cpu {
    fn new() -> Self;
    fn init(&self);
}
