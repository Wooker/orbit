#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Clocks {
    pub hclk: usize,
}

impl Clocks {
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn freeze(&mut self) {
        self.hclk = 48_000_000;
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub const fn default() -> Self {
        Self { hclk: 0 }
    }
}
