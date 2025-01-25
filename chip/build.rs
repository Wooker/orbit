use std::env;
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[allow(unused)]
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
    // p!("Features: {:?}", features);
    if features.len() != 1 {
        panic!("Use only one feature for the chip.");
    }
    let chip = features.last().unwrap();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // if chip == "ch32v208wbu6" {
    //     File::create(out.join("linkall.x"))
    //         .unwrap()
    //         .write_all(link_script_from_feature(chip, "linkall.x").as_slice())
    //         .unwrap();
    // }
    if chip == "esp32c3" {
        File::create(out.join("linkall.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/linkall.x").as_slice())
            .unwrap();
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/memory.x").as_slice())
            .unwrap();
        File::create(out.join("esp32c3.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/esp32c3.x").as_slice())
            .unwrap();
        File::create(out.join("rom-functions.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom-functions.x").as_slice())
            .unwrap();
        File::create(out.join("rwtext.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/rwtext.x").as_slice())
            .unwrap();
        File::create(out.join("text.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/text.x").as_slice())
            .unwrap();
        File::create(out.join("rwdata.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/rwdata.x").as_slice())
            .unwrap();
        File::create(out.join("rodata.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/rodata.x").as_slice())
            .unwrap();
        File::create(out.join("stack.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/stack.x").as_slice())
            .unwrap();
        File::create(out.join("rtc_fast.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/rtc_fast.x").as_slice())
            .unwrap();
        File::create(out.join("rtc_slow.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/rtc_slow.x").as_slice())
            .unwrap();
        File::create(out.join("dram2.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/sections/dram2.x").as_slice())
            .unwrap();
        File::create(out.join("debug.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/riscv/debug.x").as_slice())
            .unwrap();
        File::create(out.join("hal-defaults.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/riscv/hal-defaults.x").as_slice())
            .unwrap();
        File::create(out.join("additional.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/additional.ld").as_slice())
            .unwrap();
        DirBuilder::new()
            .recursive(true)
            .create(out.join("rom"))
            .unwrap();
        File::create(out.join("rom/esp32c3.rom.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/esp32c3.rom.ld").as_slice())
            .unwrap();
        File::create(out.join("rom/additional.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/additional.ld").as_slice())
            .unwrap();
        File::create(out.join("rom/esp32c3.rom.api.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/esp32c3.rom.api.ld").as_slice())
            .unwrap();
        File::create(out.join("rom/esp32c3.rom.eco3.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/esp32c3.rom.eco3.ld").as_slice())
            .unwrap();
        File::create(out.join("rom/esp32c3.rom.eco7.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/esp32c3.rom.eco7.ld").as_slice())
            .unwrap();
        File::create(out.join("rom/esp32c3.rom.libgcc.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/esp32c3.rom.libgcc.ld").as_slice())
            .unwrap();
        File::create(out.join("rom/esp32c3.rom.version.ld"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "ld/rom/esp32c3.rom.version.ld").as_slice())
            .unwrap();
        println!("cargo:rerun-if-changed=src/{}/linkall.x", chip);
    } else {
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(link_script_from_feature(chip, "memory.x").as_slice())
            .unwrap();
    }
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=src/{}/memory.x", chip);
    println!("cargo:rerun-if-changed=build.rs");
}
