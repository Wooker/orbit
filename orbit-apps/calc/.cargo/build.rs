use std::{
    env,
    fs::{copy, write},
    path::{Path, PathBuf},
};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[34;1m   {}: \x1b[0m{}", "orbit-component", format!($($tokens)*))
    }
}

fn main() {
    let pkg = env::var("CARGO_PKG_NAME").unwrap();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let linker_script_path = Path::new(&out).join("calc.x");
    let linker_script_contents = r"
SECTIONS
{
	.app.calc : ALIGN(4)
	{
		*calc*.o(.text .text.*);
	} >FLASH
}
        ";

    write(linker_script_path, linker_script_contents)
        .expect("Could not write linker script to OUT_DIR");

    for dep in env::vars()
        .into_iter()
        .filter(|(key, _)| key.starts_with("DEP") && !key.contains("COMPILER"))
    {
        p!("{}", dep.0);
        let component_type = dep.0.split("_").skip(1).take(1).collect::<String>();
        assert_eq!(component_type, "LIBOS");

        let delim_pos = dep.1.find('=').expect("Incorrect metadata format");
        let (key, out_dir) = dep.1.split_at(delim_pos + 1);
        assert_eq!(key, "OUT_DIR=");

        p!("{} {}", key, out_dir);

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

        let libos_path = Path::new(&out_dir).join(format!("{}.x", name));
        copy(libos_path, out.join(format!("{}.x", name))).expect("Could not copy linker file");
    }

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={}/.cargo/build.rs", pkg);

    println!("cargo:metadata=OUT_DIR={}", out.display());
}
