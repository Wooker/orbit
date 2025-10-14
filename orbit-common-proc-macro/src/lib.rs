use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Expr, Fields, Ident, ItemFn, ItemImpl, ItemStruct, Lifetime, LifetimeDef, LitStr, ReturnType,
    Token, Type,
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
    let struct_name = struct_item.ident;

    // Add 'app lifetime
    struct_item
        .generics
        .params
        .push(syn::GenericParam::Lifetime(LifetimeDef::new(
            Lifetime::new("'app", Span::call_site()),
        )));

    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
    let app_name = LitStr::new(
        format_ident!("{}", struct_name.to_string().to_lowercase())
            .to_string()
            .as_str(),
        struct_name.span(),
    );

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
            #lower: Claimed<'app, #i>,
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
            #lower: peripherals.#upper.claim()
        }
    });

    let expanded = quote! {
        use orbit_app::application::AsBytes;
        use core::{
            arch::{asm, naked_asm},
            mem::MaybeUninit,
            sync::atomic::{compiler_fence,Ordering},
            marker::PhantomData,
        };
        use orbit_kernel::{
            arch::PMP,
            chip::pac::Peripherals,
            kernel::Kernel,
            application::{Context, Application, AppContainer, PmpEntry},
            claim::{Claim, Claimed, KernelPeripherals},
            port::{RINGBUF_SIZE, RingbufType},
            ringbuf::RingBuf,
            message::Message,
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
            heap: [usize; heap_size],
            stack: [usize; stack_size],
        }

        impl #impl_generics #struct_name #ty_generics {
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            pub fn new(peripherals: &'app mut Peripherals) -> Self{
                Self {
                    context: Context::new(),
                    _buf: RingBuf::new(RingbufType::default()),
                    _phantom: PhantomData,
                    heap: [0; heap_size],
                    stack: [0; stack_size],
                    #(#existing_fields_default)*
                    #(#peripherals_in_self),*
                }
            }

            #[inline(always)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            pub const fn heap_size() -> usize{
                heap_size
            }

            #[inline(always)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            pub const fn stack_size() -> usize{
                stack_size
            }
        }

        impl #impl_generics Application<'app> for #struct_name #ty_generics #where_clause {
            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text.main"))]
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
            #[unsafe(link_section = concat!(".", #app_name, ".text.main"))]
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
            #[unsafe(link_section = concat!(".", #app_name, ".text.main"))]
            fn interrupt(&mut self) {
                self._interrupt();
                unsafe { asm!("li a0, -1; li a1, 0;") };
            }

            #[inline(always)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn context(&mut self) -> usize {
                &self.context as *const Context as usize
            }

            #[inline(always)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn buf(&mut self) -> usize {
                &self._buf as *const RingBuf<RINGBUF_SIZE, RingbufType> as usize
            }

            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn to_container<'b>(&self, name: &'b str) -> AppContainer<'b, PMP>
            where
                Self: Sized,
                'b: 'app,
            {
                let struct_addr = self as *const Self as usize;
                let main_addr = Self::main as *const fn() as usize;
                let interrupt_addr = Self::interrupt as *const fn() as usize;
                let pmp = [PmpEntry::default(); PMP];
                let peripherals = [None; PMP];
                AppContainer::new(
                    name,
                    struct_addr,
                    main_addr,
                    interrupt_addr,
                    pmp,
                    peripherals,
                )
            }

            #[unsafe(naked)]
            #[unsafe(link_section = concat!(".", #app_name, ".text.ecall"))]
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
pub fn app_init(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr).value();
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        #[inline(always)]
        #[unsafe(link_section = concat!(".", #app_name, ".text"))]
        pub fn _init(&mut self) {
            #block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_interrupt(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr).value();
    let function = parse_macro_input!(item as ItemFn);
    let block = &function.block;

    let expanded = quote! {
        #[inline(always)]
        #[unsafe(link_section = concat!(".", #app_name, ".text.interrupt"))]
        pub fn _interrupt(&mut self) {
            #block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn app_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let app_name = parse_macro_input!(attr as LitStr);
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
        #[unsafe(link_section = concat!(".", #app_name, ".text"))]
        pub fn _main<'a>(#inputs) -> impl AsBytes<Output = #output> + use<'a>{
            // #[forbid(unsafe_code)]
            #block
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

    let inits = args
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let struct_lower = format_ident!("{}", s.to_string().to_lowercase());
            quote! {
                let mut #struct_lower = #s::new();
                #struct_lower.init();
                kernel.add_application( #i, #struct_lower.to_container(), #struct_lower.peripherals() );
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
        use orbit_kernel::{ringbuf::RingBuf, arch, chip, kernel::APPS,
        application::Application};
        use orbit_common::const_assert;

        const_assert!(#app_size <= APPS);
        const_assert!(0 #(+ #app_sizes)* < chip::RAM_SIZE);

        #[unsafe(no_mangle)]
        fn main() -> ! {
            let mut kernel = Kernel::new();
            let mut peripherals = unsafe {chip::pac::Peripherals::steal()};
            arch::riscv::register::mscratch::write(&mut kernel as *mut Kernel as usize);
            unsafe { asm!("csrr gp, mscratch") };

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
