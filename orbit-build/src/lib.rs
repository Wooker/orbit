use std::{
    env,
    fs::{copy, read_dir, write},
    path::{Path, PathBuf},
};

#[macro_export]
macro_rules! print_build {
    ($($tokens: tt)*) => {
        if let Ok(pkg_name) = std::env::var("CARGO_PKG_NAME") {
            println!("cargo:warning=\r\x1b[0K\x1b[34;1m   {}: \x1b[0m{}", pkg_name, format!($($tokens)*))
        } else {
            println!("cargo:warning=\r\x1b[0K\x1b[34;1m   {}: \x1b[0m{}", env!("CARGO_PKG_NAME"), format!($($tokens)*))
        }
    }
}

#[allow(unused)]
fn print_env() {
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("CARGO"))
        .for_each(|f| print_build!("{} {}", f.0, f.1));
    env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("DEP"))
        .for_each(|f| print_build!("{} {}", f.0, f.1));
}

pub fn build_main() {
    // Set verbosity
    println!("cargo:rustc-link-arg={}", "--verbose");
    println!("cargo:rustc-link-arg={}", "--error-limit=0");

    // Get crate dir
    let crate_dir = env::var("CARGO_PKG_NAME").unwrap();

    // Get OUT_DIR and save the main linker script there
    let _out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Read bin main contents
    let file = "src/main.rs";
    let source_path = Path::new(&file);
    let _contents = std::fs::read_to_string(source_path).expect("Could not read source file");

    // Add linker scripts of memory and kernel
    println!("cargo:rustc-link-arg={}", "-Tmemory.x");
    println!("cargo:rustc-link-arg={}", "-Tkernel.x");

    for dep in env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("DEP") && !key.contains("COMPILER"))
    {
        #[cfg(debug_assertions)]
        print_build!("");
        #[cfg(debug_assertions)]
        print_build!("{}", dep.0);
        // let delim_pos =
        let (key, out_dir) = if let Some(delim_pos) = dep.1.find('=') {
            dep.1.split_at(delim_pos + 1)
            // } else {
            //     print_build!("Incorrect metadata format"));
        } else {
            continue;
        };
        assert_eq!(key, "OUT_DIR=");

        #[cfg(debug_assertions)]
        print_build!("{} {}", key, out_dir);
        println!("cargo:rustc-link-search={}", out_dir);

        #[cfg(debug_assertions)]
        let count = dep.0.split("_").count();
        #[cfg(debug_assertions)]
        let name = dep
            .0
            .split("_")
            .skip(2)
            .take(count - 3)
            .collect::<Vec<&str>>()
            .join("_")
            .to_lowercase();
        #[cfg(debug_assertions)]
        print_build!("{}", name);

        for entry in read_dir(out_dir).unwrap() {
            let entry = entry.unwrap();
            let file_path = entry.path();
            #[cfg(debug_assertions)]
            print_build!("Link {}", file_path.display());
            println!("cargo:rustc-link-arg=-T{}", file_path.display());
        }
    }

    // Add final linker scripts
    println!("cargo:rustc-link-arg={}", "-Tlink.x");

    // Create the binary map
    println!("cargo:rustc-link-arg=-Map={}.map", &crate_dir);

    // Recompile if this file changes
    println!("cargo:rerun-if-changed={}/build.rs", crate_dir);
}

pub fn build_app() {
    let pkg = env::var("CARGO_PKG_NAME").unwrap();
    let pkg_type = env::var("CARGO_MANIFEST_LINKS")
        .unwrap()
        .split("_")
        .take(1)
        .collect::<String>();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let linker_script_path = Path::new(&out).join(format!("{}.x", &pkg));
    let linker_script_contents = format!(
        r"
    SECTIONS
    {{
       	.{pkg_type}.{pkg} : ALIGN(4)
       	{{
      		*.{pkg}.*.o(.text .text.*);
      		*.{pkg}.*.o(.rodata .rodata.*);

       	}} >FLASH
    }}
    "
    );

    write(linker_script_path, linker_script_contents)
        .expect("Could not write linker script to OUT_DIR");

    for dep in env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("DEP") && !key.contains("COMPILER"))
    {
        #[cfg(debug_assertions)]
        print_build!("{}", dep.0);
        let component_type = dep.0.split("_").skip(1).take(1).collect::<String>();
        assert_eq!(component_type, "LIBOS");

        let delim_pos = dep.1.find('=').expect("Incorrect metadata format");
        let (key, out_dir) = dep.1.split_at(delim_pos + 1);
        assert_eq!(key, "OUT_DIR=");

        let count = dep.0.split("_").count();
        let name = dep
            .0
            .split("_")
            .skip(2)
            .take(count - 3)
            .collect::<Vec<&str>>()
            .join("_")
            .to_lowercase();

        // println!("cargo:rustc-link-search={}", out_dir);
        // println!("cargo:rustc-link-arg=-T{}.x", name);

        let libos_path = Path::new(&out_dir).join(format!("{}.x", name));
        copy(libos_path, out.join(format!("{}.x", name))).expect("Could not copy linker file");
    }

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}/.cargo/build.rs", pkg);

    println!("cargo:metadata=OUT_DIR={}", out.display());
}
