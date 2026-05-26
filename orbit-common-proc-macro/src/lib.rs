use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct OrbitMainArgs {
    apps: Vec<Ident>,
    drivers: Vec<Ident>,
}

impl Parse for OrbitMainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let apps_kw: Ident = input.parse()?;
        if apps_kw != "apps" {
            return Err(input.error("expected `apps`"));
        }

        let content;
        syn::parenthesized!(content in input);

        let apps = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect::<Vec<_>>();

        input.parse::<Token![,]>()?;

        let drivers_kw: Ident = input.parse()?;
        if drivers_kw != "drivers" {
            return Err(input.error("expected `drivers`"));
        }

        let content;
        syn::parenthesized!(content in input);

        let drivers = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect::<Vec<_>>();

        Ok(Self { apps, drivers })
    }
}

#[proc_macro_attribute]
pub fn orbit_main_attribute(attr: TokenStream, _item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as OrbitMainArgs);

    let apps_args = attr_args.apps;
    let drivers_args = attr_args.drivers;

    let apps = apps_args
        .iter()
        .enumerate()
        .map(|(_, s)| {
            let app = format_ident!("{}", s.to_string().to_lowercase());
            quote! {
                kernel.add_application(
                    AppContainer::new(
                        stringify!(#app),
                        None,
                        0,
                        #app::#app as *const () as usize,
                        0,
                    )
                );
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let drivers = drivers_args
        .iter()
        .enumerate()
        .map(|(_, s)| {
            let struct_lower = format_ident!("{}", s.to_string().to_lowercase());
            let s_pin = format_ident!("_{}_pin", s.to_string().to_lowercase());
            quote! {
                let mut #struct_lower = #s::new();
                let #s_pin = if let Ok(#struct_lower) = Box::try_new(#struct_lower) {
                    let pin = Box::into_pin(#struct_lower);
                    kernel.add_driver(AppContainer::new(
                        stringify!(#struct_lower),
                        Some(pin.as_ref().get_ref() as *const #s as usize),
                        #s::init as *const () as usize,
                        #s::main as *const () as usize,
                        #s::interrupt as *const () as usize,
                    ));
                    Some(pin)
                } else {
                    None
                };
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let driver_sizes = drivers_args
        .iter()
        .map(|s| {
            quote! {
                (core::mem::size_of::<#s>())
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let expanded = quote! {
        use core::{arch::asm,mem::MaybeUninit};

        extern crate alloc;
        use alloc::boxed::Box;

        use orbit_bin::{
            arch,
            chip,
            kernel::{asm, Kernel, APPS},
            application::Application,
            application_container::AppContainer,
        };
        use orbit_bin::const_assert;

        const_assert!(0 #(+ #driver_sizes)* < chip::RAM_SIZE);

        #[unsafe(no_mangle)]
        fn main() -> ! {
            let mut kernel = Kernel::new();
            arch::riscv::register::mscratch::write(&kernel as *const Kernel as usize);
            unsafe {
                asm!("csrr gp, mscratch");
                asm::save_context();
            }

            #(#apps)*
            #(#drivers)*

            kernel.setup_event_loop()
        }
    };

    TokenStream::from(expanded)
}

struct PortBinding {
    ident: Ident,
    _eq_token: Token![=],
    expr: Expr,
}

impl Parse for PortBinding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PortBinding {
            ident: input.parse()?,
            _eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn define_ports(input: TokenStream) -> TokenStream {
    let bindings =
        parse_macro_input!(input with Punctuated::<PortBinding, Token![,]>::parse_terminated);

    let mut uses = vec![];
    let mut consts = vec![];
    let mut ptrs = vec![];

    for (i, binding) in bindings.iter().enumerate() {
        let lower = Ident::new(
            &binding.ident.to_string().to_lowercase(),
            binding.ident.span(),
        );
        let upper = Ident::new(
            &binding.ident.to_string().to_uppercase(),
            binding.ident.span(),
        );

        let int_num = &binding.expr;

        let port_struct = format_ident!("PortPeripheral{}", i + 1);
        let ptr_const = format_ident!("PORT_PTR{}", i + 1);

        uses.push(quote! {
            pub use chip::pac::#lower::RegisterBlock as #port_struct;
        });

        consts.push(quote! {
            pub const #ptr_const: *const PortPeripheral = chip::pac::#upper::PTR;
        });

        ptrs.push(quote! {(#ptr_const, #int_num, PortKinds::#upper)});
    }
    let len = ptrs.len();

    let expanded = quote! {
        #(#consts)*

        pub const PORT_NUM: usize = #len;

        pub struct PortInterruptTable(pub [(*const PortPeripheral, usize, PortKinds); PORT_NUM]);
        unsafe impl Sync for PortInterruptTable {}

        pub const PORT_INTERRUPTS: PortInterruptTable =
            PortInterruptTable([#(#ptrs),*]);
    };

    TokenStream::from(expanded)
}
