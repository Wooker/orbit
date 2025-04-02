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

    pub fn read<F>(&self, f: F) -> u32
    where
        F: Fn(&P) -> u32,
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
macro_rules! impl_claim {
    ($chip:literal, $($field:ident=$val:expr),* $(,)?) => {
        #[cfg(feature = $chip)]
        #[derive(Copy, Clone)]
        pub enum KernelPeripherals {
            $( $field, )*
        }
        $(
            #[cfg(feature = $chip)]
            use chip::pac::$field;
            #[cfg(feature = $chip)]
            impl Claimable for $field {}
            #[cfg(feature = $chip)]
            impl<'k, 'p> Claim<'p, $field> for Kernel<'k> {
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

// impl_claim!("ch592", UART1, I2C, GPIO);
impl_claim!(
    "ch32v003",
    GPIOA = 0x40010800,
    GPIOC = 0x40011000,
    GPIOD = 0x40011400,
    USART1 = 0x40013800
);
impl_claim!(
    "ch32v208wbu6",
    GPIOA = 0x40010800,
    GPIOB = 0x40010c00,
    GPIOC = 0x40011000,
    UART4 = 0x40004c00
);
