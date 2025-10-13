#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

#[macro_export]
macro_rules! orbit_main {
    // This pattern matches when the macro is called with one or more identifiers.
    ( $( $ident:ident ),* ) => {
        use orbit_common_proc_macro::orbit_main_attribute;
        use orbit_kernel::kernel::Kernel;

        #[orbit_main_attribute($($ident,)*)]
        #[unsafe(no_mangle)]
        fn main() {}
    };
}
