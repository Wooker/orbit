pub enum Mode {
    Input,
    Output,
    Alternate,
}

pub trait OrbitGPIO<const Port: char, const Num: u8> {
    fn enable(&mut self);
    fn disable(&mut self);
    fn configure(&mut self);
}
