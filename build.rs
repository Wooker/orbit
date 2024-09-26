use std::env;
use std::fs;
use std::path::Path;

use orbit_build::assert_used_features;

fn main() {
    assert_used_features!("esp32c3", "stm32f103");
}
