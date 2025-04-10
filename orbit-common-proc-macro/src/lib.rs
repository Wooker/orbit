use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Fields, Ident, ItemFn, ItemStruct, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
};

struct OrbitAppArgs {
    peripherals: Punctuated<Ident, Token![,]>, // Structs
}

impl Parse for OrbitAppArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(OrbitAppArgs {
            peripherals: input.parse_terminated(Ident::parse)?,
        })
    }
}

#[proc_macro_attribute]
pub fn orbit_app(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args: Vec<Ident> = parse_macro_input!(attr as OrbitAppArgs)
        .peripherals
        .into_iter()
        .collect();
    let struct_item = parse_macro_input!(item as ItemStruct);
    let struct_name = struct_item.ident;
    let mut static_generics = struct_item.generics.clone();
    for param in &mut static_generics.params {
        if let syn::GenericParam::Lifetime(lifetime_def) = param {
            // Replace the lifetime ident with `'static`
            *lifetime_def = parse_quote!('static);
        }
    }
    let (_, ty_static_generics, _) = static_generics.split_for_impl();
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
    let app_name = LitStr::new(
        format_ident!("{}", struct_name.to_string().to_lowercase())
            .to_string()
            .as_str(),
        struct_name.span(),
    );

    let static_name = format_ident!("{}", struct_name.to_string().to_uppercase());

    let existing_fields = match struct_item.fields {
        Fields::Named(fields_named) => fields_named.named,
        _ => {
            return syn::Error::new_spanned(
                struct_item.fields.clone(),
                "Only structs with named fields are supported",
            )
            .to_compile_error()
            .into();
        }
    };

    let peripherals = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        quote! {
            #lower: MaybeUninit<Claimed<'static, #i>>,
        }
    });

    let app_text_start = Ident::new(
        &format!("_app_{}_text_start", app_name.value()),
        struct_name.span(),
    );
    let app_text_end = Ident::new(
        &format!("_app_{}_text_end", app_name.value()),
        struct_name.span(),
    );
    let app_bss_start = Ident::new(
        &format!("_app_{}_bss_start", app_name.value()),
        struct_name.span(),
    );
    let app_bss_end = Ident::new(
        &format!("_app_{}_bss_end", app_name.value()),
        struct_name.span(),
    );
    let app_text_main = Ident::new(
        &format!("_app_{}_text_main", app_name.value()),
        struct_name.span(),
    );
    let app_bss_struct = Ident::new(
        &format!("_app_{}_bss_struct", app_name.value()),
        struct_name.span(),
    );

    let _init = quote! {
        #[unsafe(link_section = concat!(".", #app_name, ".text"))]
        pub fn _init(&mut self) {
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

            self._buf
                .buf
                .iter_mut()
                .for_each(|i| *i = RingbufType::default());
        }
    };

    let expanded = quote! {

        use crate::application::Application;
        use core::{
            arch::{asm, naked_asm},
            mem::MaybeUninit,
            sync::atomic::compiler_fence,
        };
        use orbit_kernel::{
            application::Context,
            claim::{Claim, Claimed, KernelPeripherals},
            port::{RINGBUF_SIZE, RingbufType, ringbuf::RingBuf},
        };

        #[used]
        #[unsafe(no_mangle)]
        #[unsafe(link_section=concat!(".", #app_name, ".bss.struct"))]
        static mut #static_name: MaybeUninit<#struct_name #ty_static_generics> = MaybeUninit::uninit();

        #[repr(C,align(4))]
        pub struct #struct_name #ty_generics {
            context: Context,
            _buf: RingBuf<RINGBUF_SIZE, RingbufType>,
            #existing_fields
            #(#peripherals)*
        }

        impl #impl_generics #struct_name #ty_generics {
            #_init
        }

        impl #impl_generics Application for #struct_name #ty_generics #where_clause {
            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn _main(&mut self) {
                unsafe { asm!("li a0, 0;li a1, 0;") };
            }

            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
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

#[proc_macro_attribute]
pub fn app_init(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr).value();
    let function = parse_macro_input!(item as ItemFn);
    let fn_name = &function.sig.ident;
    let block = &function.block;

    let expanded = quote! {
        #[unsafe(link_section = concat!(".", #app_name, ".text"))]
        pub fn #fn_name(&mut self) {
            self._init();
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
    let app_name = parse_macro_input!(attr as LitStr);
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        #[inline(never)]
        #[unsafe(link_section = concat!(".", #app_name, ".text.main"))]
        pub fn main(&mut self) {
            // #[forbid(unsafe_code)]
            #block
            self._main();
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
