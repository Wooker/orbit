#![allow(unused)]

use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

#[allow(unused)]
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn print_env() {
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("CARGO"))
        .for_each(|f| p!("{} {}", f.0, f.1));
}

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
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
    // print_env();

    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();
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
    let chip = features.last().unwrap();

    println!("cargo:rustc-link-arg={}", "--verbose");
    println!("cargo:rustc-link-arg={}", "--error-limit=0");

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("kernel.x"))
        .unwrap()
        .write_all(include_bytes!("kernel.x"))
        .expect("Could not find kernel.x");

    match chip.as_str() {
        "ch592" => {
            println!("cargo:rustc-link-arg={}", "-Tlink.x");
            println!("cargo:rustc-link-arg={}", "-Tkernel.x");
        }
        "ch32v208wbu6" => {
            println!("cargo:rustc-link-arg={}", "-Tlink.x");
            println!("cargo:rustc-link-arg={}", "-Tkernel.x");
        }
        "esp32c3" => {
            // println!("cargo:rustc-link-arg={}", "-Tmemory.x");
            // println!("cargo:rustc-link-arg={}", "-Triscv.x");
            // println!("cargo:rustc-link-arg={}", "-Torbit-kernel/kernel.x");
            println!("cargo:rustc-link-arg={}", "-Tlinkall.x");
        }
        _ => {}
    }
    println!(
        "cargo:rustc-link-arg={}{}/{}",
        "-Map=", crate_dir, "kernel.map"
    );
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
