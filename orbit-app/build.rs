#![allow(unused)]

use std::{
    env,
    fs::{write, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

#[allow(unused)]
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[34;1m   {}: \x1b[0m{}", "orbit-app", format!($($tokens)*))
    }
}

fn print_env() {
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("CARGO"))
        .for_each(|f| p!("{} {}", f.0, f.1));
}

fn write_linker_script(out: &PathBuf, name: &str, flash: bool) {
    let (content, dest_path) = if flash == true {
        println!("cargo:rustc-link-arg=-Tapp-flash-{}-link.x", name);
        (
            format!(
                "
SECTIONS
{{
    .text.apps.{0} : ALIGN(4)
    {{
        PROVIDE( _app_{0}_text_start = .);
        *(.{0}.text)

        . = ALIGN(4);
        PROVIDE( _app_{0}_text_main = .);
        *(.{0}.text.main);

        . = ALIGN(4);
        PROVIDE( _app_{0}_text_interrupt = .);
        *(.{0}.text.interrupt);

    }} >FLASH

    .rodata.apps.{0} : ALIGN(4)
    {{
        *(.{0}.rodata);
        PROVIDE( _app_{0}_text_end = .);
    }} >FLASH
}}
",
                name
            ),
            Path::new(&out).join(format!("app-flash-{}-link.x", name)),
        )
    } else {
        println!("cargo:rustc-link-arg=-Tapp-ram-{}-link.x", name);
        (
            format!(
                "
SECTIONS
{{
    .data.apps.{0} : ALIGN(4)
    {{
        PROVIDE( _app_{0}_bss_start = .);
        *(.{0}.data);
    }} >RAM AT>FLASH

    .bss.apps.{0} : ALIGN(4)
    {{
        . = ALIGN(4);
        PROVIDE( _app_{0}_bss_struct = .);
        *(.{0}.bss.struct);

        *(.{0}.bss);
        PROVIDE( _app_{0}_bss_end = .);
    }} >RAM AT>FLASH

    .stack.apps.{0} : ALIGN(4)
    {{
        *(.{0}.stack);
        PROVIDE( _{0}_stack_top = .);
    }} >RAM AT>FLASH
}}
",
                name
            ),
            Path::new(&out).join(format!("app-ram-{}-link.x", name)),
        )
    };
    write(dest_path, content).expect("Failed to write to applicatoin linker script");
}

fn main() {
    // Get crate dir
    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();

    // Get OUT_DIR and save the main linker script there
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("app.x"))
        .unwrap()
        .write_all(include_bytes!("app.x"))
        .expect("Could not find app.x");

    // List features
    let mut features: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            // Check for the feature-related environment variables (e.g., CARGO_FEATURE_FOO)
            if key.starts_with("CARGO_FEATURE_") {
                Some(key.replace("CARGO_FEATURE_", "").to_lowercase())
            } else {
                None
            }
        })
        .collect();

    // Write linker scripts for apps
    let apps: Vec<String> = features
        .iter_mut()
        .filter_map(|s| s.starts_with("app_").then_some(s.split_off(4)))
        .collect();
    p!("Apps: {:?}", apps);
    for app in apps.iter() {
        write_linker_script(out, app, true);
        write_linker_script(out, app, false);
    }

    // Add OUT_DIR to link search
    println!("cargo:rustc-link-search={}", out.display());

    // Metadata for dependent crates
    println!("cargo::metadata=OUT_DIR={}", out.display());

    // Recompile if this file changes
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
