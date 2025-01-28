use chip::claimable::Claimable;

pub enum ClaimError {
    AlreadyClaimed,
    WrongType,
}

pub trait Claim<'p, P: Claimable> {
    fn claim(&'p mut self) -> Result<Claimed<P>, ClaimError>;
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

#[macro_export]
macro_rules! impl_claim {
    ($($field:ident),* $(,)?) => {
        $(
            impl<'p> Claim<'p, $field> for Kernel {
                fn claim(&'p mut self) -> Result<Claimed<$field>, ClaimError> {
                    let peripherals = unsafe { self.peripherals.assume_init_mut() };
                    if 1 == 1 { // Replace with actual condition for checking claim status
                        Ok(Claimed::new(&mut peripherals.$field))
                    } else {
                        Err(ClaimError::AlreadyClaimed)
                    }
                }
            }
        )*
    };
}
