#![feature(proc_macro_span)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn orbit_app(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();

    let mut func_sig = func.sig;
    let func_block = func.block;

    func_sig.ident = format_ident!("{}_main", crate_name);
    let crate_func = format_ident!("{}", crate_name);
    let func_ecall = format_ident!("{}_ecall", crate_name);

    let call_main = format!("call {}", func_sig.ident);
    let call_ecall = format!("j {}_ecall", crate_name);

    TokenStream::from(quote! {
        use core::arch::naked_asm;
        use orbit_kernel as _;

        #[unsafe(no_mangle)]
        #[unsafe(naked)]
        pub extern "C" fn #crate_func () -> ! {
            naked_asm!(
                #call_main,
                #call_ecall,
            )
        }

        #[unsafe(no_mangle)]
        #[unsafe(naked)]
        extern "C" fn #func_ecall () -> ! {
            naked_asm!("li a0, 1; li a1, 0; ecall;")
        }

        #[unsafe (no_mangle)]
        #func_sig
        #func_block
    })
}
