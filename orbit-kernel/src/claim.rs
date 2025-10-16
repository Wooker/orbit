// While compiling with  rustc 1.91.0-nightly (54c581243 2025-08-25)
// cargo produces:
// ```
// warning: `#[unsafe(link_section)]` attribute cannot be used on inherent methods
// ```
// If such behavior is no longer observable on newer versions of rustc,
// remove this attribute
#![allow(unused_attributes)]

use orbit_common::count_idents;

pub enum ClaimError {
    AlreadyClaimed,
    WrongType,
}

pub trait Claimable {}

pub trait Claim<'p, P: Claimable> {
    fn claim(&'p mut self) -> Claimed<'p, P>;
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

    pub fn read<F, O>(&self, f: F) -> O
    where
        F: Fn(&P) -> O,
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
    () => {
        #[cfg(not(feature = "rt"))]
        #[derive(Copy, Clone, PartialEq, Eq)]
        pub enum KernelPeripherals { }

        impl KernelPeripherals {
            pub const fn discriminant(&self) -> usize {
                unsafe { *(self as *const Self as *const usize) }
            }
        }
    };
    ($chip:literal, $($field:ident=$val:expr),* $(,)?) => {
        #[cfg(feature = $chip)]
        #[repr(u8)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        pub enum KernelPeripherals {
            $( $field, )*
            MAX
        }
        #[cfg(all(feature = $chip, feature = "rt"))]
        pub const PERIPHERALS_NUM: usize = count_idents!($($field),*);
        #[cfg(all(feature = $chip, feature = "rt"))]
        pub const PERIPHERALS_TABLE: [usize; PERIPHERALS_NUM] = [0;PERIPHERALS_NUM];
        $(
            #[cfg(all(feature = $chip, feature = "rt"))]
            use chip::pac::$field;
            #[cfg(all(feature = $chip, feature = "rt"))]
            impl Claimable for $field {}
            #[cfg(all(feature = $chip, feature = "rt"))]
            #[unsafe(link_section = ".kernel.text")]
            impl<'p> Claim<'p, $field> for chip::pac::$field {
                #[inline(never)]
                fn claim(&'p mut self) -> Claimed<'p,$field>{
                    Claimed::new(self)
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
    // GPIOD = 0x40011400,
    USART1 = 0x40013800
);
impl_claim!(
    "ch32v208wbu6",
    GPIOA = 0x40010800,
    GPIOB = 0x40010C00,
    // GPIOC = 0x40011000,
    UART4 = 0x40004C00
);
impl_claim!(
    "ch32x035",
    GPIOA = 0x40010800,
    GPIOB = 0x40010C00,
    GPIOC = 0x40011000,
    USART1 = 0x40013800,
    SPI1 = 0x40013000,
);
impl_claim!();
