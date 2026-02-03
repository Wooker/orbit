#![no_std]
#![allow(non_upper_case_globals)]

#[inline(never)]
pub fn version() -> usize {
    1
}

pub type Letter = [u8; 8];

macro_rules! rotate_90_clockwise {
    ($src:expr) => {{
        const fn reverse_bits(mut b: u8) -> u8 {
            let mut r = 0u8;
            let mut i = 0;
            while i < 8 {
                r <<= 1;
                r |= b & 1;
                b >>= 1;
                i += 1;
            }
            r
        }

        const fn rotate(src: [u8; 8]) -> [u8; 8] {
            let mut dst = [0u8; 8];
            let mut y = 0;
            while y < 8 {
                let mut x = 0;
                while x < 8 {
                    if (src[y] >> (7 - x)) & 1 == 1 {
                        dst[x] |= 1 << y;
                    }
                    x += 1;
                }
                y += 1;
            }
            // flip horizontally (reverse bits in each row)
            let mut i = 0;
            while i < 8 {
                dst[i] = reverse_bits(dst[i]);
                i += 1;
            }
            dst
        }

        rotate($src)
    }};
}

macro_rules! rotate_font {
    ($src:ident, $dst:ident) => {
        pub const $dst: [Letter; $src.len()] = {
            let mut out: [Letter; $src.len()] = [[0; 8]; $src.len()];
            let mut i = 0;
            while i < $src.len() {
                out[i] = rotate_90_clockwise!($src[i]);
                i += 1;
            }
            out
        };
    };
}

pub const ASCII: [Letter; 256] = [
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, // 0x20
    space, CHAR_21, CHAR_22, CHAR_23, CHAR_24, CHAR_25, CHAR_26, CHAR_27, CHAR_28, CHAR_29,
    CHAR_2A, CHAR_2B, CHAR_2C, CHAR_2D, CHAR_2E, CHAR_2F, CHAR_30, CHAR_31, CHAR_32, CHAR_33,
    CHAR_34, CHAR_35, CHAR_36, CHAR_37, CHAR_38, CHAR_39, space, space, space, space, space, space,
    space, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, space,
    space, space, space, space, space, A_LOWER, B_LOWER, C_LOWER, D_LOWER, E_LOWER, F_LOWER,
    G_LOWER, H_LOWER, I_LOWER, J_LOWER, K_LOWER, L_LOWER, M_LOWER, N_LOWER, O_LOWER, P_LOWER,
    Q_LOWER, R_LOWER, S_LOWER, T_LOWER, U_LOWER, V_LOWER, W_LOWER, X_LOWER, Y_LOWER, Z_LOWER,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space, space, space, space, space, space, space, space, space, space, space,
    space, space, space,
];
rotate_font!(ASCII, ASCII_ROTATED);

pub const space: Letter = [
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0],
];

