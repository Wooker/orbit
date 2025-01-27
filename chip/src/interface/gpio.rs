pub enum Mode {
    Input,
    Output,
    Alternate,
}

pub trait OrbitGPIO<const PORT: char, const NUM: u8> {
    fn enable(&mut self);
    fn disable(&mut self);
    fn configure(&mut self);
}
