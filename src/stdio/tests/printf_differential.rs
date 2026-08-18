//! Differential tests for the integer `printf` formatting adapters.

use std::ffi::CStr;

use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

use super::{signed, unsigned};

const BUFFER_SIZE: usize = 256;

fn check_result(
    c_format: &'static [u8],
    rust_output: String,
    type_name: &str,
    value: impl std::fmt::Display,
    written: libc::c_int,
    buffer: &[libc::c_char; BUFFER_SIZE],
) {
    let format = CStr::from_bytes_with_nul(c_format)
        .expect("test format must have exactly one trailing NUL")
        .to_string_lossy();
    assert!(
        written >= 0,
        "snprintf failed: format={format:?}, type={type_name}, value={value}",
    );
    let written = written as usize;
    assert!(
        written < buffer.len(),
        "snprintf truncated output: format={format:?}, type={type_name}, value={value}, required={written}, capacity={}",
        buffer.len(),
    );

    // `snprintf` initialized exactly `written` output bytes before its trailing
    // NUL. Viewing that initialized prefix as bytes is valid for every possible
    // `c_char` signedness.
    let c_output = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), written) };
    assert_eq!(
        c_output,
        rust_output.as_bytes(),
        "format mismatch: format={format:?}, type={type_name}, value={value}, libc={:?}, rust={:?}",
        String::from_utf8_lossy(c_output),
        rust_output,
    );
}

macro_rules! define_oracle {
    ($name:ident, $abi_type:ty) => {
        fn $name(
            c_format: &'static [u8],
            value: $abi_type,
            rust_output: String,
            type_name: &str,
            source_value: impl std::fmt::Display,
        ) {
            let mut buffer = [0 as libc::c_char; BUFFER_SIZE];
            // SAFETY: every caller supplies a static, NUL-terminated format
            // containing exactly one integer conversion whose required
            // variadic type is this function's concrete `$abi_type`. The
            // destination has `BUFFER_SIZE` writable elements. `check_result`
            // rejects errors and truncation before inspecting initialized data.
            let written = unsafe {
                libc::snprintf(
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    c_format.as_ptr().cast(),
                    value,
                )
            };
            check_result(
                c_format,
                rust_output,
                type_name,
                source_value,
                written,
                &buffer,
            );
        }
    };
}

define_oracle!(oracle_c_int, libc::c_int);
define_oracle!(oracle_c_uint, libc::c_uint);
define_oracle!(oracle_c_longlong, libc::c_longlong);
define_oracle!(oracle_c_ulonglong, libc::c_ulonglong);
define_oracle!(oracle_ssize_t, libc::ssize_t);
define_oracle!(oracle_size_t, libc::size_t);

macro_rules! pair {
    ($oracle:ident, $c_format:expr, $abi_value:expr, $rust_output:expr, $type_name:expr, $value:expr) => {
        $oracle($c_format, $abi_value, $rust_output, $type_name, $value)
    };
}

macro_rules! signed_matrix {
    ($value:expr, $type_name:literal, $length:literal, $oracle:ident, $abi_type:ty) => {{
        let value = $value;
        let abi_value = value as $abi_type;

        pair!(
            $oracle,
            concat!("%", $length, "d\0").as_bytes(),
            abi_value,
            format!("{}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%", $length, "i\0").as_bytes(),
            abi_value,
            format!("{}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%+", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:+}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("% ", $length, "d\0").as_bytes(),
            abi_value,
            format!("{}", signed(value).space_sign()),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%+ ", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:+}", signed(value).space_sign()),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%8", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:8}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%-8", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:<8}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%08", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:08}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%-08", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:<08}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%+08", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:+08}", signed(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%.", "0", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:.0}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.", "1", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:.1}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.", "5", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:.5}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%08.5", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:08.5}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%-08.5", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:<08.5}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%+08.5", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:+08.5}", signed(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("% 08.5", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:08.5}", signed(value).space_sign()),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%130.129", $length, "d\0").as_bytes(),
            abi_value,
            format!("{:130.129}", signed(value)),
            $type_name,
            value
        );
    }};
}

macro_rules! unsigned_matrix {
    ($value:expr, $type_name:literal, $length:literal, $oracle:ident, $abi_type:ty) => {{
        let value = $value;
        let abi_value = value as $abi_type;

        pair!(
            $oracle,
            concat!("%", $length, "u\0").as_bytes(),
            abi_value,
            format!("{}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:X}", unsigned(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%8", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:8}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%-8", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:<8}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%08", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:08}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%-08", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:<08}", unsigned(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%.0", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:.0}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.1", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:.1}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.5", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:.5}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.0", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:.0o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.1", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:.1o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.5", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:.5o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.0", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:.0x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.1", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:.1x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.5", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:.5x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.0", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:.0X}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.1", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:.1X}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%.5", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:.5X}", unsigned(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%08.5", $length, "u\0").as_bytes(),
            abi_value,
            format!("{:08.5}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%08.5", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:08.5o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%08.5", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:08.5x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%08.5", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:08.5X}", unsigned(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%#", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:#o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:#x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:#X}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#.0", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:#.0o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#.0", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:#.0x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#.0", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:#.0X}", unsigned(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%#08", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:#08o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#08", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:#08x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#08", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:#08X}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#08.5", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:#08.5o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#08.5", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:#08.5x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#08.5", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:#08.5X}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#-08.5", $length, "o\0").as_bytes(),
            abi_value,
            format!("{:<#08.5o}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#-08.5", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:<#08.5x}", unsigned(value)),
            $type_name,
            value
        );
        pair!(
            $oracle,
            concat!("%#-08.5", $length, "X\0").as_bytes(),
            abi_value,
            format!("{:<#08.5X}", unsigned(value)),
            $type_name,
            value
        );

        pair!(
            $oracle,
            concat!("%#130.129", $length, "x\0").as_bytes(),
            abi_value,
            format!("{:#130.129x}", unsigned(value)),
            $type_name,
            value
        );
    }};
}

