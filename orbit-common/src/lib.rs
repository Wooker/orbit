#![no_std]
#![no_main]

pub use paste;

#[macro_export]
macro_rules! feature_mod {
    ($vis:vis, $chip:literal, $alias:ident) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
            #[cfg(feature = $chip)]
            $vis use [<$chip>]::*;
        }
    };
}
