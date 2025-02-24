#![no_std]
#![no_main]
#![feature(strict_overflow_ops)]

use orbit_common::feature_mod;

feature_mod!("ch592", pub, uart);
// feature_mod!("ch32v208wbu6", pub, uart_v208);
#[cfg(feature = "ch32v208wbu6")]
pub mod uart_v208;
