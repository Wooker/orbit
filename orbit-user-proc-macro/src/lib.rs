use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Field, Fields, Ident, ItemFn, ItemStruct, Lifetime, LifetimeDef, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
};

enum OrbitKind {
    Invokable,
    Driver(Vec<Path>),
    App,
}

struct OrbitArgs {
    systems: Vec<Path>,
    kind: Vec<OrbitKind>,
}

fn to_snake_case(s: String) -> String {
    let mut snake_name = String::with_capacity(s.len() * 2);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            // Add an underscore if it's not the first character
            if i > 0 {
                snake_name.push('_');
            }
            snake_name.push(ch.to_ascii_lowercase());
        } else {
            snake_name.push(ch);
        }
    }
    snake_name
}

impl Parse for OrbitArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut systems = None;
        let mut kind: Vec<OrbitKind> = vec![];

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            match ident.to_string().as_str() {
                "systems" => {
                    if !kind.is_empty()
                        && kind.iter().find(|k| matches!(k, OrbitKind::App)).is_some()
                    {
                        return Err(syn::Error::new(ident.span(), "`app` must be used alone"));
                    } else {
                        let content;
                        syn::parenthesized!(content in input);

                        systems = Some(
                            content
                                .parse_terminated::<Path, Token![,]>(Path::parse)?
                                .into_iter()
                                .collect(),
                        );
                    }
                }

                "invokable" => {
                    if !kind.is_empty()
                        && kind.iter().find(|k| matches!(k, OrbitKind::App)).is_some()
                    {
                        return Err(syn::Error::new(ident.span(), "`app` must be used alone"));
                    } else {
                        kind.push(OrbitKind::Invokable)
                    }
                }

                "driver" => {
                    if !kind.is_empty()
                        && kind.iter().find(|k| matches!(k, OrbitKind::App)).is_some()
                    {
                        return Err(syn::Error::new(ident.span(), "`app` must be used alone"));
                    } else {
                        let content;
                        syn::parenthesized!(content in input);

                        let drivers = content
                            .parse_terminated::<Path, Token![,]>(Path::parse)?
                            .into_iter()
                            .collect();

                        kind.push(OrbitKind::Driver(drivers))
                    }
                }

                "app" => {
                    if !kind.is_empty() || systems.is_some() {
                        return Err(syn::Error::new(ident.span(), "`app` must be used alone"));
                    }

                    kind.push(OrbitKind::App);
                }

                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown orbit argument `{ident}`"),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if kind.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "expected only `app` or combination of `invokable`, `driver`, and `systems`",
            ));
        };

        Ok(Self {
            systems: systems.unwrap_or(vec![]),
            kind,
        })
    }
}
#[proc_macro_attribute]
pub fn orbit(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(attr as OrbitArgs);
    #[cfg(debug_assertions)]
    println!(
        "\r\x1b[0K\x1b[34;1m   orbit-user-proc-macro: \x1b[0m{} {:?} {:?}",
        item.to_string(),
        args.systems
            .iter()
            .map(|s| s.segments.iter().last().unwrap().ident.to_string())
            .collect::<Vec<_>>(),
        args.kind
            .iter()
            .map(|k| match k {
                OrbitKind::Invokable => "invokable",
                OrbitKind::Driver(_) => "driver",
                OrbitKind::App => "app",
            })
            .collect::<Vec<_>>()
    );

    // Process ItemFn
    let extends = if let Some(_) = args.kind.iter().find(|k| matches!(k, OrbitKind::App)) {
        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
        let item2 = item.clone();
        let fn_item = parse_macro_input!(item2 as ItemFn);
        let struct_name = format_ident!("App_{}", crate_name);
        let fn_name = &fn_item.sig.ident;

        let expanded = quote! {
            #[orbit(invokable)]
            pub struct #struct_name {}

            impl<'orbit> orbit_kernel::containers::invokable::Invokable<'orbit> for #struct_name {
                fn main(&mut self, buf: &[u8]) -> Result<usize,usize> {
                    #fn_name (buf)
                }
            }

            #fn_item
        };
        TokenStream::from(expanded)

    // Process ItemStruct
    } else {
        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();

        let item2 = item.clone();
        let struct_item = parse_macro_input!(item2 as ItemStruct);
        let attributes = struct_item.attrs.clone();
        let _struct_name_str = struct_item.ident.clone().to_string();
        let struct_name = struct_item.ident.clone();
        let struct_item_orig = struct_item.clone();
        let mut struct_item_orbit = struct_item.clone();

        let (_, ty_generics, where_clause) = struct_item_orig.generics.split_for_impl();

        // Add 'state lifetime
        let orbit_lifetime = syn::GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
            "'orbit",
            Span::call_site(),
        )));
        struct_item_orbit
            .generics
            .params
            .push(orbit_lifetime.clone());
        let (impl_generics_orbit, ty_generics_orbit, _) =
            struct_item_orbit.generics.split_for_impl();

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

        // Implement Default asnd DataAuto if either of Invokable or Driver present
        let (periph_field, periph_init) = if args
            .kind
            .iter()
            .find(|k| matches!(k, OrbitKind::Driver(_)))
            .is_some()
        {
            (
                quote! {
                    peripherals: Peripherals,
                },
                quote! {
                    peripherals: Peripherals::new(),
                },
            )
        } else {
            (quote! {}, quote! {})
        };
        let struct_new = quote! {
            #[repr(C,align(4))]
            #(#attributes)*
            pub struct #struct_name #ty_generics {
                #existing_fields
                #periph_field
            }

            impl Default for #struct_name {
                fn default() -> Self {
                    Self {
                        #(#existing_fields_default)*
                        #periph_init
                    }
                }
            }

            impl #impl_generics_orbit orbit_kernel::containers::data::DataAuto #ty_generics_orbit for #struct_name #ty_generics #where_clause {}
            impl #impl_generics_orbit orbit_kernel::containers::named::NamedAuto #ty_generics_orbit for #struct_name #ty_generics #where_clause {
                const NAME: & #orbit_lifetime str = #crate_name;
            }
        };

        let expanded = args.kind.iter().map(|k|{
        match k{
        OrbitKind::Invokable => {
            quote! {
                impl #impl_generics_orbit orbit_kernel::containers::invokable::InvokableAuto #ty_generics_orbit for #struct_name #ty_generics #where_clause {}
            }
        }

        OrbitKind::Driver(args) => {
            // Parsing peripherals
            let peripherals = args.iter().map(|i| {
                let lower = format_ident!(
                    "{}",
                    i.segments.last().unwrap().ident.to_string().to_lowercase()
                );
                quote! {
                    #lower: core::mem::MaybeUninit<#i>,
                }
            });
            let _claim_peripherals = args.iter().map(|i| {
                let _lower = format_ident!(
                    "{}",
                    i.segments.last().unwrap().ident.to_string().to_lowercase()
                );
                // let fn_name = format_ident!("claim_peripheral_{}", lower);
                quote! {
                    // fn #fn_name(&mut self) -> orbit_kernel::chip::pac::#i {
                        // self.ringbuf.push(KernelPeripherals::#i as u8);
                        // syscall!(SysCall::ClaimPeripheral);
                        // self.peripherals.#lower.write(unsafe { orbit_kernel::chip::pac::#i::steal() });
                    // }
                }
            });
            let peripherals_in_self = args
                .iter()
                .map(|i| {
                    let lower = format_ident!(
                        "{}",
                        i.segments.last().unwrap().ident.to_string().to_lowercase()
                    );
                    // let upper = format_ident!("{}", i.to_string().to_uppercase());
                    quote! {
                        #lower: core::mem::MaybeUninit::uninit(), //peripherals.#upper.claim()
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                #[repr(C,align(4))]
                pub struct Peripherals {
                    #(#peripherals)*
                }

                impl Peripherals {
                    fn new() -> Self {
                        Self {
                            #(#peripherals_in_self)*
                        }
                    }
                }

                impl #impl_generics_orbit orbit_kernel::containers::driver::DriverAuto #ty_generics_orbit for #struct_name #ty_generics #where_clause {
                    type Peripherals = Peripherals;
                }
            }
        }
        _ => {
            quote! { }
            }
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();

        let mut tokens = proc_macro2::TokenStream::new();
        expanded.into_iter().for_each(|t| tokens.extend(t));
        tokens.extend(struct_new);
        TokenStream::from(tokens)
    };

    let systems_expanded = args
        .systems
        .iter()
        .map(|s| {
            let item2 = item.clone();
            let struct_item = parse_macro_input!(item2 as ItemStruct);
            let struct_name = struct_item.ident.clone();

            let system_name = s.segments.iter().last().unwrap().ident.clone();
            let system_check_ident = format_ident!("{}Auto", system_name.to_string());
            let system_lower = format_ident!("{}", to_snake_case(system_name.to_string()));

            TokenStream::from(quote! {

            impl orbit_kernel::containers::systems::#system_lower::#system_check_ident for #struct_name {}

            })
        })
        .collect::<Vec<_>>();

    let mut extends = TokenStream::from(extends);
    systems_expanded
        .into_iter()
        .for_each(|se| extends.extend(se));

    let extends = if let None = args.kind.iter().find(|k| matches!(k, OrbitKind::App)) {
        let item2 = item.clone();
        let struct_item = parse_macro_input!(item2 as ItemStruct);
        let struct_name = struct_item.ident.clone();
        let container_impls = args
            .kind
            .iter()
            .map(|k| {
                TokenStream2::from(match k {
                    OrbitKind::Invokable => quote! {
                        invokable: Some(self.to_invokable_container()),
                    },
                    OrbitKind::Driver(_) => quote! {
                        driver:Some(self.to_driver_container()),
                    },
                    OrbitKind::App => quote! {},
                })
            })
            .collect::<Vec<_>>();

        let systems_impls = args
            .systems
            .iter()
            .map(|s| {
                let system_name = s.segments.iter().last().unwrap().ident.clone();
                let system_check_ident = format_ident!("{}Auto", system_name.to_string());
                let system_lower = format_ident!("{}", to_snake_case(system_name.to_string()));
                quote! {
                    #system_lower: Some(<Self as orbit_kernel::containers::systems:: #system_lower :: #system_check_ident >::to_system_record(self)),
                }
            })
            .collect::<Vec<_>>();
        let systems_field = if !systems_impls.is_empty() {
            quote! {
                systems: Some(orbit_kernel::containers::systems::SystemContainer {
                    #(#systems_impls)*
                    ..Default::default()
                }),
            }
        } else {
            quote! {
                systems: None,
            }
        };

        let container_impl = quote! {
            use orbit_kernel::containers::{named::NamedAuto ,driver::DriverAuto ,invokable::InvokableAuto ,data::DataAuto};
            impl<'orbit> orbit_kernel::containers::Containable<'orbit> for #struct_name {
                fn to_container(&self) -> orbit_kernel::containers::Container<'orbit> {
                    orbit_kernel::containers::Container {
                        named: Some(self.to_named_container()),
                        data: Some(self.to_data_container()),
                        #(#container_impls)*
                        #systems_field
                        ..Default::default()
                    }
                }
            }
        };
        let mut extends = TokenStream2::from(extends);
        extends.extend(container_impl);
        extends
    } else {
        TokenStream2::from(extends)
    };

    TokenStream::from(extends)
}
