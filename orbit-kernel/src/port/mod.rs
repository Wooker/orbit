use chip::PortPeripheral;

pub(crate) struct Port<'p> {
    peripheral: &'p PortPeripheral,
}

impl<'p> Port<'p> {
    pub(crate) const fn new(peripheral: &'p PortPeripheral) -> Self {
        Self { peripheral }
    }
}

use orbit_common::feature_mod_use;

#[cfg(any(feature = "ch32v208wbu6", feature = "ch32v003"))]
mod uart_v208;

// feature_mod_use!("ch592", pub);
// feature_mod_use!("ch32v208wbu6", pub);
// feature_mod_use!("ch32v003", pub);
// feature_mod_use!("bl702", pub);
