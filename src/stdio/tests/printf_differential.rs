//! Differential tests for the integer and floating-point `printf` formatting
//! adapters.

use std::ffi::CStr;

use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

#[cfg(target_env = "gnu")]
use super::hex_float;
use super::{fixed, fixed_upper, general, general_upper, scientific, signed, unsigned};

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
            // containing exactly one conversion whose required variadic type
            // is this function's concrete `$abi_type`. The
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
define_oracle!(oracle_c_double, libc::c_double);

fn snprintf_c_double(c_format: &'static [u8], value: libc::c_double) -> String {
    let mut buffer = [0 as libc::c_char; BUFFER_SIZE];
    let format = CStr::from_bytes_with_nul(c_format)
        .expect("test format must have exactly one trailing NUL")
        .to_string_lossy();
    // SAFETY: callers supply a static, NUL-terminated format containing
    // exactly one conversion for a `double`. The destination is writable for
    // `BUFFER_SIZE` elements, and the initialized prefix is inspected only
    // after checking for errors and truncation.
    let written = unsafe {
        libc::snprintf(
            buffer.as_mut_ptr(),
            buffer.len(),
            c_format.as_ptr().cast(),
            value,
        )
    };
    assert!(written >= 0, "snprintf failed: format={format:?}");
    let written = written as usize;
    assert!(
        written < buffer.len(),
        "snprintf truncated output: format={format:?}, required={written}, capacity={}",
        buffer.len(),
    );
    // `snprintf` initialized these bytes before its trailing NUL.
    let output = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), written) };
    String::from_utf8(output.to_vec()).expect("floating-point output is ASCII")
}

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

