#![allow(unused)]

use regex::Regex;
use std::{
    env,
    fs::{self, File, write},
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[34;1m   {}: \x1b[0m{}", "orbit-bin", format!($($tokens)*))
    }
}

fn print_env() {
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("CARGO"))
        .for_each(|f| p!("{} {}", f.0, f.1));
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("DEP"))
        .for_each(|f| p!("{} {}", f.0, f.1));
}

fn link_script_from_feature(feature: &String, script_name: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    let script_path =
        PathBuf::from_str(format!("src/{}/{}", feature.as_str(), script_name).as_str()).unwrap();
    p!("Script path {:?}", script_path);
    let mut file = File::open(script_path).unwrap();
    file.read_to_end(&mut buf).unwrap();
    buf
}

fn write_linker_script(out: &PathBuf, name: String, flash: bool) {
    let (content, dest_path) = if flash == true {
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

        . = ALIGN(4);
        PROVIDE( _app_{0}_text_ecall = .);
        *(.{0}.text.ecall);

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
    // Set verbosity
    println!("cargo:rustc-link-arg={}", "--verbose");
    println!("cargo:rustc-link-arg={}", "--error-limit=0");

    // Get crate dir
    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();

    // Get features
    let features: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            // Check for the feature-related environment variables (e.g., CARGO_FEATURE_FOO)
            if key.starts_with("CARGO_FEATURE_") {
                Some(key.replace("CARGO_FEATURE_", "").to_lowercase())
            } else {
                None
            }
        })
        .collect();
    if features.len() != 1 {
        panic!("Use only one feature for the chip.");
    }
    p!("Chip: {}", features[0]);

    // Get OUT_DIR and save the main linker script there
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .expect("Could not find link.x");

    // Get OUT_DIR of orbit-app crate
    let app_out_dir_var = "DEP_ORBIT_APP_OUT_DIR";
    let app_out = &PathBuf::from(
        env::var_os(app_out_dir_var)
            .expect(format!("Cannot find the {} env var", app_out_dir_var).as_str()),
    );

    // Delete all linker files in app OUT_DIR
    for entry in fs::read_dir(app_out).unwrap() {
        let e = entry.unwrap();
        let file_name = e.file_name().into_string().unwrap();
        if file_name.starts_with("app") && file_name.ends_with("link.x") {
            fs::remove_file(e.path()).expect("Could not delete app link file");
        }
    }

    // Read bin main contents
    let file = format!(
        "src/bin/{}.rs",
        features.get(0).expect("No features provided")
    );
    let source_path = Path::new(&file);
    let contents = std::fs::read_to_string(source_path).expect("Could not read source file");

    // Capture applications in bin main content
    let re = Regex::new(r"orbit_main!\((\w+(?:,\s*\w+)*)?\);").unwrap();
    if let Some(caps) = re.captures(&contents) {
        let args = caps.get(1).map_or("", |m| m.as_str());
        if !args.is_empty() {
            let names = args
                .split(",")
                .map(|s| s.trim().to_string().to_lowercase())
                .collect::<Vec<String>>();
            p!("Apps: {:?}", names);
            for name in names {
                write_linker_script(app_out, name.clone(), true);
                write_linker_script(app_out, name, false);
            }
        } else {
            p!("Apps empty: []");
        }
    } else {
        println!("cargo:warning=orbit_main! macro not found");
    }

    // Add linker scripts of memory and kernel
    println!("cargo:rustc-link-arg={}", "-Tmemory.x");
    println!("cargo:rustc-link-arg={}", "-Tkernel.x");

    // Add linker scripts of applications
    for entry in fs::read_dir(app_out).unwrap() {
        let e = entry.unwrap();
        let file_name = e.file_name().into_string().unwrap();
        if file_name.starts_with("app") && file_name.ends_with("link.x") {
            // p!("Including linker script: {}", file_name);
            println!("cargo:rustc-link-arg=-T{}", e.file_name().to_str().unwrap());
        }
    }

    // Add final linker scripts
    // println!("cargo:rustc-link-arg={}", "-Tapp.x");
    println!("cargo:rustc-link-arg={}", "-Tlink.x");

    // Create the binary map
    println!(
        "cargo:rustc-link-arg={}{}/{}",
        "-Map=", crate_dir, "bin.map"
    );

    // Add OUT_DIR to link search
    println!("cargo:rustc-link-search={}", out.display());

    // Recompile if this file changes
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
    println!(
        "cargo:rerun-if-changed={}/bin/{}.rs",
        crate_dir, features[0]
    );
}
