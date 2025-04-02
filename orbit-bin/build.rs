#![allow(unused)]

use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
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
    p!("Features: {:?}", features);
    if features.len() != 1 {
        panic!("Use only one feature for the chip.");
    }

    // Get OUT_DIR and save the main linker script there
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .expect("Could not find link.x");

    // Get OUT_DIR of orbit-app crate
    let app_var = "DEP_ORBIT_APP_OUT_DIR";
    let app_out = &PathBuf::from(
        env::var_os(app_var).expect(format!("Cannot find the {} env var", app_var).as_str()),
    );

    // Add linker scripts of memory and kernel
    println!("cargo:rustc-link-arg={}", "-Tmemory.x");
    println!("cargo:rustc-link-arg={}", "-Tkernel.x");

    // Add linker scripts of applications
    for entry in fs::read_dir(app_out).unwrap() {
        let e = entry.unwrap();
        let file_name = e.file_name().into_string().unwrap();
        if file_name.starts_with("app") && file_name.ends_with("link.x") {
            p!("Including linker script: {}", file_name);
            println!("cargo:rustc-link-arg=-T{}", e.file_name().to_str().unwrap());
        }
    }

    // Add final linker scripts
    println!("cargo:rustc-link-arg={}", "-Tapp.x");
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
}
