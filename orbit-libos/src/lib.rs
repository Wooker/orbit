#![no_std]
#![no_main]

use orbit_common::feature_mod;

feature_mod!("ch592", pub, uart);
feature_mod!("ch32x035", pub, spi);
feature_mod!("ch32x035", pub, eink);
feature_mod!("ch32x035", pub, font_8x8);
feature_mod!("ch32x035", pub, font_8x10);
// feature_mod!("ch32v208wbu6", pub, uart_v208);

// #[cfg(any(feature = "ch32v208wbu6", feature = "ch32v003"))]
// pub mod uart_v208;
