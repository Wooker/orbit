#![feature(proc_macro_span)]

use std::path::Path;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    Field, Fields, Ident, ItemFn, ItemImpl, ItemStruct, Lifetime, LifetimeDef, ReturnType, Token,
    Type,
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
        Fields::Named(ref named) => named
            .named
            .iter()
            .enumerate()
            .map(|(_, f)| {
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
            })
            .collect::<Vec<_>>(),
        Fields::Unnamed(ref unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                // let name = f.ident.as_ref().expect("Expected named field");
                let ty = &f.ty;

                match ty {
                    Type::Array(arr) => {
                        let arr_len = &arr.len;

                        quote! { #i: [0; #arr_len], }
                    }
                    _ => quote! {
                        #i: <#ty>::default(),
                    },
                }
            })
            .collect::<Vec<_>>(),
        Fields::Unit => {
            vec![]
        }
    };

    let existing_fields = match struct_item.fields {
        Fields::Named(ref fields_named) => &fields_named.named,
        Fields::Unnamed(ref fields_unnamed) => &fields_unnamed.unnamed,
        Fields::Unit => &Punctuated::<Field, Comma>::new(),
    };

    // Parsing peripherals
    let peripherals = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        quote! {
            #lower: MaybeUninit<#i>,
        }
    });
    let claim_peripherals = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        // let fn_name = format_ident!("claim_peripheral_{}", lower);
        quote! {
            // fn #fn_name(&mut self) -> orbit_kernel::chip::pac::#i {
                // self.ringbuf.push(KernelPeripherals::#i as u8);
                // syscall!(SysCall::ClaimPeripheral);
                self.peripherals.#lower.write(unsafe { orbit_kernel::chip::pac::#i::steal() });
            // }
        }
    });
    let peripherals_assume_init = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        // let fn_name = format_ident!("claim_peripheral_{}", lower);
        quote! {
            // fn #fn_name(&mut self) -> orbit_kernel::chip::pac::#i {
                let #lower = unsafe {self.peripherals.#lower.assume_init_mut() };
            // }
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
    let peripherals_in_self = args
        .iter()
        .map(|i| {
            let lower = format_ident!("{}", i.to_string().to_lowercase());
            // let upper = format_ident!("{}", i.to_string().to_uppercase());
            quote! {
                #lower: MaybeUninit::uninit(), //peripherals.#upper.claim()
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        use orbit_app_common::{syscall,AsBytes};
        use core::{
            arch::{asm, naked_asm},
            mem::MaybeUninit,
            sync::atomic::{compiler_fence,Ordering},
            marker::PhantomData,
        };
        use orbit_kernel::{
            spaceport::message::Message,
            application::Application,
            context::Context,
            claim::{Claim, Claimed, KernelPeripherals},
            {PMP, RINGBUF_SIZE},
            ringbuf::RingBuf,
            syscall::SysCall,
        };
        use orbit_common::const_assert;

        #[repr(C,align(4))]
        #(#attributes)*
        pub struct Peripherals<'app> {
            #(#peripherals)*
            _phantom: PhantomData<&'app ()>,
        }

        #[repr(C,align(4))]
        #(#attributes)*
        pub struct #struct_name #ty_generics {
            #existing_fields
            peripherals: Peripherals<'app>,
            _phantom: PhantomData<&'app ()>,
        }

        impl #impl_generics #struct_name #ty_generics {
            pub fn new() -> Self{
                let mut app = Self {
                    #(#existing_fields_default)*
                    peripherals: Peripherals {
                        _phantom: PhantomData,
                        #(#peripherals_in_self)*
                    },
                    _phantom: PhantomData,
                };
                // app.context.ra = Self::ecall as *const fn() as usize;
                // app.context.sp = &app.stack as *const [usize; STACK_SIZE] as usize + STACK_SIZE;
                // app.context.gp = &app as *const Self as usize;

                app
            }
        }


        impl #impl_generics Application<'app> for #struct_name #ty_generics #where_clause {
            type Peripherals = Peripherals<'app>;
            const NAME: &'app str = #struct_name_str;
            #[inline(never)]
            fn init(&mut self) {
                // self.context.sp = self as *mut Self as usize + core::mem::size_of::<Self>();
                // self.context.gp = &self.context as *const Context as usize;
                // self.context.t0 = 0;
                // self.context.sp = self as *mut Self as usize + core::mem::size_of::<Self>();
                // self.context.gp = &self.context as *const Context as usize;
                // self.context.ra = ecall_addr;

                // self.ringbuf
                //     .buf
                //     .iter_mut()
                //     .for_each(|i| *i = 32);
                #(#claim_peripherals)*
                #(#peripherals_assume_init)*

                self._init();
                unsafe { asm!("li a0, 0;li a1, 0; ecall") };
            }

            #[inline(never)]
            fn main(&mut self, buf: &[u8]) {
                // #(#peripherals_assume_main)*
                let output = Self::_main(self, buf);
                // self.ringbuf.push(self.ringbuf.termination);
                unsafe { asm!("li a0, 1;li a1, 0; ecall;") };
            }

            #[inline(never)]
            fn interrupt(&mut self) {
                self._interrupt();
                unsafe { asm!("li a0, 2; li a1, 0;ecall;") };
            }

            #[unsafe(naked)]
            extern "C" fn ecall(){
                naked_asm!("ecall")
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
        #[inline(never)]
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
