use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Fields, Ident, ItemFn, ItemStruct, LitStr, ReturnType, Token,
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
    let attributes = struct_item.attrs;
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

    let existing_fields_default = match struct_item.fields {
        Fields::Named(ref fields_named) => fields_named.named.iter().map(|f| {
            let name = f.ident.as_ref().expect("Expected named field");
            let ty = &f.ty;
            quote! {
                #name: <#ty>::default()
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
            #lower: Claimed<'static, #i>,
        }
    });
    let peripherals_default = args.iter().map(|i| {
        let lower = format_ident!("{}", i.to_string().to_lowercase());
        quote! {
            #lower: unsafe {Kernel::instance().claim().unwrap_unchecked()}
        }
    });

    let _init = quote! {
        #[inline(never)]
        #[unsafe(link_section = concat!(".", #app_name, ".text"))]
        pub fn _init(&mut self) {
            self.context = Context::new();
            self.context.t0 = 0;
            self.context.sp = self as *const Self as usize + core::mem::size_of::<Self>() + Self::stack_size();
            self.context.gp = &self.context as *const Context as usize;
            self.context.ra = Self::ecall as *const fn() as usize;

            self._buf = RingBuf::new(b'\0');
            self._buf
                .buf
                .iter_mut()
                .for_each(|i| *i = RingbufType::default());
        }
    };

    let expanded = quote! {
        use crate::application::AsBytes;
        use core::{
            arch::{asm, naked_asm},
            mem::MaybeUninit,
            sync::atomic::{compiler_fence,Ordering},
        };
        use orbit_kernel::{
            kernel::Kernel,
            application::{Context, Application},
            claim::{Claim, Claimed, KernelPeripherals},
            port::{RINGBUF_SIZE, RingbufType, ringbuf::RingBuf, message::Message},
        };

        #[repr(C,align(4))]
        #(#attributes)*
        pub struct #struct_name #ty_generics {
            context: Context,
            pub _buf: RingBuf<RINGBUF_SIZE, RingbufType>,
            #existing_fields
            #(#peripherals)*
        }

        impl #impl_generics #struct_name #ty_generics {
            pub fn new() -> Self {
                Self {
                    context: Context::new(),
                    _buf: RingBuf::new(0),
                    #(#existing_fields_default),*
                    #(#peripherals_default),*
                }
            }
            #_init
        }

        impl #impl_generics Application for #struct_name #ty_generics #where_clause {
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
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn context(&mut self) -> Context {
                self.context
            }

            #[inline(never)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn buf(&mut self) -> &mut RingBuf<RINGBUF_SIZE, RingbufType> {
                &mut self._buf
            }

            #[inline(always)]
            #[unsafe(link_section = concat!(".", #app_name, ".text"))]
            fn stack_size() -> usize{
                stack_size
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
    let sig = &function.sig;
    let block = &function.block;

    let inputs = &sig.inputs;
    let output = match &sig.output {
        ReturnType::Default => quote! { () }, // No return type (i.e. -> ())
        ReturnType::Type(_, ty) => quote! { #ty }, // ty is a Box<Type>
    };

    let expanded = quote! {
        #[inline(never)]
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
            let struct_lower_ptr = format_ident!("{}_ptr", struct_lower);
            quote! {
                let #struct_lower_ptr = next_addr as *mut MaybeUninit<#s>;
                unsafe {
                    (*#struct_lower_ptr).as_mut_ptr().write(
                        #s::new()
                    )
                };
                let #struct_lower = unsafe { &mut *(*(#struct_lower_ptr)).assume_init_mut() };
                #struct_lower.init();
                kernel.add_application(
                    #i,
                    stringify!(#struct_lower),
                    #struct_lower,
                    next_addr, //unsafe { &#s as *const #s as usize },
                    #s::main as usize,
                    #s::interrupt as usize,
                    // unsafe { &mut (#struct_lower).context() as *mut _ },
                    // #struct_upper.buf(),
                    // unsafe { &mut (#struct_lower)._buf as *mut _ },
                );
                next_addr += core::mem::size_of::<#s>() + #s::stack_size();
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let expanded = quote! {
        use core::{arch::asm,mem::MaybeUninit};

        use orbit_kernel::application::{Application, Context};
        use orbit_kernel::port::ringbuf::RingBuf;
        use orbit_kernel::arch;

        unsafe extern "C" {
            static _kernel_start: usize;
        }

        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".text.bin")]
        unsafe fn main() {
            let kernel_ptr = unsafe { &_kernel_start as *const usize as *mut MaybeUninit<Kernel> };
            unsafe { (*kernel_ptr).as_mut_ptr().write(Kernel::new()) };
            let kernel = unsafe { &mut *(*kernel_ptr).assume_init_mut() };

            let mut next_addr = kernel_ptr as usize + core::mem::size_of::<Kernel>();

            #(#inits)*

            arch::riscv::register::mscratch::write(kernel as *mut Kernel as usize);
            unsafe { asm!("csrr gp, mscratch") };
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
