use crate::kernel::Kernel;

pub enum ClaimError {
    AlreadyClaimed,
    WrongType,
}

pub trait Claimable {}

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

    pub fn read<F>(&self, f: F) -> u8
    where
        F: Fn(&P) -> u8,
    {
        let peripheral = &self.0;
        f(peripheral)
    }
}

/// Macro which implements _Claim_ trait for Kernel targeting given _$chip_.
/// Provide peripherals from PAC to be able to _claim()_ in user application.
/// ```rust
/// impl_claim!("chip", Peripheral1, Peripheral2)
/// ```
/// will give access to _Peripheral1_ and _Peripheral2_ of _chip_ while
/// hiding all other peripherals via trait bound of the _Claimable_ trait.
#[macro_export]
macro_rules! impl_claim {
    ($chip:literal, $($field:ident),* $(,)?) => {
        $(
            #[cfg(feature = $chip)]
            use chip::pac::$field;
            #[cfg(feature = $chip)]
            impl Claimable for $field {}
            #[cfg(feature = $chip)]
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

impl_claim!("ch592", UART1, I2C, GPIO);
impl_claim!("ch32v208wbu6", RCC, GPIOB);
