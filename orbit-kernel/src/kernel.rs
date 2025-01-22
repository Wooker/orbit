#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static KERNEL_MAJOR: u8 = 0;
#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static KERNEL_MINOR: u8 = 1;

// #[used]
// #[no_mangle]
// #[link_section = ".kernel"]
// pub static KERNEL: Kernel = Kernel::new();

#[cfg(feature = "ch32v208wbu6")]
use chip::Peripherals;

#[cfg(feature = "ch592")]
use chip::Peripherals;

#[cfg(feature = "esp32c3")]
use chip::Peripherals;

pub struct Kernel {
    peripherals: Peripherals,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            peripherals: unsafe { Peripherals::steal() },
        }
    }

    pub fn initialize(&mut self) -> ! {
        self.peripherals
            .RCC
            .apb2prstr
            .write(|w| unsafe { w.bits(1 << 3) });
        self.peripherals
            .RCC
            .apb2prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 3)) });
        // Enable GPIO PORT B
        self.peripherals
            .RCC
            .apb2pcenr
            .write(|w| unsafe { w.bits(1 << 3) });
        // Set PB8 as output with 50Mhz speed
        self.peripherals
            .GPIOB
            .cfghr
            .write(|w| unsafe { w.bits(0b0101) });
        self.peripherals
            .GPIOB
            .bshr
            .write(|w| unsafe { w.bits(1 << 24) });
        loop {
            self.peripherals
                .GPIOB
                .bshr
                .write(|w| unsafe { w.bits(1 << 8) });
            orbit_arch::riscv32::riscv::asm::delay(1000000);
            self.peripherals
                .GPIOB
                .bshr
                .write(|w| unsafe { w.bits(1 << 24) });
            orbit_arch::riscv32::riscv::asm::delay(1000000);
        }
    }
    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}
