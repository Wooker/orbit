use std::{env, fs::File, io::Write, path::PathBuf};

#[allow(unused)]
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[34;1m   {}: \x1b[0m{}", "orbit-kernel", format!($($tokens)*))
    }
}

#[allow(unused)]
fn print_env() {
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("CARGO"))
        .for_each(|f| p!("{} {}", f.0, f.1));
}

fn linker(stack_size: &String) -> String {
    format!(
        "
ENTRY(_start)

SECTIONS
{{
    .kernel.text : ALIGN(4)
    {{
        *orbit_kernel*.o(.init);
        *orbit_arch*.o(.text .text.*);
        *chip*.o(.text .text.*);
        *orbit_kernel*.o(.text .text.*);
        *orbit_bin*.o(.text .text.*);
    }} >FLASH

    .kernel.rodata : ALIGN(4)
    {{
        *orbit_kernel*.o(.rodata .rodata.*);
    }} >FLASH

    _stack_size = {stack_size};
    .kernel.stack ORIGIN(RAM) : ALIGN(4)
    {{
        . += _stack_size;
        PROVIDE(_stack_top = .);
    }} > RAM

    /* FLASH load address of .data */
    _sidata = LOADADDR(.kernel.data);

    /* RAM runtime addresses of .data */
    _sdata = ADDR(.kernel.data);
    _edata = ADDR(.kernel.data) + SIZEOF(.kernel.data);

    .kernel.data : ALIGN(4)
    {{
        *orbit_kernel*(.data, .data.*);
    }} >RAM AT>FLASH

    .kernel.bss : ALIGN(4)
    {{
        _sbss = .;
        *orbit_arch*.o(.bss .bss.* .sbss.*);
        *chip*.o(.bss .bss* .sbss.*);
        *orbit_kernel*.o(.bss .bss.* .sbss.*);
        _ebss = .;
    }} >RAM

    .kernel.heap : ALIGN(4)
    {{
        _arena_start = .;
        . += ORIGIN(RAM) + LENGTH(RAM) - .;
        _arena_end = .;
    }} >RAM
}}
"
    )
}

fn main() {
    // print_env();

    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    if let Ok(stack_size) = std::env::var("ORBIT_KERNEL_STACK_SIZE") {
        std::fs::write(out.join("kernel.x"), linker(&stack_size)).unwrap();
        println!("cargo:rustc-link-arg=--defsym=_stack_size={}", stack_size);
    } else {
        std::fs::write(out.join("kernel.x"), linker(&String::from("__stack_size"))).unwrap();
        println!("cargo:rustc-link-arg=--defsym=_stack_size=__stack_size");
    }

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
