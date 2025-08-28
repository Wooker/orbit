#![no_std]
#![no_main]

pub use paste;

#[macro_export]
macro_rules! feature_mod {
    ($chip:literal) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            pub mod [<$chip>];
        }
    };
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
    ($chip:literal) => {
        $crate::paste::paste! {
            #[cfg(feature = $chip)]
            pub mod [<$chip>];
            #[cfg(feature = $chip)]
            pub use [<$chip>]::*;
        }
    };
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

#[macro_export]
macro_rules! feature_mod_use_mutual {
    ($name:ident, $($chip:literal),+ $(,)?) => {
        $crate::paste::paste! {
            #[cfg(any($(feature = $chip),+))]
            pub mod [<$name>];
            #[cfg(any($(feature = $chip),+))]
            pub use [<$name>]::*;
        }
    };
}

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        // const _: [(); 0 - !{
        //     const ASSERT: bool = $x;
        //     ASSERT
        // } as usize] = [];
        const _: usize = 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize;
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
