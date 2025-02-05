#![no_std]
#![no_main]

pub use paste;

#[macro_export]
macro_rules! feature_mod {
    ($chip:literal, $vis:vis) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
        }
    };
    ($chip:literal, $vis:vis, $name:ident) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$name>];
        }
    };
}

#[macro_export]
macro_rules! feature_mod_use {
    ($chip:literal, $vis:vis) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
            #[cfg(feature = $chip)]
            $vis use [<$chip>]::*;
        }
    };
    ($chip:literal, $vis:vis, $name:ident) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
            #[cfg(feature = $chip)]
            $vis use [<$chip>] as $name;
        }
    };
}