macro_rules! fixed_matrix {
    ($value:expr, $type_name:literal) => {{
        let value = $value;
        let abi_value = value as libc::c_double;

        pair!(
            oracle_c_double,
            b"%f\0",
            abi_value,
            format!("{}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%F\0",
            abi_value,
            format!("{}", fixed_upper(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%lf\0",
            abi_value,
            format!("{}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%lF\0",
            abi_value,
            format!("{}", fixed_upper(value)),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%.0f\0",
            abi_value,
            format!("{:.0}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.1f\0",
            abi_value,
            format!("{:.1}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.2f\0",
            abi_value,
            format!("{:.2}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.6f\0",
            abi_value,
            format!("{:.6}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.17f\0",
            abi_value,
            format!("{:.17}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.0F\0",
            abi_value,
            format!("{:.0}", fixed_upper(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.6F\0",
            abi_value,
            format!("{:.6}", fixed_upper(value)),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%+f\0",
            abi_value,
            format!("{:+}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"% f\0",
            abi_value,
            format!("{}", fixed(value).space_sign()),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%+ f\0",
            abi_value,
            format!("{:+}", fixed(value).space_sign()),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%+F\0",
            abi_value,
            format!("{:+}", fixed_upper(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"% F\0",
            abi_value,
            format!("{}", fixed_upper(value).space_sign()),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%20.2f\0",
            abi_value,
            format!("{:20.2}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%-20.2f\0",
            abi_value,
            format!("{:<20.2}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%020.2f\0",
            abi_value,
            format!("{:020.2}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%-020.2f\0",
            abi_value,
            format!("{:<020.2}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%+020.2f\0",
            abi_value,
            format!("{:+020.2}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"% 020.2f\0",
            abi_value,
            format!("{:020.2}", fixed(value).space_sign()),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%#.0f\0",
            abi_value,
            format!("{:#.0}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%#20.0f\0",
            abi_value,
            format!("{:#20.0}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%#020.0f\0",
            abi_value,
            format!("{:#020.0}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%-#020.0f\0",
            abi_value,
            format!("{:<#020.0}", fixed(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%#.0F\0",
            abi_value,
            format!("{:#.0}", fixed_upper(value)),
            $type_name,
            value
        );
    }};
}

fn check_f32(value: f32) {
    fixed_matrix!(value, "f32");
}

fn check_f64(value: f64) {
    fixed_matrix!(value, "f64");
}

macro_rules! scientific_matrix {
    ($value:expr, $type_name:literal) => {{
        let value = $value;
        let abi_value = value as libc::c_double;

        pair!(
            oracle_c_double,
            b"%e\0",
            abi_value,
            format!("{:e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%E\0",
            abi_value,
            format!("{:E}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%le\0",
            abi_value,
            format!("{:e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%lE\0",
            abi_value,
            format!("{:E}", scientific(value)),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%.0e\0",
            abi_value,
            format!("{:.0e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.1e\0",
            abi_value,
            format!("{:.1e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.2e\0",
            abi_value,
            format!("{:.2e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.6e\0",
            abi_value,
            format!("{:.6e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.17e\0",
            abi_value,
            format!("{:.17e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.0E\0",
            abi_value,
            format!("{:.0E}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%.6E\0",
            abi_value,
            format!("{:.6E}", scientific(value)),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%+e\0",
            abi_value,
            format!("{:+e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"% e\0",
            abi_value,
            format!("{:e}", scientific(value).space_sign()),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%+ e\0",
            abi_value,
            format!("{:+e}", scientific(value).space_sign()),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%+E\0",
            abi_value,
            format!("{:+E}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"% E\0",
            abi_value,
            format!("{:E}", scientific(value).space_sign()),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%20.2e\0",
            abi_value,
            format!("{:20.2e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%-20.2e\0",
            abi_value,
            format!("{:<20.2e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%020.2e\0",
            abi_value,
            format!("{:020.2e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%-020.2e\0",
            abi_value,
            format!("{:<020.2e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%+020.2e\0",
            abi_value,
            format!("{:+020.2e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"% 020.2e\0",
            abi_value,
            format!("{:020.2e}", scientific(value).space_sign()),
            $type_name,
            value
        );

        pair!(
            oracle_c_double,
            b"%#.0e\0",
            abi_value,
            format!("{:#.0e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%#20.0e\0",
            abi_value,
            format!("{:#20.0e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%#020.0e\0",
            abi_value,
            format!("{:#020.0e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%-#020.0e\0",
            abi_value,
            format!("{:<#020.0e}", scientific(value)),
            $type_name,
            value
        );
        pair!(
            oracle_c_double,
            b"%#.0E\0",
            abi_value,
            format!("{:#.0E}", scientific(value)),
            $type_name,
            value
        );
    }};
}

fn check_scientific_f32(value: f32) {
    scientific_matrix!(value, "f32");
}

fn check_scientific_f64(value: f64) {
    scientific_matrix!(value, "f64");
}

macro_rules! general_matrix {
    ($value:expr, $type_name:literal) => {{
        let value = $value;
        let abi_value = value as libc::c_double;

        macro_rules! check {
            ($c_format:literal, $rust_format:literal, $wrapper:expr) => {
                pair!(
                    oracle_c_double,
                    concat!($c_format, "\0").as_bytes(),
                    abi_value,
                    format!($rust_format, $wrapper),
                    $type_name,
                    value
                )
            };
        }

        check!("%g", "{}", general(value));
        check!("%G", "{}", general_upper(value));
        check!("%lg", "{}", general(value));
        check!("%lG", "{}", general_upper(value));

        check!("%.0g", "{:.0}", general(value));
        check!("%.1g", "{:.1}", general(value));
        check!("%.2g", "{:.2}", general(value));
        check!("%.6g", "{:.6}", general(value));
        check!("%.17g", "{:.17}", general(value));
        check!("%.0G", "{:.0}", general_upper(value));
        check!("%.6G", "{:.6}", general_upper(value));

        check!("%+g", "{:+}", general(value));
        check!("% g", "{}", general(value).space_sign());
        check!("%+ g", "{:+}", general(value).space_sign());
        check!("%+G", "{:+}", general_upper(value));
        check!("% G", "{}", general_upper(value).space_sign());

        check!("%20.6g", "{:20.6}", general(value));
        check!("%-20.6g", "{:<20.6}", general(value));
        check!("%020.6g", "{:020.6}", general(value));
        check!("%-020.6g", "{:<020.6}", general(value));
        check!("%+020.6g", "{:+020.6}", general(value));
        check!("% 020.6g", "{:020.6}", general(value).space_sign());

        check!("%#.0g", "{:#.0}", general(value));
        check!("%#.6g", "{:#.6}", general(value));
        check!("%#20.6g", "{:#20.6}", general(value));
        check!("%#020.6g", "{:#020.6}", general(value));
        check!("%-#020.6g", "{:<#020.6}", general(value));
        check!("%#.6G", "{:#.6}", general_upper(value));
    }};
}

fn check_general_f32(value: f32) {
    general_matrix!(value, "f32");
}

fn check_general_f64(value: f64) {
    general_matrix!(value, "f64");
}

// These byte-for-byte checks intentionally use GNU's permitted subnormal
// spelling as the oracle; ordinary hexadecimal tests remain cross-platform.
#[cfg(target_env = "gnu")]
macro_rules! hex_float_matrix {
    ($value:expr, $type_name:literal) => {{
        let value = $value;
        let abi_value = libc::c_double::from(value);

        macro_rules! check {
            ($c_format:literal, $rust_format:literal, $wrapper:expr) => {
                pair!(
                    oracle_c_double,
                    concat!($c_format, "\0").as_bytes(),
                    abi_value,
                    format!($rust_format, $wrapper),
                    $type_name,
                    value
                )
            };
        }

        check!("%a", "{:x}", hex_float(value));
        check!("%A", "{:X}", hex_float(value));
        check!("%la", "{:x}", hex_float(value));
        check!("%lA", "{:X}", hex_float(value));

        check!("%.0a", "{:.0x}", hex_float(value));
        check!("%.1a", "{:.1x}", hex_float(value));
        check!("%.2a", "{:.2x}", hex_float(value));
        check!("%.6a", "{:.6x}", hex_float(value));
        check!("%.13a", "{:.13x}", hex_float(value));
        check!("%.20a", "{:.20x}", hex_float(value));
        check!("%.0A", "{:.0X}", hex_float(value));
        check!("%.6A", "{:.6X}", hex_float(value));

        check!("%+a", "{:+x}", hex_float(value));
        check!("% a", "{:x}", hex_float(value).space_sign());
        check!("%+ a", "{:+x}", hex_float(value).space_sign());
        check!("%+A", "{:+X}", hex_float(value));
        check!("% A", "{:X}", hex_float(value).space_sign());

        check!("%24.6a", "{:24.6x}", hex_float(value));
        check!("%-24.6a", "{:<24.6x}", hex_float(value));
        check!("%024.6a", "{:024.6x}", hex_float(value));
        check!("%-024.6a", "{:<024.6x}", hex_float(value));
        check!("%+024.6a", "{:+024.6x}", hex_float(value));
        check!("% 024.6a", "{:024.6x}", hex_float(value).space_sign());

        check!("%#.0a", "{:#.0x}", hex_float(value));
        check!("%#.6a", "{:#.6x}", hex_float(value));
        check!("%#24.0a", "{:#24.0x}", hex_float(value));
        check!("%#024.0a", "{:#024.0x}", hex_float(value));
        check!("%-#024.0a", "{:<#024.0x}", hex_float(value));
        check!("%#.6A", "{:#.6X}", hex_float(value));
    }};
}

#[cfg(target_env = "gnu")]
fn check_hex_float_f32(value: f32) {
    hex_float_matrix!(value, "f32");
}

#[cfg(target_env = "gnu")]
fn check_hex_float_f64(value: f64) {
    hex_float_matrix!(value, "f64");
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

#[test]
fn deterministic_f32_values_match_libc() {
    for value in [
        f32::NEG_INFINITY,
        f32::MIN,
        -65_536.5,
        -3.5,
        -2.5,
        -1.375,
        -1.125,
        -0.0,
        0.0,
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        1.125,
        1.375,
        2.5,
        3.5,
        65_536.5,
        f32::MAX,
        f32::INFINITY,
        f32::NAN,
        f32::from_bits(f32::NAN.to_bits() | (1 << 31)),
    ] {
        check_f32(value);
    }
}

#[test]
fn deterministic_f64_values_match_libc() {
    for value in [
        f64::NEG_INFINITY,
        -1.0e200,
        -65_536.5,
        -3.5,
        -2.5,
        -1.375,
        -1.125,
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        1.125,
        1.375,
        2.5,
        3.5,
        65_536.5,
        1.0e200,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(f64::NAN.to_bits() | (1 << 63)),
    ] {
        check_f64(value);
    }
}

#[test]
fn deterministic_f32_scientific_values_match_libc() {
    for value in [
        f32::NEG_INFINITY,
        f32::MIN,
        -1.0e20,
        -10.0,
        -9.999,
        -1.0,
        -0.1,
        -0.0,
        0.0,
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        0.1,
        1.0,
        1.125,
        1.375,
        2.5,
        3.5,
        9.5,
        9.999,
        10.0,
        1.0e20,
        f32::MAX,
        f32::INFINITY,
        f32::NAN,
        f32::from_bits(f32::NAN.to_bits() | (1 << 31)),
    ] {
        check_scientific_f32(value);
    }
}

#[test]
fn deterministic_f64_scientific_values_match_libc() {
    let below_one = f64::from_bits(1.0_f64.to_bits() - 1);
    let above_one = f64::from_bits(1.0_f64.to_bits() + 1);
    for value in [
        f64::NEG_INFINITY,
        -1.0e300,
        -10.0,
        -9.999,
        -1.0,
        -0.1,
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.1,
        below_one,
        1.0,
        above_one,
        1.125,
        1.375,
        2.5,
        3.5,
        9.5,
        9.999,
        10.0,
        1.0e300,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(f64::NAN.to_bits() | (1 << 63)),
    ] {
        check_scientific_f64(value);
    }
}

#[test]
fn deterministic_f32_general_values_match_libc() {
    for value in [
        f32::NEG_INFINITY,
        f32::MIN,
        -999_999.4,
        -0.000_099_999_95,
        -0.0,
        0.0,
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        0.000_009_999_995,
        0.000_01,
        0.000_1,
        0.999_999_5,
        1.0,
        9.999_995,
        99_999.95,
        999_999.4,
        f32::MAX,
        f32::INFINITY,
        f32::NAN,
        f32::from_bits(f32::NAN.to_bits() | (1 << 31)),
    ] {
        check_general_f32(value);
    }
}

#[test]
fn deterministic_f64_general_values_match_libc() {
    for value in [
        f64::NEG_INFINITY,
        -1.0e300,
        -999_999.4,
        -0.000_099_999_95,
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.000_009_999_995,
        0.000_01,
        0.000_1,
        0.999_999_5,
        1.0,
        9.999_995,
        99_999.95,
        999_999.4,
        1.0e300,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(f64::NAN.to_bits() | (1 << 63)),
    ] {
        check_general_f64(value);
    }
}

#[cfg(target_env = "gnu")]
#[test]
fn deterministic_f32_hex_float_values_match_libc() {
    for value in [
        f32::NEG_INFINITY,
        f32::MIN,
        -26.5,
        -2.0,
        -1.5,
        -1.0,
        -0.0,
        0.0,
        f32::from_bits(1),
        f32::from_bits((1_u32 << 23) - 1),
        f32::MIN_POSITIVE,
        1.0,
        1.09375,
        1.15625,
        1.25,
        1.5,
        f32::from_bits(0x3f80_0001),
        2.0,
        26.5,
        f32::MAX,
        f32::INFINITY,
        f32::NAN,
        f32::from_bits(f32::NAN.to_bits() | (1 << 31)),
    ] {
        check_hex_float_f32(value);
    }
}

#[cfg(target_env = "gnu")]
#[test]
fn deterministic_f64_hex_float_values_match_libc() {
    for value in [
        f64::NEG_INFINITY,
        f64::MIN,
        -26.5,
        -2.0,
        -1.5,
        -1.0,
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::from_bits((1_u64 << 52) - 1),
        f64::MIN_POSITIVE,
        1.0,
        1.09375,
        1.15625,
        1.25,
        1.5,
        f64::from_bits(1.0_f64.to_bits() + 1),
        2.0,
        26.5,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(f64::NAN.to_bits() | (1 << 63)),
    ] {
        check_hex_float_f64(value);
    }
}

fn check_dr_233_alternate_boundary(
    c_format: &'static [u8],
    abi_value: libc::c_double,
    wrapper_output: String,
    standard_output: &str,
    _gnu_output: &str,
) {
    assert_eq!(wrapper_output, standard_output);
    let libc_output = snprintf_c_double(c_format, abi_value);
    if libc_output == standard_output {
        return;
    }

    // WG14 DR 233 requires the alternate form to retain the significant
    // zeros. GNU printf is known to omit them at this rounding/style boundary.
    #[cfg(target_env = "gnu")]
    assert_eq!(libc_output, _gnu_output);
    #[cfg(not(target_env = "gnu"))]
    assert_eq!(libc_output, standard_output);
}

#[test]
fn dr_233_alternate_general_boundary_is_explicitly_checked_against_libc() {
    for value in [999_999.5_f32, -999_999.5_f32] {
        let negative = value.is_sign_negative();
        check_dr_233_alternate_boundary(
            b"%#.6g\0",
            libc::c_double::from(value),
            format!("{:#.6}", general(value)),
            if negative {
                "-1.00000e+06"
            } else {
                "1.00000e+06"
            },
            if negative { "-1.e+06" } else { "1.e+06" },
        );
        check_dr_233_alternate_boundary(
            b"%#.6G\0",
            libc::c_double::from(value),
            format!("{:#.6}", general_upper(value)),
            if negative {
                "-1.00000E+06"
            } else {
                "1.00000E+06"
            },
            if negative { "-1.E+06" } else { "1.E+06" },
        );
    }

    for value in [999_999.5_f64, -999_999.5_f64] {
        let negative = value.is_sign_negative();
        check_dr_233_alternate_boundary(
            b"%#.6g\0",
            value,
            format!("{:#.6}", general(value)),
            if negative {
                "-1.00000e+06"
            } else {
                "1.00000e+06"
            },
            if negative { "-1.e+06" } else { "1.e+06" },
        );
        check_dr_233_alternate_boundary(
            b"%#.6G\0",
            value,
            format!("{:#.6}", general_upper(value)),
            if negative {
                "-1.00000E+06"
            } else {
                "1.00000E+06"
            },
            if negative { "-1.E+06" } else { "1.E+06" },
        );
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
    #[test]
    fn generated_f32_values_match_libc(bits in any::<u32>()) { check_f32(f32::from_bits(bits)); }
    #[test]
    fn generated_f64_values_match_libc(
        value in any::<u64>()
            .prop_map(f64::from_bits)
            .prop_filter("fixed output must fit the oracle buffer", |value| {
                !value.is_finite() || value.abs() <= 1.0e200
            })
    ) {
        check_f64(value);
    }
    #[test]
    fn generated_f32_scientific_values_match_libc(bits in any::<u32>()) {
        check_scientific_f32(f32::from_bits(bits));
    }
    #[test]
    fn generated_f64_scientific_values_match_libc(bits in any::<u64>()) {
        check_scientific_f64(f64::from_bits(bits));
    }
    #[test]
    fn generated_f32_general_values_match_libc(bits in any::<u32>()) {
        check_general_f32(f32::from_bits(bits));
    }
    #[test]
    fn generated_f64_general_values_match_libc(bits in any::<u64>()) {
        check_general_f64(f64::from_bits(bits));
    }
}

#[cfg(target_env = "gnu")]
proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn generated_f32_hex_float_values_match_libc(bits in any::<u32>()) {
        check_hex_float_f32(f32::from_bits(bits));
    }
    #[test]
    fn generated_f64_hex_float_values_match_libc(bits in any::<u64>()) {
        check_hex_float_f64(f64::from_bits(bits));
    }
}