pub const CHAR_21: Letter = [
    // '!'
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x3C, // [0, 0, 1, 1, 1, 1, 0, 0]
    0x3C, // [0, 0, 1, 1, 1, 1, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_22: Letter = [
    // '"'
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x24, // [0, 0, 1, 0, 0, 1, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_23: Letter = [
    // '#'
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x7F, // [0, 1, 1, 1, 1, 1, 1, 1]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x7F, // [0, 1, 1, 1, 1, 1, 1, 1]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_24: Letter = [
    // '$'
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x3E, // [0, 0, 1, 1, 1, 1, 1, 0]
    0x03, // [0, 0, 0, 0, 0, 0, 1, 1]
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x1F, // [0, 0, 0, 1, 1, 1, 1, 1]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_25: Letter = [
    // '%'
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x63, // [0, 1, 1, 0, 0, 0, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x66, // [0, 1, 1, 0, 0, 1, 1, 0]
    0x63, // [0, 1, 1, 0, 0, 0, 1, 1]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_26: Letter = [
    // '&'
    0x1C, // [0, 0, 0, 1, 1, 1, 0, 0]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x1C, // [0, 0, 0, 1, 1, 1, 0, 0]
    0x6E, // [0, 1, 1, 0, 1, 1, 1, 0]
    0x3B, // [0, 0, 1, 1, 1, 0, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x6E, // [0, 1, 1, 0, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_27: Letter = [
    // '''
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x03, // [0, 0, 0, 0, 0, 0, 1, 1]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_28: Letter = [
    // '('
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_29: Letter = [
    // ')'
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_2A: Letter = [
    // '*'
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x66, // [0, 1, 1, 0, 0, 1, 1, 0]
    0x3C, // [0, 0, 1, 1, 1, 1, 0, 0]
    0xFF, // [1, 1, 1, 1, 1, 1, 1, 1]
    0x3C, // [0, 0, 1, 1, 1, 1, 0, 0]
    0x66, // [0, 1, 1, 0, 0, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_2B: Letter = [
    // '+'
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x3F, // [0, 0, 1, 1, 1, 1, 1, 1]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_2C: Letter = [
    // ','
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_2D: Letter = [
    // '-'
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x3F, // [0, 0, 1, 1, 1, 1, 1, 1]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_2E: Letter = [
    // '.'
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_2F: Letter = [
    // '/'
    0x60, // [0, 1, 1, 0, 0, 0, 0, 0]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x03, // [0, 0, 0, 0, 0, 0, 1, 1]
    0x01, // [0, 0, 0, 0, 0, 0, 0, 1]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_30: Letter = [
    // '0'
    0x3E, // [0, 0, 1, 1, 1, 1, 1, 0]
    0x63, // [0, 1, 1, 0, 0, 0, 1, 1]
    0x73, // [0, 1, 1, 1, 0, 0, 1, 1]
    0x7B, // [0, 1, 1, 1, 1, 0, 1, 1]
    0x6F, // [0, 1, 1, 0, 1, 1, 1, 1]
    0x67, // [0, 1, 1, 0, 0, 1, 1, 1]
    0x3E, // [0, 0, 1, 1, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_31: Letter = [
    // '1'
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0E, // [0, 0, 0, 0, 1, 1, 1, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x3F, // [0, 0, 1, 1, 1, 1, 1, 1]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_32: Letter = [
    // '2'
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x1C, // [0, 0, 0, 1, 1, 1, 0, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x3F, // [0, 0, 1, 1, 1, 1, 1, 1]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_33: Letter = [
    // '3'
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x1C, // [0, 0, 0, 1, 1, 1, 0, 0]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_34: Letter = [
    // '4'
    0x38, // [0, 0, 1, 1, 1, 0, 0, 0]
    0x3C, // [0, 0, 1, 1, 1, 1, 0, 0]
    0x36, // [0, 0, 1, 1, 0, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x7F, // [0, 1, 1, 1, 1, 1, 1, 1]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x78, // [0, 1, 1, 1, 1, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_35: Letter = [
    // '5'
    0x3F, // [0, 0, 1, 1, 1, 1, 1, 1]
    0x03, // [0, 0, 0, 0, 0, 0, 1, 1]
    0x1F, // [0, 0, 0, 1, 1, 1, 1, 1]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_36: Letter = [
    // '6'
    0x1C, // [0, 0, 0, 1, 1, 1, 0, 0]
    0x06, // [0, 0, 0, 0, 0, 1, 1, 0]
    0x03, // [0, 0, 0, 0, 0, 0, 1, 1]
    0x1F, // [0, 0, 0, 1, 1, 1, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_37: Letter = [
    // '7'
    0x3F, // [0, 0, 1, 1, 1, 1, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x0C, // [0, 0, 0, 0, 1, 1, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_38: Letter = [
    // '8'
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const CHAR_39: Letter = [
    // '9'
    0x1E, // [0, 0, 0, 1, 1, 1, 1, 0]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x33, // [0, 0, 1, 1, 0, 0, 1, 1]
    0x3E, // [0, 0, 1, 1, 1, 1, 1, 0]
    0x30, // [0, 0, 1, 1, 0, 0, 0, 0]
    0x18, // [0, 0, 0, 1, 1, 0, 0, 0]
    0x0E, // [0, 0, 0, 0, 1, 1, 1, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
];

pub const A: Letter = [
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0xF0, // [1, 1, 1, 1, 0, 0, 0, 0]
    0x70, // [0, 1, 1, 1, 0, 0, 0, 0]
    0x50, // [0, 1, 0, 1, 0, 0, 0, 0]
    0xF8, // [1, 1, 1, 1, 1, 0, 0, 0]
    0xD8, // [1, 1, 0, 1, 1, 0, 0, 0]
    0xDC, // [1, 1, 0, 1, 1, 1, 0, 0]
];

pub const B: Letter = [
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0xF0, // [1, 1, 1, 1, 0, 0, 0, 0]
    0xD8, // [1, 1, 0, 1, 1, 0, 0, 0]
    0xF0, // [1, 1, 1, 1, 0, 0, 0, 0]
    0xD8, // [1, 1, 0, 1, 1, 0, 0, 0]
    0xD8, // [1, 1, 0, 1, 1, 0, 0, 0]
    0xF0, // [1, 1, 1, 1, 0, 0, 0, 0]
];

pub const C: Letter = [
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x00, // [0, 0, 0, 0, 0, 0, 0, 0]
    0x78, // [0, 1, 1, 1, 1, 0, 0, 0]
    0xD8, // [1, 1, 0, 1, 1, 0, 0, 0]
    0xC0, // [1, 1, 0, 0, 0, 0, 0, 0]
    0xC0, // [1, 1, 0, 0, 0, 0, 0, 0]
    0xD8, // [1, 1, 0, 1, 1, 0, 0, 0]
    0x70, // [0, 1, 1, 1, 0, 0, 0, 0]
];

pub const D: Letter = [0x00, 0x02, 0x02, 0x3e, 0x42, 0x42, 0x42, 0x3e];
pub const E: Letter = [0x00, 0x00, 0x3e, 0x40, 0x7c, 0x40, 0x40, 0x3e];
pub const F: Letter = [0x00, 0x00, 0x3e, 0x40, 0x7c, 0x40, 0x40, 0x40];
pub const G: Letter = [0x00, 0x00, 0x3c, 0x40, 0x40, 0x4e, 0x42, 0x3c];
pub const H: Letter = [0x00, 0x00, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42];
pub const I: Letter = [0x00, 0x00, 0x3c, 0x08, 0x08, 0x08, 0x08, 0x3c];
pub const J: Letter = [0x00, 0x00, 0x04, 0x04, 0x04, 0x44, 0x44, 0x38];
pub const K: Letter = [0x00, 0x00, 0x42, 0x44, 0x78, 0x44, 0x42, 0x42];
pub const L: Letter = [0x00, 0x00, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7e];
pub const M: Letter = [0x00, 0x00, 0x42, 0x66, 0x5a, 0x42, 0x42, 0x42];
pub const N: Letter = [0x00, 0x00, 0x42, 0x62, 0x52, 0x4a, 0x46, 0x42];
pub const O: Letter = [0x00, 0x00, 0x3c, 0x42, 0x42, 0x42, 0x42, 0x3c];
pub const P: Letter = [0x00, 0x00, 0x7c, 0x42, 0x42, 0x7c, 0x40, 0x40];
pub const Q: Letter = [0x00, 0x00, 0x3c, 0x42, 0x42, 0x4a, 0x44, 0x3a];
pub const R: Letter = [0x00, 0x00, 0x7c, 0x42, 0x42, 0x7c, 0x44, 0x42];
pub const S: Letter = [0x00, 0x00, 0x3e, 0x40, 0x3c, 0x02, 0x02, 0x7c];
pub const T: Letter = [0x00, 0x00, 0x7f, 0x08, 0x08, 0x08, 0x08, 0x08];
pub const U: Letter = [0x00, 0x00, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3e];
pub const V: Letter = [0x00, 0x00, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18];
pub const W: Letter = [0x00, 0x00, 0x42, 0x42, 0x5a, 0x5a, 0x66, 0x42];
pub const X: Letter = [0x00, 0x00, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42];
pub const Y: Letter = [0x00, 0x00, 0x42, 0x24, 0x18, 0x18, 0x08, 0x08];
pub const Z: Letter = [0x00, 0x00, 0x7e, 0x04, 0x08, 0x10, 0x20, 0x7e];

pub const A_LOWER: Letter = [
    0x00, // [0,0,0,0,0,0,0,0]
    0x00, // [0,0,0,0,0,0,0,0]
    0x3c, // [0,1,1,1,1,1,0,0]
    0x02, // [0,0,0,0,0,0,1,0]
    0x3e, // [0,0,1,1,1,1,1,0]
    0x42, // [0,1,0,0,0,0,1,0]
    0x42, // [0,1,0,0,0,0,1,0]
    0x3e, // [0,0,1,1,1,1,1,0]
];

pub const B_LOWER: Letter = [
    0x00, // [0,0,0,0,0,0,0,0]
    0x40, // [0,1,0,0,0,0,0,0]
    0x40, // [0,1,0,0,0,0,0,0]
    0x7c, // [0,1,1,1,1,1,0,0]
    0x42, // [0,1,0,0,0,0,1,0]
    0x42, // [0,1,0,0,0,0,1,0]
    0x42, // [0,1,0,0,0,0,1,0]
    0x7c, // [0,1,1,1,1,1,0,0]
];

pub const C_LOWER: Letter = [
    0x00, // [0,0,0,0,0,0,0,0]
    0x00, // [0,0,0,0,0,0,0,0]
    0x18, // [0,0,0,1,1,0,0,0]
    0x24, // [0,0,1,0,0,1,0,0]
    0x40, // [0,1,0,0,0,0,0,0]
    0x40, // [0,1,0,0,0,0,0,0]
    0x40, // [0,1,0,0,0,0,0,0]
    0x3e, // [0,0,1,1,1,1,1,0]
];

pub const D_LOWER: Letter = [0x00, 0x02, 0x02, 0x3e, 0x42, 0x42, 0x42, 0x3e];
pub const E_LOWER: Letter = [0x00, 0x00, 0x3c, 0x42, 0x7e, 0x40, 0x42, 0x3c];
pub const F_LOWER: Letter = [0x00, 0x00, 0x1e, 0x20, 0x3c, 0x20, 0x20, 0x20];
pub const G_LOWER: Letter = [0x00, 0x00, 0x3e, 0x40, 0x40, 0x4e, 0x42, 0x3c];
pub const H_LOWER: Letter = [0x00, 0x40, 0x40, 0x7c, 0x42, 0x42, 0x42, 0x42];
pub const I_LOWER: Letter = [0x00, 0x10, 0x00, 0x30, 0x10, 0x10, 0x10, 0x38];
pub const J_LOWER: Letter = [0x00, 0x04, 0x00, 0x0c, 0x04, 0x44, 0x44, 0x38];
pub const K_LOWER: Letter = [0x00, 0x40, 0x48, 0x50, 0x60, 0x50, 0x48, 0x44];
pub const L_LOWER: Letter = [0x00, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x1c];
pub const M_LOWER: Letter = [0x00, 0x00, 0x6c, 0x52, 0x52, 0x52, 0x52, 0x52];
pub const N_LOWER: Letter = [0x00, 0x00, 0x5c, 0x62, 0x42, 0x42, 0x42, 0x42];
pub const O_LOWER: Letter = [0x00, 0x00, 0x3c, 0x42, 0x42, 0x42, 0x42, 0x3c];
pub const P_LOWER: Letter = [0x00, 0x00, 0x7c, 0x42, 0x7c, 0x40, 0x40, 0x40];
pub const Q_LOWER: Letter = [0x00, 0x00, 0x3c, 0x42, 0x42, 0x4a, 0x44, 0x3a];
pub const R_LOWER: Letter = [0x00, 0x00, 0x5c, 0x62, 0x40, 0x40, 0x40, 0x40];
pub const S_LOWER: Letter = [0x00, 0x00, 0x3e, 0x40, 0x3c, 0x02, 0x02, 0x7c];
pub const T_LOWER: Letter = [0x00, 0x10, 0x10, 0x7c, 0x10, 0x10, 0x10, 0x0c];
pub const U_LOWER: Letter = [0x00, 0x00, 0x42, 0x42, 0x42, 0x42, 0x46, 0x3a];
pub const V_LOWER: Letter = [0x00, 0x00, 0x42, 0x42, 0x24, 0x24, 0x18, 0x18];
pub const W_LOWER: Letter = [0x00, 0x00, 0x42, 0x42, 0x52, 0x52, 0x52, 0x2c];
pub const X_LOWER: Letter = [0x00, 0x00, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42];
pub const Y_LOWER: Letter = [0x00, 0x00, 0x42, 0x24, 0x18, 0x18, 0x10, 0x10];
pub const Z_LOWER: Letter = [0x00, 0x00, 0x7e, 0x04, 0x08, 0x10, 0x20, 0x7e];
