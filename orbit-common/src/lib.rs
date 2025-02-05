#![no_std]
#![no_main]

pub use paste;

#[macro_export]
macro_rules! feature_mod {
    ($vis:vis, $chip:literal) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
        }
    };
    ($vis:vis, $chip:literal, $name:ident) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>] as $name;
        }
    };
}

#[macro_export]
macro_rules! feature_mod_use {
    ($vis:vis, $chip:literal) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
            #[cfg(feature = $chip)]
            $vis use [<$chip>]::*;
        }
    };
    ($vis:vis, $chip:literal, $name:ident) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            $vis mod [<$chip>];
            #[cfg(feature = $chip)]
            $vis use [<$chip>] as $name;
        }
    };
}
