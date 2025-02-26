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
    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("app-link.x"))
        .unwrap()
        .write_all(include_bytes!("app-link.x"))
        .expect("Could not find app-link.x");

    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rustc-link-arg={}", "-Tapp-link.x");
    // println!("cargo:rustc-link-arg={}", "-Tlink.x");

    println!("cargo:rustc-link-arg={}", "-Map=app.map");

    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
