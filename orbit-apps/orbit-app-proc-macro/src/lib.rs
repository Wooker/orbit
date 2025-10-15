#![feature(proc_macro_span)]

use std::path::Path;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Fields, Ident, ItemFn, ItemImpl, ItemStruct, Lifetime, LifetimeDef, ReturnType, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
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
    let mut struct_item = parse_macro_input!(item as ItemStruct);
    let attributes = struct_item.attrs;
    let struct_name_str = struct_item.ident.clone().to_string();
    let struct_name = struct_item.ident;

    // Add 'app lifetime
    struct_item
        .generics
        .params
        .push(syn::GenericParam::Lifetime(LifetimeDef::new(
            Lifetime::new("'app", Span::call_site()),
        )));

    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
    // let app_name = LitStr::new(
    //     format_ident!("{}", struct_name.to_string().to_lowercase())
    //         .to_string()
    //         .as_str(),
    //     struct_name.span(),
    // );

    let existing_fields_default = match struct_item.fields {
        Fields::Named(ref fields_named) => fields_named.named.iter().map(|f| {
            let name = f.ident.as_ref().expect("Expected named field");
            let ty = &f.ty;

            match ty {
                Type::Array(arr) => {
                    let arr_len = &arr.len;

                    quote! { #name: [0; #arr_len], }
                }
                _ => quote! {
                    #name: <#ty>::default(),
                },
            }
        }),
        _ => {
            return syn::Error::new_spanned(
                struct_item.fields.clone(),
                "Only structs with named fields are supported",
            )
            .to_compile_error()
            .into();
        }
    };

    let existing_fields = match struct_item.fields {
        Fields::Named(ref fields_named) => &fields_named.named,
        _ => {
            return syn::Error::new_spanned(
                struct_item.fields.clone(),
                "Only structs with named fields are supported",
            )
            .to_compile_error()
            .into();
        }
    };

    // Parsing peripherals
    let peripherals = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        quote! {
            #lower: MaybeUninit<Claimed<'app, #i>>,
        }
    });
    let claim_peripherals = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        let fn_name = format_ident!("claim_peripheral_{}", lower);
        quote! {
            fn #fn_name(p: KernelPeripherals) -> orbit_kernel::chip::pac::#i {
                syscall!(SysCall::ClaimPeripheral);
                unsafe { orbit_kernel::chip::pac::#i::steal() }
            }
        }
    });
    // println!(
    //     "{}{:?}",
    //     app_name.to_token_stream().to_string(),
    //     args.iter()
    //         .map(|p| quote! { KernelPeripheral::#p as usize}
    //             .to_token_stream()
    //             .to_string())
    //         .collect::<Vec<String>>()
    // );
    let peripherals_in_self = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        let upper = format_ident!("{}", i.to_string().to_uppercase());
        quote! {
            #lower: MaybeUninit::uninit() //peripherals.#upper.claim()
        }
    });

    let expanded = quote! {
        use orbit_app_common::{syscall,AsBytes};
        use core::{
            arch::{asm, naked_asm},
            mem::MaybeUninit,
            sync::atomic::{compiler_fence,Ordering},
            marker::PhantomData,
        };
        use orbit_kernel::{
            application::Application,
            context::Context,
            claim::{Claim, Claimed, KernelPeripherals},
            {PMP, RINGBUF_SIZE, RingbufType},
            ringbuf::RingBuf,
            message::Message,
            syscall::SysCall,
        };
        use orbit_common::const_assert;

        #[repr(C,align(4))]
        #(#attributes)*
        pub struct #struct_name #ty_generics {
            context: Context,
            _buf: RingBuf<RINGBUF_SIZE, RingbufType>,
            #existing_fields
            #(#peripherals)*
            _phantom: PhantomData<&'app ()>,
            heap: [usize; HEAP_SIZE],
            stack: [usize; STACK_SIZE],
        }

        impl #impl_generics #struct_name #ty_generics {
            pub fn new() -> Self{
                Self {
                    context: Context::new(),
                    _buf: RingBuf::new(RingbufType::default()),
                    _phantom: PhantomData,
                    heap: [0; HEAP_SIZE],
                    stack: [0; STACK_SIZE],
                    #(#existing_fields_default)*
                    #(#peripherals_in_self),*
                }
            }

            #(#claim_peripherals)*

            #[inline(always)]
            pub const fn heap_size() -> usize{
                HEAP_SIZE
            }

            #[inline(always)]
            pub const fn stack_size() -> usize{
                STACK_SIZE
            }
        }

        impl #impl_generics Application<'app> for #struct_name #ty_generics #where_clause {
            const NAME: &'app str = #struct_name_str;
            #[inline(never)]
            fn init(&mut self) {
                self.context.t0 = 0;
                self.context.sp = self as *mut Self as usize + core::mem::size_of::<Self>();
                self.context.gp = &self.context as *const Context as usize;
                self.context.ra = Self::ecall as *const fn() as usize;

                self._buf
                    .buf
                    .iter_mut()
                    .for_each(|i| *i = RingbufType::default());
                self._init();
            }

            #[inline(never)]
            fn main(&mut self) {
                let output = self._main();
                self._buf.push(Message::Reply as u8);
                output.as_bytes()
                    .iter()
                    .for_each(|byte| self._buf.push(*byte));
                self._buf.push(self._buf.termination);
                unsafe { asm!("li a0, 0;li a1, 0;") };
            }

            #[inline(never)]
            fn interrupt(&mut self) {
                self._interrupt();
                unsafe { asm!("li a0, -1; li a1, 0;") };
            }

            #[inline(always)]
            fn context(&mut self) -> usize {
                &self.context as *const Context as usize
            }

            #[inline(always)]
            fn buf(&mut self) -> usize {
                &self._buf as *const RingBuf<RINGBUF_SIZE, RingbufType> as usize
            }

            #[unsafe(naked)]
            extern "C" fn ecall() {
                unsafe {naked_asm!("ecall")};
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn orbit_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_item = parse_macro_input!(item as ItemImpl);
    let lifetime = Lifetime::new("'app", Span::call_site());

    // Add 'app lifetime
    impl_item
        .generics
        .params
        .push(syn::GenericParam::Lifetime(LifetimeDef::new(
            lifetime.clone(),
        )));

    if let syn::Type::Path(type_path) = &mut *impl_item.self_ty {
        if let Some(last_segment) = type_path.path.segments.last_mut() {
            last_segment.arguments =
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: syn::token::Lt(Span::call_site()),
                    args: {
                        let mut args = syn::punctuated::Punctuated::new();
                        args.push(syn::GenericArgument::Lifetime(lifetime));
                        args
                    },
                    gt_token: syn::token::Gt(Span::call_site()),
                });
        }
    }
    let expanded = quote! {
        #impl_item
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        #[inline(always)]
        pub fn _init(&mut self) {
            #block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_interrupt(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        #[inline(always)]
        pub fn _interrupt(&mut self) {
            #block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let sig = &function.sig;
    let block = &function.block;

    let inputs = &sig.inputs;
    let output = match &sig.output {
        ReturnType::Default => quote! { () }, // No return type (i.e. -> ())
        ReturnType::Type(_, ty) => quote! { #ty }, // ty is a Box<Type>
    };

    let expanded = quote! {
        #[inline(always)]
        pub fn _main<'a>(#inputs) -> impl AsBytes<Output = #output> + use<'a>{
            // #[forbid(unsafe_code)]
            #block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_main_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let sig = &function.sig;
    let block = &function.block;
    let span_file = &proc_macro::Span::call_site().source().file();
    let crate_name = Path::new(&span_file)
        .iter()
        .nth(0)
        .unwrap()
        .to_str()
        .unwrap();
    let main_struct = format_ident!(
        "{}",
        crate_name
            .chars()
            .enumerate()
            .map(|(i, ch)| if i == 0 {
                ch.to_uppercase().next().unwrap()
            } else {
                ch.to_lowercase().next().unwrap()
            })
            .collect::<String>()
    );

    let inputs = &sig.inputs;
    let output = match &sig.output {
        ReturnType::Default => quote! { () }, // No return type (i.e. -> ())
        ReturnType::Type(_, ty) => quote! { #ty }, // ty is a Box<Type>
    };

    let expanded = quote! {
        impl<'app> Application<'app> #main_struct<'app> {
            #[inline(always)]
            pub fn _main<'a>(#inputs) -> impl AsBytes<Output = #output> + use<'a>{
                // #[forbid(unsafe_code)]
                #block
            }
        }
    };

    TokenStream::from(expanded)
}
