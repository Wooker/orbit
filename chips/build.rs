use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn memory_from_feature(feature: &String) -> Vec<u8> {
    let mut buf = Vec::new();
    let memory_path =
        PathBuf::from_str(format!("src/{}/memory.x", feature.as_str()).as_str()).unwrap();
    let mut memory = File::open(memory_path).unwrap();
    memory.read_to_end(&mut buf).unwrap();
    buf
}

fn main() {
    // for var in env::vars()
    //     .into_iter()
    //     .filter(|(key, _)| key.starts_with("CARGO"))
    // {
    //     p!("{} {}", var.0, var.1);
    // }
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

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memory_from_feature(chip).as_slice())
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=src/{}/memory.x", chip);
    println!("cargo:rerun-if-changed=build.rs");
}
