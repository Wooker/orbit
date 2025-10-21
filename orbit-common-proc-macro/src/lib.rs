use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

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

    let inits = args
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let struct_lower = format_ident!("{}", s.to_string().to_lowercase());
            quote! {
                let mut #struct_lower = #s::new();
                // #struct_lower.init();
                kernel.add_application( #i, #struct_lower.to_container(), & #struct_lower.stack as *const usize as usize );
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let app_sizes = args
        .iter()
        .map(|s| {
            quote! {
                (core::mem::size_of::<#s>() + #s::stack_size() + #s::heap_size())
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let app_size = args.iter().count();

    let expanded = quote! {
        use core::{arch::asm,mem::MaybeUninit};

        // use orbit_kernel::application::{Application, Context, PmpEntry, AppContainer};
        use orbit_kernel::{ringbuf::RingBuf, arch, chip, kernel::{asm, APPS},
        application::Application};
        use orbit_common::const_assert;

        const_assert!(#app_size <= APPS);
        const_assert!(0 #(+ #app_sizes)* < chip::RAM_SIZE);

        #[unsafe(no_mangle)]
        fn main() -> ! {
            let mut kernel = Kernel::new();
            // let mut peripherals = unsafe {chip::pac::Peripherals::steal()};
            arch::riscv::register::mscratch::write(&mut kernel as *mut Kernel as usize);
            unsafe {
                asm!("csrr gp, mscratch");
                asm::save_context()
            };

            #(#inits)*

            unsafe { Kernel::initialize_finish() }
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