fn check_i8(value: i8) {
    signed_matrix!(value, "i8", "hh", oracle_c_int, libc::c_int);
}

fn check_i16(value: i16) {
    signed_matrix!(value, "i16", "h", oracle_c_int, libc::c_int);
}

fn check_i32(value: i32) {
    signed_matrix!(value, "i32", "", oracle_c_int, libc::c_int);
}

fn check_i64(value: i64) {
    signed_matrix!(value, "i64", "ll", oracle_c_longlong, libc::c_longlong);
}

fn check_isize(value: isize) {
    signed_matrix!(value, "isize", "z", oracle_ssize_t, libc::ssize_t);
}

fn check_u8(value: u8) {
    unsigned_matrix!(value, "u8", "hh", oracle_c_int, libc::c_int);
}

fn check_u16(value: u16) {
    unsigned_matrix!(value, "u16", "h", oracle_c_int, libc::c_int);
}

fn check_u32(value: u32) {
    unsigned_matrix!(value, "u32", "", oracle_c_uint, libc::c_uint);
}

fn check_u64(value: u64) {
    unsigned_matrix!(value, "u64", "ll", oracle_c_ulonglong, libc::c_ulonglong);
}

fn check_usize(value: usize) {
    unsigned_matrix!(value, "usize", "z", oracle_size_t, libc::size_t);
}

#[test]
fn all_i8_values_match_libc() {
    for value in i8::MIN..=i8::MAX {
        check_i8(value);
    }
}

#[test]
fn all_u8_values_match_libc() {
    for value in u8::MIN..=u8::MAX {
        check_u8(value);
    }
}

#[test]
fn deterministic_signed_boundaries_match_libc() {
    for value in [
        i16::MIN,
        -257,
        -256,
        -255,
        -101,
        -100,
        -99,
        -17,
        -16,
        -15,
        -9,
        -8,
        -7,
        -1,
        0,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        99,
        100,
        101,
        255,
        256,
        257,
        i16::MAX,
    ] {
        check_i16(value);
    }
    for value in [
        i32::MIN,
        -65_537,
        -65_536,
        -65_535,
        -257,
        -256,
        -255,
        -101,
        -100,
        -99,
        -17,
        -16,
        -15,
        -9,
        -8,
        -7,
        -1,
        0,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        99,
        100,
        101,
        255,
        256,
        257,
        65_535,
        65_536,
        65_537,
        i32::MAX,
    ] {
        check_i32(value);
    }
    for value in [
        i64::MIN,
        -4_294_967_297,
        -4_294_967_296,
        -4_294_967_295,
        -257,
        -256,
        -255,
        -101,
        -100,
        -99,
        -17,
        -16,
        -15,
        -9,
        -8,
        -7,
        -1,
        0,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        99,
        100,
        101,
        255,
        256,
        257,
        4_294_967_295,
        4_294_967_296,
        4_294_967_297,
        i64::MAX,
    ] {
        check_i64(value);
    }
    for value in [
        isize::MIN,
        -257,
        -256,
        -255,
        -101,
        -100,
        -99,
        -17,
        -16,
        -15,
        -9,
        -8,
        -7,
        -1,
        0,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        99,
        100,
        101,
        255,
        256,
        257,
        isize::MAX,
    ] {
        check_isize(value);
    }
}

#[test]
fn deterministic_unsigned_boundaries_match_libc() {
    for value in [
        0_u16,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        63,
        64,
        65,
        99,
        100,
        101,
        255,
        256,
        257,
        4_095,
        4_096,
        4_097,
        u16::MAX,
    ] {
        check_u16(value);
    }
    for value in [
        0_u32,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        63,
        64,
        65,
        99,
        100,
        101,
        255,
        256,
        257,
        4_095,
        4_096,
        4_097,
        65_535,
        65_536,
        65_537,
        u32::MAX,
    ] {
        check_u32(value);
    }
    for value in [
        0_u64,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        63,
        64,
        65,
        99,
        100,
        101,
        255,
        256,
        257,
        4_095,
        4_096,
        4_097,
        65_535,
        65_536,
        65_537,
        4_294_967_295,
        4_294_967_296,
        4_294_967_297,
        u64::MAX,
    ] {
        check_u64(value);
    }
    for value in [
        0_usize,
        1,
        7,
        8,
        9,
        15,
        16,
        17,
        63,
        64,
        65,
        99,
        100,
        101,
        255,
        256,
        257,
        4_095,
        4_096,
        4_097,
        usize::MAX,
    ] {
        check_usize(value);
    }
}

fn proptest_config() -> Config {
    Config {
        cases: 512,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(0x0050_524f_4354_4f52),
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn generated_i16_values_match_libc(value in any::<i16>()) { check_i16(value); }
    #[test]
    fn generated_i32_values_match_libc(value in any::<i32>()) { check_i32(value); }
    #[test]
    fn generated_i64_values_match_libc(value in any::<i64>()) { check_i64(value); }
    #[test]
    fn generated_isize_values_match_libc(value in any::<isize>()) { check_isize(value); }
    #[test]
    fn generated_u16_values_match_libc(value in any::<u16>()) { check_u16(value); }
    #[test]
    fn generated_u32_values_match_libc(value in any::<u32>()) { check_u32(value); }
    #[test]
    fn generated_u64_values_match_libc(value in any::<u64>()) { check_u64(value); }
    #[test]
    fn generated_usize_values_match_libc(value in any::<usize>()) { check_usize(value); }
}
