use std::{env, fs::File, io::Write, path::PathBuf};

#[allow(unused)]
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[34;1m   {}: \x1b[0m{}", "orbit-kernel", format!($($tokens)*))
    }
}

#[allow(unused)]
fn print_env() {
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("CARGO"))
        .for_each(|f| p!("{} {}", f.0, f.1));
}

fn main() {
    // print_env();

    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("kernel.x"))
        .unwrap()
        .write_all(include_bytes!("kernel.x"))
        .expect("Could not find kernel.x");

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
