#![allow(unused)]

use std::env;

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
    match chip.as_str() {
        "ch592" => {
            println!("cargo:rustc-link-arg={}", "-Tlink.x");
        }
        "ch32v208wbu6" => {
            println!("cargo:rustc-link-arg={}", "-Tlink.x");
        }
        "esp32c3" => {
            println!("cargo:rustc-link-arg={}", "-Tmemory.x");
            println!("cargo:rustc-link-arg={}", "-Tlink.x");
        }
        _ => {}
    }
    println!(
        "cargo:rustc-link-arg={}{}/{}",
        "-Map=", crate_dir, "kernel.map"
    );
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
