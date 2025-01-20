use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("riscv.x"))
        .unwrap()
        .write_all(include_bytes!("riscv-link.x").as_slice())
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
}
