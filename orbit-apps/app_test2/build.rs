use std::{env, fs::File, io::Read, io::Write, path::PathBuf};

fn main() {
    let pkg = env::var("CARGO_PKG_NAME").unwrap();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut linker_script_file = File::open(format!("{}.x", pkg))
        .expect(format!("Could not open linker script of {}", pkg).as_str());
    let mut linker_script_contents = String::new();
    let _count = linker_script_file.read_to_string(&mut linker_script_contents);
    File::create(out.join(format!("{}.x", &pkg)))
        .unwrap()
        .write_all(linker_script_contents.as_bytes())
        .expect(format!("Could not find {}.x", pkg).as_str());

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}/build.rs", pkg);

    println!("cargo:metadata=OUT_DIR={}", out.display());

    println!("cargo:rustc-link-arg={}", "-Ttest_app.x");
}
