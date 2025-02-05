#![no_std]
#![no_main]
#![feature(strict_overflow_ops)]

use orbit_common::feature_mod;

feature_mod!("ch592", pub, uart);
