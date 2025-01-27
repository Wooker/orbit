use chip::{pac::Peripherals, Claimable, ClaimablePeripheral};

pub enum ClaimError {
    AlreadyClaimed,
    WrongType,
}

pub trait Claim<'p, P: Claimable> {
    fn claim(&'p mut self, peripheral: ClaimablePeripheral) -> Result<Claimed<P>, ClaimError>;
}

pub struct Claimed<'p, P: Claimable>(&'p mut P);
impl<'p, P: Claimable> Claimed<'p, P> {
    pub fn new(peripheral: &'p mut P) -> Self {
        Self { 0: peripheral }
    }

    pub fn revoke(self) -> Result<(), ClaimError> {
        Ok(())
    }

    pub fn modify<F>(&mut self, mut f: F)
    where
        F: FnMut(&&mut P),
    {
        let peripheral = &self.0;
        f(peripheral)
    }
}
