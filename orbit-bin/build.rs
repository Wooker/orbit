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

    // Add linker scripts of memory and kernel
    println!("cargo:rustc-link-arg={}", "-Tmemory.x");
    println!("cargo:rustc-link-arg={}", "-Tkernel.x");

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
