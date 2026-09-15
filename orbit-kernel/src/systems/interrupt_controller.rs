pub trait InterruptControllerSystem<'s> {
    fn enable_interrupt(&mut self);
    fn disable_interrupt(&mut self);
    fn dispatch(&mut self) -> &'s str;
}
