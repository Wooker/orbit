#![allow(unused)]

use std::{
    env,
    fs::{self, File, read_dir, write},
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

fn write_linker_script(out: &PathBuf, name: String) {
    let (content, dest_path) = (
        format!(
            "
SECTIONS
{{
    .apps.{0}.text : ALIGN(4)
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

        PROVIDE( _app_{0}_text_end = .);
    }} >FLASH
    .apps.{0}.rodata : ALIGN(4)
    {{
        *(.{0}.rodata)
    }}
    .apps.{0}.data : ALIGN(4)
    {{
        *(.{0}.data)
    }}
    .apps.{0}.bss : ALIGN(4)
    {{
        *(.{0}.bss)
    }}
}}
",
            name
        ),
        Path::new(&out).join(format!("app-{}-flash-link.x", name)),
    );
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
    p!("Chip: {}", features[0]);

    // Get OUT_DIR and save the main linker script there
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .expect("Could not find link.x");

    // Get OUT_DIR of orbit-app crate
    // let app_out_dir_var = "DEP_ORBIT_APP_OUT_DIR";
    // let app_out = &PathBuf::from(
    //     env::var_os(app_out_dir_var)
    //         .expect(format!("Cannot find the {} env var", app_out_dir_var).as_str()),
    // );

    // Delete all linker files in app OUT_DIR
    // for entry in fs::read_dir(app_out).unwrap() {
    //     let e = entry.unwrap();
    //     let file_name = e.file_name().into_string().unwrap();
    //     if file_name.starts_with("app") && file_name.ends_with("link.x") {
    //         fs::remove_file(e.path()).expect("Could not delete app link file");
    //     }
    // }

    // Add linker scripts of memory and kernel
    println!("cargo:rustc-link-arg={}", "-Tmemory.x");
    println!("cargo:rustc-link-arg={}", "-Tkernel.x");

    // let mut dir: Vec<_> = fs::read_dir(app_out)
    //     .unwrap()
    //     .filter_map(Result::ok)
    //     .collect();

    // dir.sort_by_key(|entry| entry.file_name());

    // Add linker scripts of applications
    // for e in dir {
    //     let file_name = e.file_name().into_string().unwrap();
    //     if file_name.starts_with("app") && file_name.ends_with("link.x") {
    //         // p!("Including linker script: {}", file_name);
    //         println!("cargo:rustc-link-arg=-T{}", e.file_name().to_str().unwrap());
    //     }
    // }
    for dep in env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("DEP") && !key.contains("COMPILER"))
    {
        p!("asdasdasdasd");
        p!("{}", dep.0);
        let delim_pos = dep.1.find('=').expect("Incorrect metadata format");
        let (key, out_dir) = dep.1.split_at(delim_pos + 1);
        assert_eq!(key, "OUT_DIR=");

        p!("{} {}", key, out_dir);
        println!("cargo:rustc-link-search={}", out_dir);

        let count = dep.0.split("_").count();
        let name = dep
            .0
            .split("_")
            .skip(2)
            .take(count - 3)
            .collect::<Vec<&str>>()
            .join("_")
            .to_lowercase();
        p!("{}", name);

        for entry in read_dir(out_dir).unwrap() {
            let entry = entry.unwrap();
            let file_path = entry.path();
            p!("Link {}", file_path.display());
            println!("cargo:rustc-link-arg=-T{}", file_path.display());
        }
    }

    // Add final linker scripts
    println!("cargo:rustc-link-arg={}", "-Tlink.x");

    // Create the binary map
    println!(
        "cargo:rustc-link-arg=-Map={}/bin-{}.map",
        crate_dir, features[0]
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
