pub trait ExternalInterruptControllerSystem<'s> {
    fn enable_interrupt(&mut self, irq: usize);
    fn disable_interrupt(&mut self);
    fn dispatch(&mut self) -> &'s str;
}
