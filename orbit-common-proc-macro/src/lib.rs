use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Ident, ItemFn, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct AppMainArgs {
    app_name: LitStr,   // "app"
    _comma: Token![,],  // Comma separator
    struct_name: Ident, // App
}

impl Parse for AppMainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AppMainArgs {
            app_name: input.parse()?,    // Parse "app"
            _comma: input.parse()?,      // Parse the comma
            struct_name: input.parse()?, // Parse App
        })
    }
}

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
    let attr_args = parse_macro_input!(attr as AppMainArgs);
    let app_name = attr_args.app_name;
    let struct_name = attr_args.struct_name;
    let struct_ident = format_ident!("{}", struct_name);
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        impl Application for #struct_ident {
            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text.main"))]
            fn main(&mut self) {
                // #[forbid(unsafe_code)]
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

struct OrbitMainArgs {
    structs: Punctuated<Ident, Token![,]>, // Structs
}

impl Parse for OrbitMainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(OrbitMainArgs {
            structs: input.parse_terminated(Ident::parse)?,
        })
    }
}

#[proc_macro_attribute]
pub fn orbit_main_attribute(attr: TokenStream, _item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as OrbitMainArgs);
    let args: Vec<Ident> = attr_args.structs.into_iter().collect();

    let statics = args
        .iter()
        .map(|s| {
            let struct_upper = format_ident!("{}", s.to_string().to_uppercase());
            quote! {
                static mut #struct_upper: #s;
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let inits = args
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let struct_upper = format_ident!("{}", s.to_string().to_uppercase());
            let _struct_lower = format_ident!("{}", s.to_string().to_lowercase());
            quote! {
                #struct_upper.init();
                KERNEL.add_application(
                    #i,
                    unsafe { &#struct_upper as *const #s as usize },
                    #s::main as usize,
                    Some(#s::interrupt as usize),
                    #struct_upper.context(),
                );
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let expanded = quote! {
        unsafe extern "Rust" {
            static mut KERNEL: Kernel<'static>;
            #(#statics)*
        }

        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".text.bin")]
        unsafe fn main() -> ! {
            KERNEL.clock.freeze();

            #(#inits)*

            KERNEL.initialize();
            panic!();
        }
    };

    TokenStream::from(expanded)
}
