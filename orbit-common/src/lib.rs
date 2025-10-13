#![no_std]
#![no_main]

pub use paste;

#[macro_export]
macro_rules! feature_mod {
    ($chip:literal) => {
        $crate::paste::paste! {
            #[cfg(all(feature = $chip, feature = "rt"))]
            pub mod [<$chip>];
        }
    };
    ($chip:literal, $vis:vis) => {
        $crate::paste::paste! {
            #[cfg(all(feature = $chip, feature = "rt"))]
            $vis mod [<$chip>];
        }
    };
    ($chip:literal, $vis:vis, $name:ident) => {
        $crate::paste::paste! {
            #[cfg(all(feature = $chip, feature = "rt"))]
            $vis mod [<$name>];
        }
    };
}

#[macro_export]
macro_rules! feature_mod_use {
    ($chip:literal) => {
        $crate::paste::paste! {
            #[cfg(all(feature = $chip, feature = "rt"))]
            pub mod [<$chip>];
            #[cfg(all(feature = $chip, feature = "rt"))]
            pub use [<$chip>]::*;
        }
    };
    ($chip:literal, $vis:vis) => {
        $crate::paste::paste! {
            #[cfg(all(feature = $chip, feature = "rt"))]
            $vis mod [<$chip>];
            #[cfg(all(feature = $chip, feature = "rt"))]
            $vis use [<$chip>]::*;
        }
    };
    ($chip:literal, $vis:vis, $name:ident) => {
        $crate::paste::paste! {
            #[cfg(all(feature = $chip, feature = "rt"))]
            $vis mod [<$chip>];
            #[cfg(all(feature = $chip, feature = "rt"))]
            $vis use [<$chip>] as $name;
        }
    };
}

#[macro_export]
macro_rules! feature_mod_use_mutual {
    ($name:ident, $($chip:literal),+ $(,)?) => {
        $crate::paste::paste! {
            #[cfg(all(any($(feature = $chip),+), feature = "rt"))]
            pub mod [<$name>];
            #[cfg(all(any($(feature = $chip),+), feature = "rt"))]
            pub use [<$name>]::*;
        }
    };
}

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        const _: usize = 0
            - (!{
                const ASSERT: bool = $x;
                ASSERT
            } as usize);
    };
}

#[macro_export]
macro_rules! app_stack {
    ($size:expr) => {
        const stack_size: usize = $size;
        const_assert!(stack_size >= 1);
    };
}
#[macro_export]
macro_rules! app_heap {
    ($size:expr) => {
        const heap_size: usize = $size;
        const_assert!(stack_size >= 0);
    };
}

#[macro_export]
macro_rules! count_idents {
    () => {0usize};
    ($head:ident $(, $tail:ident)*) => {1usize + count_idents!($($tail),*)};
}

pub const fn check_overlap<const N: usize>(_perpherals: [usize; N]) -> bool {
    false
}
