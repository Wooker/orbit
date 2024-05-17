use core::{hash::Hash, pin};

use esp_hal::{
    clock::Clocks,
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{Peripherals, AES, DMA, GPIO, IO_MUX, SYSTEM},
    system::SystemClockControl,
    IO,
};

use crate::resources::{resource::Resource, Resources};

pub trait Multiplex<T>
where
    T: Peripheral<P = T>,
{
    type Reference<R>;

    fn claim(&self) -> &Self::Reference<T>;
}
pub struct Multiplexer {
    table: [bool; 2],
    aes: Resource<'static, AES>,
    dma: Resource<'static, DMA>,
    gpio: Resource<'static, GPIO>,
    io_mux: Resource<'static, IO_MUX>,
    // clocks: Resource<'static, Clocks<'static>>,
    // peripherals: Peripherals,
}

impl Multiplexer {
    pub fn new(resources: Resources) -> Self {
        Multiplexer {
            table: [false; 2],
            // peripherals: Option::take(peripherals).unwrap(),
            dma: Resource::new(pin::pin!(resources.DMA)),
            aes: Resource::new(pin::pin!(resources.AES)),
            gpio: Resource::new(resources.GPIO),
            io_mux: Resource::new(resources.IO_MUX),
            // clocks: resources.clocks,
        }
    }

    pub fn io(&mut self) -> Resource<'static, IO> {
        // let gpio = <Resource<'_, esp_hal::peripherals::GPIO> as Clone>::clone(&self.gpio).inner();
        // let io =
        todo!()
    }
}

impl Multiplex<AES> for Multiplexer {
    type Reference<R> = Resource<'static, AES>;

    fn claim(&self) -> &Self::Reference<AES> {
        &self.aes
    }
}

impl Multiplex<DMA> for Multiplexer {
    type Reference<R> = Resource<'static, DMA>;

    fn claim(&self) -> &Self::Reference<DMA> {
        &self.dma
    }
}

impl Multiplex<GPIO> for Multiplexer {
    type Reference<R> = Resource<'static, GPIO>;

    fn claim(&self) -> &Self::Reference<GPIO> {
        &self.gpio
    }
}

impl Multiplex<IO_MUX> for Multiplexer {
    type Reference<R> = Resource<'static, IO_MUX>;

    fn claim(&self) -> &Self::Reference<IO_MUX> {
        &self.io_mux
    }
}
