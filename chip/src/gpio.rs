pub enum Mode {
    Input,
    Output,
    Alternate,
}

pub trait OrbitGPIO<const Port: char, const Num: u8> {
    fn enable(self) -> impl OrbitGPIOBuilder<Port, Num>;
    fn disable(&self);
}

pub trait OrbitGPIOBuilder<const Port: char, const Num: u8> {
    fn configure(self) -> impl OrbitGPIO<Port, Num>;
}
