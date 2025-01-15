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

fn link_script_from_feature(feature: &String, script_name: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    let script_path =
        PathBuf::from_str(format!("src/{}/{}", feature.as_str(), script_name).as_str()).unwrap();
    let mut memory = File::open(script_path).unwrap();
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
        .write_all(link_script_from_feature(chip, "memory.x").as_slice())
        .unwrap();
    if chip == "esp32c3" {
        File::create(out.join("link.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "link.x").as_slice())
            .unwrap();
    }
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=src/{}/memory.x", chip);
    println!("cargo:rerun-if-changed=build.rs");
}
