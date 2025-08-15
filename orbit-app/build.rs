use std::{env, path::PathBuf};

#[allow(unused)]
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[34;1m   {}: \x1b[0m{}", "orbit-app", format!($($tokens)*))
    }
}

fn main() {
    // Get crate dir
    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();

    // Get OUT_DIR and save the main linker script there
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // File::create(out.join("app.x"))
    //     .unwrap()
    //     .write_all(include_bytes!("app.x"))
    //     .expect("Could not find app.x");

    // Add OUT_DIR to link search
    println!("cargo:rustc-link-search={}", out.display());

    // Metadata for dependent crates
    println!("cargo::metadata=OUT_DIR={}", out.display());

    // Recompile if this file changes
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}
