//! Timer trait
pub trait Timer {
    /// Delay in ns
    fn delay(&self, ns: u32);
}
