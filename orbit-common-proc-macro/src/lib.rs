use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn app_init(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr).value();
    let function = parse_macro_input!(item as ItemFn);
    let fn_name = &function.sig.ident;
    let block = &function.block;

    let app_text_start = Ident::new(&format!("_app_{}_text_start", app_name), fn_name.span());
    let app_text_end = Ident::new(&format!("_app_{}_text_end", app_name), fn_name.span());
    let app_bss_start = Ident::new(&format!("_app_{}_bss_start", app_name), fn_name.span());
    let app_bss_end = Ident::new(&format!("_app_{}_bss_end", app_name), fn_name.span());
    let app_text_main = Ident::new(&format!("_app_{}_text_main", app_name), fn_name.span());
    let app_bss_struct = Ident::new(&format!("_app_{}_bss_struct", app_name), fn_name.span());

    let expanded = quote! {
        #[unsafe(link_section = concat!(".", #app_name, ".text"))]
        pub fn #fn_name(&mut self) {
            unsafe extern "C" {
                static #app_text_start: usize;
                static #app_text_end: usize;
                static #app_bss_start: usize;
                static #app_bss_end: usize;
                static #app_text_main: usize;
                static #app_bss_struct: usize;
            }

            let provides = unsafe {
                &#app_text_end as *const usize as usize
                    | &#app_text_start as *const usize as usize
                    | &#app_text_end as *const usize as usize
                    | &#app_bss_start as *const usize as usize
                    | &#app_bss_end as *const usize as usize
                    | &#app_text_main as *const usize as usize
                    | &#app_bss_struct as *const usize as usize
            };

            self.context = Context::new();
            self.context.t0 = provides;
            compiler_fence(core::sync::atomic::Ordering::SeqCst);

            self.context.t0 = 0;
            self.context.sp = unsafe { STACK.last().unwrap_unchecked() as *const usize as usize + 0x4 };
            self.context.gp = &self.context as *const Context as usize;
            self.context.ra = Self::ecall as *const fn() as usize;

            #block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_interrupt(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr).value();
    let function = parse_macro_input!(item as ItemFn);
    let fn_name = &function.sig.ident;
    let block = &function.block;

    let expanded = quote! {
        #[unsafe(link_section = concat!(".", #app_name, ".text.interrupt"))]
        pub fn #fn_name(&mut self) {
            #block
            unsafe { asm!("li a0, -1; li a1, 0;") };
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr).value();
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        impl Application for Wfi {
            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text.main"))]
            fn main(&mut self) {
                #block
                unsafe { asm!("li a0, 0;li a1, 0;") };
            }

            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text.interrupt"))]
            fn context(&self) -> Context {
                self.context
            }

            #[naked]
            #[unsafe(link_section = concat!(".", #app_name, ".text.ecall"))]
            extern "C" fn ecall() {
                unsafe {naked_asm!("ecall")};
            }
        }
    };

    TokenStream::from(expanded)
}
