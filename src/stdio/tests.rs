use std::io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::process::{Command, Stdio};

use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, RngSeed};

#[cfg(target_os = "linux")]
use std::ffi::OsString;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicUsize, Ordering};

use super::printf::{
    byte_string, fixed, fixed_upper, general, general_upper, hex_float, scientific, signed,
    unsigned,
};
use super::{
    fgetc, fgets, fputc, fputs, fread, fseek, ftell, fwrite, getchar, putchar, puts, rewind,
};
#[cfg(target_os = "linux")]
use super::{remove, rename};

#[cfg(unix)]
mod printf_differential;

const STANDARD_STREAM_CHILD: &str = "PROCTOR_LIBC_STANDARD_STREAM_CHILD";

fn unwrap_stdio<T>((value, status): (T, io::Result<()>)) -> T {
    status.unwrap();
    value
}

#[test]
fn signed_formats_every_supported_primitive_without_widening_at_the_call_site() {
    macro_rules! assert_type {
        ($ty:ty) => {
            for value in [<$ty>::MIN, 0, <$ty>::MAX] {
                assert_eq!(format!("{}", signed(value)), format!("{value}"));
                assert_eq!(format!("{:+}", signed(value)), format!("{value:+}"));
                assert_eq!(format!("{:30}", signed(value)), format!("{value:30}"));
                assert_eq!(format!("{:<30}", signed(value)), format!("{value:<30}"));
                assert_eq!(format!("{:030}", signed(value)), format!("{value:030}"));
            }
        };
    }

    assert_type!(i8);
    assert_type!(i16);
    assert_type!(i32);
    assert_type!(i64);
    assert_type!(isize);
}

#[test]
fn unsigned_formats_every_supported_primitive_without_widening_at_the_call_site() {
    macro_rules! assert_type {
        ($ty:ty) => {
            for value in [<$ty>::MIN, 1, <$ty>::MAX] {
                assert_eq!(format!("{}", unsigned(value)), format!("{value}"));
                assert_eq!(format!("{:o}", unsigned(value)), format!("{value:o}"));
                assert_eq!(format!("{:x}", unsigned(value)), format!("{value:x}"));
                assert_eq!(format!("{:X}", unsigned(value)), format!("{value:X}"));
                assert_eq!(format!("{:30}", unsigned(value)), format!("{value:30}"));
                assert_eq!(format!("{:<30}", unsigned(value)), format!("{value:<30}"));
                assert_eq!(format!("{:030X}", unsigned(value)), format!("{value:030X}"));
            }
        };
    }

    assert_type!(u8);
    assert_type!(u16);
    assert_type!(u32);
    assert_type!(u64);
    assert_type!(usize);
}

#[test]
fn signed_precision_has_printf_minimum_digit_semantics() {
    assert_eq!(format!("{:.0}", signed(0_i8)), "");
    assert_eq!(format!("{:+.0}", signed(0_i8)), "+");
    assert_eq!(format!("{:.0}", signed(0_i8).space_sign()), " ");
    assert_eq!(format!("{:.0}", signed(1_i8)), "1");
    assert_eq!(format!("{:.5}", signed(42_i16)), "00042");
    assert_eq!(format!("{:.5}", signed(-42_i16)), "-00042");
    assert_eq!(format!("{:.2}", signed(123_i32)), "123");
}

#[test]
fn signed_sign_flags_follow_printf_precedence() {
    assert_eq!(format!("{}", signed(42_i32).space_sign()), " 42");
    assert_eq!(format!("{}", signed(0_i32).space_sign()), " 0");
    assert_eq!(format!("{}", signed(-42_i32).space_sign()), "-42");
    assert_eq!(format!("{:+}", signed(42_i32).space_sign()), "+42");
    assert_eq!(format!("{:+}", signed(-42_i32).space_sign()), "-42");
}

#[test]
fn signed_width_alignment_zero_padding_and_precision_interact_like_printf() {
    assert_eq!(format!("{:8}", signed(42_i32)), "      42");
    assert_eq!(format!("{:<8}", signed(42_i32)), "42      ");
    assert_eq!(format!("{:08}", signed(-42_i32)), "-0000042");
    assert_eq!(format!("{:+08}", signed(42_i32)), "+0000042");
    assert_eq!(format!("{:08}", signed(42_i32).space_sign()), " 0000042");
    assert_eq!(format!("{:<08}", signed(42_i32)), "42      ");

    // Integer precision disables the `0` flag, while the sign remains part of
    // the field width.
    assert_eq!(format!("{:08.5}", signed(42_i32)), "   00042");
    assert_eq!(format!("{:+08.3}", signed(42_i32)), "    +042");
    assert_eq!(format!("{:<8.5}", signed(-42_i32)), "-00042  ");
}

#[test]
fn unsigned_precision_applies_to_every_conversion() {
    assert_eq!(format!("{:.0}", unsigned(0_u8)), "");
    assert_eq!(format!("{:.5}", unsigned(42_u16)), "00042");
    assert_eq!(format!("{:.5o}", unsigned(0o52_u16)), "00052");
    assert_eq!(format!("{:.5x}", unsigned(0x2a_u16)), "0002a");
    assert_eq!(format!("{:.5X}", unsigned(0x2a_u16)), "0002A");
    assert_eq!(format!("{:.2x}", unsigned(0x123_u16)), "123");
}

#[test]
fn unsigned_alternate_octal_uses_a_leading_zero_digit() {
    assert_eq!(format!("{:#o}", unsigned(0_u32)), "0");
    assert_eq!(format!("{:#o}", unsigned(8_u32)), "010");
    assert_eq!(format!("{:#.0o}", unsigned(0_u32)), "0");
    assert_eq!(format!("{:#.2o}", unsigned(8_u32)), "010");
    assert_eq!(format!("{:#.3o}", unsigned(8_u32)), "010");
}

#[test]
fn unsigned_alternate_hexadecimal_omits_the_prefix_for_zero() {
    assert_eq!(format!("{:#x}", unsigned(0_u32)), "0");
    assert_eq!(format!("{:#X}", unsigned(0_u32)), "0");
    assert_eq!(format!("{:#.0x}", unsigned(0_u32)), "");
    assert_eq!(format!("{:#.0X}", unsigned(0_u32)), "");
    assert_eq!(format!("{:#x}", unsigned(0x2a_u32)), "0x2a");
    assert_eq!(format!("{:#X}", unsigned(0x2a_u32)), "0X2A");
}

#[test]
fn unsigned_width_zero_padding_precision_and_prefixes_interact_like_printf() {
    assert_eq!(format!("{:8.5}", unsigned(42_u32)), "   00042");
    assert_eq!(format!("{:08.5}", unsigned(42_u32)), "   00042");
    assert_eq!(format!("{:<8.5}", unsigned(42_u32)), "00042   ");
    assert_eq!(format!("{:#08x}", unsigned(0x2a_u32)), "0x00002a");
    assert_eq!(format!("{:#08X}", unsigned(0x2a_u32)), "0X00002A");
    assert_eq!(format!("{:#08.4x}", unsigned(0x2a_u32)), "  0x002a");
    assert_eq!(format!("{:#08.0x}", unsigned(0_u32)), "        ");
    assert_eq!(format!("{:#05.0o}", unsigned(0_u32)), "    0");
    assert_eq!(format!("{:<#8x}", unsigned(0x2a_u32)), "0x2a    ");
}

#[test]
fn integer_padding_larger_than_an_internal_chunk_is_complete() {
    let space_padded = format!("{:130}", signed(42_i32));
    assert_eq!(space_padded.len(), 130);
    assert!(space_padded.starts_with(&" ".repeat(128)));
    assert!(space_padded.ends_with("42"));

    let zero_padded = format!("{:0130x}", unsigned(0x2a_u32));
    assert_eq!(zero_padded.len(), 130);
    assert!(zero_padded.starts_with(&"0".repeat(128)));
    assert!(zero_padded.ends_with("2a"));
}

#[test]
fn fixed_formats_every_supported_type_without_conversion_at_the_call_site() {
    assert_eq!(format!("{}", fixed(1.25_f32)), "1.250000");
    assert_eq!(format!("{}", fixed(1.25_f64)), "1.250000");
    assert_eq!(format!("{}", fixed(f128::f128::new(1.25_f64))), "1.250000");

    assert_eq!(format!("{}", fixed_upper(1.25_f32)), "1.250000");
    assert_eq!(format!("{}", fixed_upper(1.25_f64)), "1.250000");
    assert_eq!(
        format!("{}", fixed_upper(f128::f128::new(1.25_f64))),
        "1.250000"
    );
}

#[test]
fn fixed_precision_rounds_to_nearest_with_ties_to_even() {
    for output in [
        format!("{:.0}", fixed(2.5_f32)),
        format!("{:.0}", fixed(2.5_f64)),
        format!("{:.0}", fixed(f128::f128::new(2.5_f64))),
    ] {
        assert_eq!(output, "2");
    }
    for output in [
        format!("{:.0}", fixed(3.5_f32)),
        format!("{:.0}", fixed(3.5_f64)),
        format!("{:.0}", fixed(f128::f128::new(3.5_f64))),
    ] {
        assert_eq!(output, "4");
    }

    assert_eq!(format!("{:.2}", fixed(1.125_f64)), "1.12");
    assert_eq!(format!("{:.2}", fixed(1.375_f64)), "1.38");
    assert_eq!(format!("{:.2}", fixed(9.999_f64)), "10.00");
    assert_eq!(
        format!("{:.2}", fixed(f128::f128::parse("1.125").unwrap())),
        "1.12"
    );
    assert_eq!(
        format!("{:.2}", fixed(f128::f128::parse("1.375").unwrap())),
        "1.38"
    );
}

#[test]
fn fixed_sign_flags_and_negative_zero_follow_printf() {
    assert_eq!(format!("{}", fixed(0.0_f64)), "0.000000");
    assert_eq!(format!("{}", fixed(-0.0_f64)), "-0.000000");
    assert_eq!(format!("{:+}", fixed(0.0_f64)), "+0.000000");
    assert_eq!(format!("{}", fixed(0.0_f64).space_sign()), " 0.000000");
    assert_eq!(format!("{:+}", fixed(0.0_f64).space_sign()), "+0.000000");
    assert_eq!(format!("{}", fixed(-0.0_f64).space_sign()), "-0.000000");

    assert_eq!(format!("{}", fixed(f128::f128::NEG_ZERO)), "-0.000000");
    assert_eq!(
        format!("{}", fixed_upper(f128::f128::ZERO).space_sign()),
        " 0.000000"
    );
}

#[test]
fn fixed_width_alignment_zero_padding_and_alternate_form_follow_printf() {
    assert_eq!(format!("{:8.2}", fixed(1.25_f64)), "    1.25");
    assert_eq!(format!("{:<8.2}", fixed(1.25_f64)), "1.25    ");
    assert_eq!(format!("{:08.2}", fixed(-1.25_f64)), "-0001.25");
    assert_eq!(format!("{:+08.2}", fixed(1.25_f64)), "+0001.25");
    assert_eq!(format!("{:08.2}", fixed(1.25_f64).space_sign()), " 0001.25");
    assert_eq!(format!("{:<08.2}", fixed(1.25_f64)), "1.25    ");
    assert_eq!(format!("{:#.0}", fixed(2.0_f64)), "2.");
    assert_eq!(format!("{:#08.0}", fixed(2.0_f64)), "0000002.");
    assert_eq!(format!("{:<#08.0}", fixed(2.0_f64)), "2.      ");
    assert_eq!(format!("{:.0}", fixed(2.0_f64)), "2");
}

#[test]
fn fixed_handles_finite_extrema_and_subnormals() {
    assert_eq!(format!("{:.6}", fixed(f32::MIN_POSITIVE)), "0.000000");
    assert_eq!(format!("{:.6}", fixed(f32::from_bits(1))), "0.000000");
    assert_eq!(format!("{:.6}", fixed(f64::MIN_POSITIVE)), "0.000000");
    assert_eq!(format!("{:.6}", fixed(f64::from_bits(1))), "0.000000");
    assert_eq!(
        format!("{:.6}", fixed(f128::f128::MIN_POSITIVE)),
        "0.000000"
    );
    assert_eq!(
        format!("{:.6}", fixed(f128::f128::MIN_POSITIVE_SUBNORMAL)),
        "0.000000"
    );

    assert!(
        format!("{:.0}", fixed(f32::MAX)).starts_with("340282346638528859811704183484516925440")
    );
    assert!(
        format!("{:.0}", fixed(f64::MAX)).starts_with("179769313486231570814527423731704356798")
    );
    let f128_max = format!("{:.0}", fixed(f128::f128::MAX));
    assert_eq!(f128_max.len(), 4933);
    assert!(f128_max.starts_with("118973149535723176508575932662800701619"));
}

#[test]
fn fixed_nonfinite_spelling_signs_and_padding_follow_printf() {
    assert_eq!(format!("{}", fixed(f64::INFINITY)), "inf");
    assert_eq!(format!("{}", fixed(f64::NEG_INFINITY)), "-inf");
    assert_eq!(format!("{}", fixed(f64::NAN)), "nan");
    assert_eq!(format!("{}", fixed_upper(f64::INFINITY)), "INF");
    assert_eq!(format!("{}", fixed_upper(f64::NEG_INFINITY)), "-INF");
    assert_eq!(format!("{}", fixed_upper(f64::NAN)), "NAN");
    assert_eq!(format!("{:+}", fixed(f64::INFINITY)), "+inf");
    assert_eq!(format!("{}", fixed(f64::NAN).space_sign()), " nan");
    assert_eq!(format!("{:08}", fixed(f64::INFINITY)), "     inf");
    assert_eq!(format!("{:<8}", fixed_upper(f64::NAN)), "NAN     ");

    assert_eq!(format!("{}", fixed(f128::f128::INFINITY)), "inf");
    assert_eq!(format!("{}", fixed_upper(f128::f128::NEG_INFINITY)), "-INF");
    assert_eq!(format!("{}", fixed(f128::f128::NAN)), "nan");
}

#[test]
fn fixed_large_width_and_precision_are_complete() {
    let precise = format!("{:.129}", fixed(1.25_f64));
    assert_eq!(precise.len(), 131);
    assert!(precise.starts_with("1.25"));
    assert!(precise.ends_with(&"0".repeat(127)));

    let padded = format!("{:200.129}", fixed(-1.25_f64));
    assert_eq!(padded.len(), 200);
    assert!(padded.starts_with(&" ".repeat(68)));
    assert!(padded.ends_with(&precise));
}

#[test]
fn fixed_is_consistent_across_exact_cross_type_conversions() {
    for value in [-16.0_f32, -1.25, -0.0, 0.0, 1.25, 16.0, 65_536.0] {
        let as_f64 = f64::from(value);
        let as_f128 = f128::f128::new(as_f64);
        for precision in [0, 1, 2, 6, 20] {
            let from_f32 = format!("{value:.precision$}", value = fixed(value));
            let from_f64 = format!("{value:.precision$}", value = fixed(as_f64));
            let from_f128 = format!("{value:.precision$}", value = fixed(as_f128));
            assert_eq!(from_f32, from_f64);
            assert_eq!(from_f64, from_f128);
        }
    }
}

fn f128_from_bits(bits: u128) -> f128::f128 {
    // SAFETY: `f128::f128` is a `repr(C)` wrapper around `[u8; 16]`, and every
    // IEEE binary128 bit pattern is a valid floating-point representation.
    unsafe { std::mem::transmute(bits.to_ne_bytes()) }
}

fn f128_invariant_config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(0x0046_3132_385f_464d),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(f128_invariant_config())]

    // These properties validate representation-independent invariants. Host
    // `%Lf` is deliberately not used as an oracle because its ABI may not be
    // IEEE binary128.
    #[test]
    fn generated_f128_values_satisfy_fixed_format_invariants(
        bits in any::<u128>(),
        precision in 0_usize..=20,
    ) {
        let value = f128_from_bits(bits);
        let lower = format!("{value:.precision$}", value = fixed(value));
        let upper = format!("{value:.precision$}", value = fixed_upper(value));
        prop_assert_eq!(lower.to_ascii_uppercase(), upper);

        let negative = bits >> 127 != 0;
        prop_assert_eq!(lower.starts_with('-'), negative);

        let exponent_field = (bits >> 112) & 0x7fff;
        if exponent_field != 0x7fff {
            let magnitude = lower.strip_prefix('-').unwrap_or(&lower);
            if precision == 0 {
                prop_assert!(!magnitude.contains('.'));
            } else {
                let (_, fraction) = magnitude
                    .split_once('.')
                    .expect("finite nonzero precision has a radix point");
                prop_assert_eq!(fraction.len(), precision);
                prop_assert!(fraction.bytes().all(|byte| byte.is_ascii_digit()));
            }
        }
    }
}

#[test]
fn scientific_formats_every_supported_type_without_conversion_at_the_call_site() {
    assert_eq!(format!("{:e}", scientific(1.25_f32)), "1.250000e+00");
    assert_eq!(format!("{:e}", scientific(1.25_f64)), "1.250000e+00");
    assert_eq!(
        format!("{:e}", scientific(f128::f128::new(1.25_f64))),
        "1.250000e+00"
    );

    assert_eq!(format!("{:E}", scientific(1.25_f32)), "1.250000E+00");
    assert_eq!(format!("{:E}", scientific(1.25_f64)), "1.250000E+00");
    assert_eq!(
        format!("{:E}", scientific(f128::f128::new(1.25_f64))),
        "1.250000E+00"
    );
}

#[test]
fn scientific_precision_rounds_to_nearest_with_ties_to_even_and_carries() {
    for output in [
        format!("{:.0e}", scientific(2.5_f32)),
        format!("{:.0e}", scientific(2.5_f64)),
        format!("{:.0e}", scientific(f128::f128::new(2.5_f64))),
    ] {
        assert_eq!(output, "2e+00");
    }
    for output in [
        format!("{:.0e}", scientific(3.5_f32)),
        format!("{:.0e}", scientific(3.5_f64)),
        format!("{:.0e}", scientific(f128::f128::new(3.5_f64))),
    ] {
        assert_eq!(output, "4e+00");
    }

    assert_eq!(format!("{:.2e}", scientific(1.125_f64)), "1.12e+00");
    assert_eq!(format!("{:.2e}", scientific(1.375_f64)), "1.38e+00");
    assert_eq!(format!("{:.0e}", scientific(9.5_f64)), "1e+01");
    assert_eq!(format!("{:.2e}", scientific(9.999_f64)), "1.00e+01");
    assert_eq!(
        format!("{:.2e}", scientific(f128::f128::parse("9.999").unwrap())),
        "1.00e+01"
    );
}

#[test]
fn scientific_decimal_exponents_are_exact_at_boundaries() {
    assert_eq!(format!("{:.6e}", scientific(0.1_f32)), "1.000000e-01");
    assert_eq!(format!("{:.6e}", scientific(10.0_f32)), "1.000000e+01");
    assert_eq!(format!("{:.6e}", scientific(0.0_f64)), "0.000000e+00");
    assert_eq!(format!("{:.6e}", scientific(0.1_f64)), "1.000000e-01");
    assert_eq!(format!("{:.6e}", scientific(1.0_f64)), "1.000000e+00");
    assert_eq!(format!("{:.6e}", scientific(10.0_f64)), "1.000000e+01");
    assert_eq!(format!("{:.6e}", scientific(1.0e100_f64)), "1.000000e+100");
    assert_eq!(format!("{:.6e}", scientific(1.0e-100_f64)), "1.000000e-100");
    assert_eq!(
        format!("{:.6e}", scientific(f128::f128::parse("1e1000").unwrap())),
        "1.000000e+1000"
    );
    assert_eq!(
        format!("{:.6e}", scientific(f128::f128::parse("1e-1000").unwrap())),
        "1.000000e-1000"
    );

    let below_one = f64::from_bits(1.0_f64.to_bits() - 1);
    let above_one = f64::from_bits(1.0_f64.to_bits() + 1);
    assert_eq!(
        format!("{:.17e}", scientific(below_one)),
        "9.99999999999999889e-01"
    );
    assert_eq!(
        format!("{:.17e}", scientific(above_one)),
        "1.00000000000000022e+00"
    );
}

#[test]
fn scientific_sign_flags_and_negative_zero_follow_printf() {
    assert_eq!(format!("{:e}", scientific(0.0_f64)), "0.000000e+00");
    assert_eq!(format!("{:e}", scientific(-0.0_f64)), "-0.000000e+00");
    assert_eq!(format!("{:+e}", scientific(0.0_f64)), "+0.000000e+00");
    assert_eq!(
        format!("{:e}", scientific(0.0_f64).space_sign()),
        " 0.000000e+00"
    );
    assert_eq!(
        format!("{:+e}", scientific(0.0_f64).space_sign()),
        "+0.000000e+00"
    );
    assert_eq!(
        format!("{:e}", scientific(-0.0_f64).space_sign()),
        "-0.000000e+00"
    );
    assert_eq!(
        format!("{:E}", scientific(f128::f128::NEG_ZERO)),
        "-0.000000E+00"
    );
}

#[test]
fn scientific_width_alignment_zero_padding_and_alternate_form_follow_printf() {
    assert_eq!(format!("{:12.2e}", scientific(1.25_f64)), "    1.25e+00");
    assert_eq!(format!("{:<12.2e}", scientific(1.25_f64)), "1.25e+00    ");
    assert_eq!(format!("{:012.2e}", scientific(-1.25_f64)), "-0001.25e+00");
    assert_eq!(format!("{:+012.2e}", scientific(1.25_f64)), "+0001.25e+00");
    assert_eq!(
        format!("{:012.2e}", scientific(1.25_f64).space_sign()),
        " 0001.25e+00"
    );
    assert_eq!(format!("{:<012.2e}", scientific(1.25_f64)), "1.25e+00    ");
    assert_eq!(format!("{:#.0e}", scientific(2.0_f64)), "2.e+00");
    assert_eq!(format!("{:.0E}", scientific(2.0_f64)), "2E+00");
    assert_eq!(format!("{:#012.0e}", scientific(2.0_f64)), "0000002.e+00");
    assert_eq!(format!("{:<#012.0e}", scientific(2.0_f64)), "2.e+00      ");
}

#[test]
fn scientific_handles_finite_extrema_subnormals_and_exponent_widths() {
    assert_eq!(
        format!("{:.6e}", scientific(f32::from_bits(1))),
        "1.401298e-45"
    );
    assert_eq!(format!("{:.6e}", scientific(f32::MAX)), "3.402823e+38");
    assert_eq!(
        format!("{:.6e}", scientific(f64::from_bits(1))),
        "4.940656e-324"
    );
    assert_eq!(format!("{:.6e}", scientific(f64::MAX)), "1.797693e+308");
    assert_eq!(
        format!("{:.6e}", scientific(f128::f128::MIN_POSITIVE_SUBNORMAL)),
        "6.475175e-4966"
    );
    assert_eq!(
        format!("{:.6e}", scientific(f128_from_bits(1_u128 << 112))),
        "3.362103e-4932"
    );
    assert_eq!(
        format!("{:.6E}", scientific(f128::f128::MAX)),
        "1.189731E+4932"
    );
}

#[test]
fn scientific_nonfinite_spelling_signs_and_padding_follow_printf() {
    assert_eq!(format!("{:e}", scientific(f64::INFINITY)), "inf");
    assert_eq!(format!("{:e}", scientific(f64::NEG_INFINITY)), "-inf");
    assert_eq!(format!("{:e}", scientific(f64::NAN)), "nan");
    assert_eq!(format!("{:E}", scientific(f64::INFINITY)), "INF");
    assert_eq!(format!("{:E}", scientific(f64::NEG_INFINITY)), "-INF");
    assert_eq!(format!("{:E}", scientific(f64::NAN)), "NAN");
    assert_eq!(format!("{:+e}", scientific(f64::INFINITY)), "+inf");
    assert_eq!(format!("{:e}", scientific(f64::NAN).space_sign()), " nan");
    assert_eq!(format!("{:08e}", scientific(f64::INFINITY)), "     inf");
    assert_eq!(format!("{:<8E}", scientific(f64::NAN)), "NAN     ");

    assert_eq!(format!("{:e}", scientific(f128::f128::INFINITY)), "inf");
    assert_eq!(
        format!("{:E}", scientific(f128::f128::NEG_INFINITY)),
        "-INF"
    );
    assert_eq!(format!("{:e}", scientific(f128::f128::NAN)), "nan");
}

#[test]
fn scientific_large_width_and_precision_are_complete() {
    let precise = format!("{:.129e}", scientific(1.25_f64));
    assert_eq!(precise.len(), 135);
    assert!(precise.starts_with("1.25"));
    assert!(precise.ends_with("e+00"));

    let padded = format!("{:200.129e}", scientific(-1.25_f64));
    assert_eq!(padded.len(), 200);
    assert!(padded.starts_with(&" ".repeat(64)));
    assert!(padded.ends_with(&format!("-{precise}")));
}

#[test]
fn scientific_is_consistent_across_exact_cross_type_conversions() {
    for value in [-16.0_f32, -1.25, -0.0, 0.0, 1.25, 16.0, 65_536.0] {
        let as_f64 = f64::from(value);
        let as_f128 = f128::f128::new(as_f64);
        for precision in [0, 1, 2, 6, 20] {
            let from_f32 = format!("{value:.precision$e}", value = scientific(value));
            let from_f64 = format!("{value:.precision$e}", value = scientific(as_f64));
            let from_f128 = format!("{value:.precision$e}", value = scientific(as_f128));
            assert_eq!(from_f32, from_f64);
            assert_eq!(from_f64, from_f128);
        }
    }
}

proptest! {
    #![proptest_config(f128_invariant_config())]

    // Host `%Le` is deliberately not used as an oracle because its ABI may
    // not be IEEE binary128.
    #[test]
    fn generated_f128_values_satisfy_scientific_format_invariants(
        bits in any::<u128>(),
        precision in 0_usize..=20,
    ) {
        let value = f128_from_bits(bits);
        let lower = format!("{value:.precision$e}", value = scientific(value));
        let upper = format!("{value:.precision$E}", value = scientific(value));
        prop_assert_eq!(lower.to_ascii_uppercase(), upper.as_str());

        let negative = bits >> 127 != 0;
        prop_assert_eq!(lower.starts_with('-'), negative);

        let exponent_field = (bits >> 112) & 0x7fff;
        if exponent_field != 0x7fff {
            let magnitude = lower.strip_prefix('-').unwrap_or(&lower);
            let (mantissa, exponent) = magnitude
                .split_once('e')
                .expect("finite scientific output has an exponent marker");
            prop_assert!(matches!(exponent.as_bytes().first(), Some(b'+') | Some(b'-')));
            prop_assert!(exponent[1..].len() >= 2);
            prop_assert!(exponent[1..].bytes().all(|byte| byte.is_ascii_digit()));
            if precision == 0 {
                prop_assert!(!mantissa.contains('.'));
                prop_assert_eq!(mantissa.len(), 1);
            } else {
                let (integer, fraction) = mantissa
                    .split_once('.')
                    .expect("nonzero precision has a radix point");
                prop_assert_eq!(integer.len(), 1);
                prop_assert_eq!(fraction.len(), precision);
                prop_assert!(mantissa
                    .bytes()
                    .filter(|&byte| byte != b'.')
                    .all(|byte| byte.is_ascii_digit()));
            }
        }
    }
}

#[test]
fn general_formats_every_supported_type_without_conversion_at_the_call_site() {
    assert_eq!(format!("{}", general(1.25_f32)), "1.25");
    assert_eq!(format!("{}", general(1.25_f64)), "1.25");
    assert_eq!(format!("{}", general(f128::f128::new(1.25_f64))), "1.25");

    assert_eq!(format!("{}", general_upper(1.0e6_f32)), "1E+06");
    assert_eq!(format!("{}", general_upper(1.0e6_f64)), "1E+06");
    assert_eq!(
        format!("{}", general_upper(f128::f128::new(1.0e6_f64))),
        "1E+06"
    );
}

#[test]
fn general_precision_is_significant_digits_and_zero_means_one() {
    assert_eq!(format!("{}", general(123.45_f64)), "123.45");
    assert_eq!(format!("{:.0}", general(12.5_f64)), "1e+01");
    assert_eq!(format!("{:.1}", general(12.5_f64)), "1e+01");
    assert_eq!(format!("{:.2}", general(12.5_f64)), "12");
    assert_eq!(format!("{:.3}", general(12.5_f64)), "12.5");
    assert_eq!(format!("{:.6}", general(123.45_f64)), "123.45");
}

#[test]
fn general_style_selection_uses_the_post_rounding_decimal_exponent() {
    assert_eq!(format!("{:.4}", general(0.000_1_f64)), "0.0001");
    assert_eq!(format!("{:.4}", general(0.000_01_f64)), "1e-05");
    assert_eq!(format!("{:.6}", general(99_999.0_f64)), "99999");
    assert_eq!(format!("{:.6}", general(100_000.0_f64)), "100000");
    assert_eq!(format!("{:.6}", general(1_000_000.0_f64)), "1e+06");

    assert_eq!(format!("{:.4}", general(9_999.6_f64)), "1e+04");
    assert_eq!(format!("{:#.4}", general(9_999.6_f64)), "1.000e+04");
    assert_eq!(format!("{:.4}", general(0.000_099_996_f64)), "0.0001");
    assert_eq!(format!("{:#.4}", general(0.000_099_996_f64)), "0.0001000");
}

#[test]
fn general_alternate_form_retains_significant_zeros_across_dr_233_boundary() {
    for output in [
        format!("{:#.6}", general(999_999.5_f32)),
        format!("{:#.6}", general(999_999.5_f64)),
        format!("{:#.6}", general(f128::f128::new(999_999.5_f64))),
    ] {
        assert_eq!(output, "1.00000e+06");
    }
    for output in [
        format!("{:#.6}", general(-999_999.5_f32)),
        format!("{:#.6}", general(-999_999.5_f64)),
        format!("{:#.6}", general(f128::f128::new(-999_999.5_f64))),
    ] {
        assert_eq!(output, "-1.00000e+06");
    }
    for output in [
        format!("{:#.6}", general_upper(999_999.5_f32)),
        format!("{:#.6}", general_upper(999_999.5_f64)),
        format!("{:#.6}", general_upper(f128::f128::new(999_999.5_f64))),
    ] {
        assert_eq!(output, "1.00000E+06");
    }
    for output in [
        format!("{:#.6}", general_upper(-999_999.5_f32)),
        format!("{:#.6}", general_upper(-999_999.5_f64)),
        format!("{:#.6}", general_upper(f128::f128::new(-999_999.5_f64))),
    ] {
        assert_eq!(output, "-1.00000E+06");
    }
}

#[test]
fn general_rounds_to_nearest_with_ties_to_even() {
    for output in [
        format!("{:.2}", general(1.25_f32)),
        format!("{:.2}", general(1.25_f64)),
        format!("{:.2}", general(f128::f128::new(1.25_f64))),
    ] {
        assert_eq!(output, "1.2");
    }
    for output in [
        format!("{:.2}", general(1.75_f32)),
        format!("{:.2}", general(1.75_f64)),
        format!("{:.2}", general(f128::f128::new(1.75_f64))),
    ] {
        assert_eq!(output, "1.8");
    }
    assert_eq!(format!("{:.1}", general(9.5_f64)), "1e+01");
}

#[test]
fn general_trims_fractional_zeros_unless_alternate_form_is_requested() {
    assert_eq!(format!("{:.6}", general(123.0_f64)), "123");
    assert_eq!(format!("{:#.6}", general(123.0_f64)), "123.000");
    assert_eq!(format!("{:.6}", general(1.23e10_f64)), "1.23e+10");
    assert_eq!(format!("{:#.6}", general(1.23e10_f64)), "1.23000e+10");
    assert_eq!(format!("{:#.1}", general(2.0_f64)), "2.");
}

#[test]
fn general_sign_width_alignment_and_zero_padding_follow_printf() {
    assert_eq!(format!("{}", general(-0.0_f64)), "-0");
    assert_eq!(format!("{:+}", general(0.0_f64)), "+0");
    assert_eq!(format!("{}", general(0.0_f64).space_sign()), " 0");
    assert_eq!(format!("{:+}", general(0.0_f64).space_sign()), "+0");
    assert_eq!(format!("{:10.4}", general(12.5_f64)), "      12.5");
    assert_eq!(format!("{:<10.4}", general(12.5_f64)), "12.5      ");
    assert_eq!(format!("{:010.4}", general(-12.5_f64)), "-0000012.5");
    assert_eq!(format!("{:+010.4}", general(12.5_f64)), "+0000012.5");
    assert_eq!(
        format!("{:010.4}", general(12.5_f64).space_sign()),
        " 0000012.5"
    );
}

#[test]
fn general_handles_finite_extrema_subnormals_and_exponent_widths() {
    assert_eq!(format!("{}", general(f32::from_bits(1))), "1.4013e-45");
    assert_eq!(format!("{}", general(f32::MAX)), "3.40282e+38");
    assert_eq!(format!("{}", general(f64::from_bits(1))), "4.94066e-324");
    assert_eq!(format!("{}", general(f64::MAX)), "1.79769e+308");
    assert_eq!(
        format!("{}", general(f128::f128::MIN_POSITIVE_SUBNORMAL)),
        "6.47518e-4966"
    );
    assert_eq!(
        format!("{}", general(f128_from_bits(1_u128 << 112))),
        "3.3621e-4932"
    );
    assert_eq!(
        format!("{}", general_upper(f128::f128::MAX)),
        "1.18973E+4932"
    );
}

#[test]
fn general_nonfinite_spelling_signs_and_padding_follow_printf() {
    assert_eq!(format!("{}", general(f64::INFINITY)), "inf");
    assert_eq!(format!("{}", general(f64::NEG_INFINITY)), "-inf");
    assert_eq!(format!("{}", general(f64::NAN)), "nan");
    assert_eq!(format!("{}", general_upper(f64::INFINITY)), "INF");
    assert_eq!(format!("{}", general_upper(f64::NEG_INFINITY)), "-INF");
    assert_eq!(format!("{}", general_upper(f64::NAN)), "NAN");
    assert_eq!(format!("{:+}", general(f64::INFINITY)), "+inf");
    assert_eq!(format!("{}", general(f64::NAN).space_sign()), " nan");
    assert_eq!(format!("{:08}", general(f64::INFINITY)), "     inf");
    assert_eq!(format!("{:<8}", general_upper(f64::NAN)), "NAN     ");

    assert_eq!(format!("{}", general(f128::f128::INFINITY)), "inf");
    assert_eq!(
        format!("{}", general_upper(f128::f128::NEG_INFINITY)),
        "-INF"
    );
}

#[test]
fn general_large_width_and_precision_are_complete() {
    let precise = format!("{:#.129}", general(1.25_f64));
    assert_eq!(precise.len(), 130);
    assert!(precise.starts_with("1.25"));
    assert!(precise.ends_with(&"0".repeat(126)));

    let padded = format!("{:200.129}", general(-1.25_f64));
    assert_eq!(padded.len(), 200);
    assert!(padded.starts_with(&" ".repeat(195)));
    assert!(padded.ends_with("-1.25"));
}

#[test]
fn general_is_consistent_across_exact_cross_type_conversions() {
    for value in [-16.0_f32, -1.25, -0.0, 0.0, 1.25, 16.0, 65_536.0] {
        let as_f64 = f64::from(value);
        let as_f128 = f128::f128::new(as_f64);
        for precision in [0, 1, 2, 6, 20] {
            let from_f32 = format!("{value:.precision$}", value = general(value));
            let from_f64 = format!("{value:.precision$}", value = general(as_f64));
            let from_f128 = format!("{value:.precision$}", value = general(as_f128));
            assert_eq!(from_f32, from_f64);
            assert_eq!(from_f64, from_f128);
        }
    }
}

proptest! {
    #![proptest_config(f128_invariant_config())]

    // Host `%Lg` is deliberately not used as an oracle because its ABI may
    // not be IEEE binary128.
    #[test]
    fn generated_f128_values_satisfy_general_format_invariants(
        bits in any::<u128>(),
        precision in 0_usize..=20,
    ) {
        let value = f128_from_bits(bits);
        let lower = format!("{value:.precision$}", value = general(value));
        let upper = format!("{value:.precision$}", value = general_upper(value));
        prop_assert_eq!(lower.to_ascii_uppercase(), upper.as_str());

        let negative = bits >> 127 != 0;
        prop_assert_eq!(lower.starts_with('-'), negative);

        let exponent_field = (bits >> 112) & 0x7fff;
        if exponent_field != 0x7fff {
            let magnitude = lower.strip_prefix('-').unwrap_or(&lower);
            prop_assert!(!magnitude.ends_with('.'));
            if let Some((mantissa, exponent)) = magnitude.split_once('e') {
                prop_assert!(matches!(exponent.as_bytes().first(), Some(b'+') | Some(b'-')));
                prop_assert!(exponent[1..].len() >= 2);
                prop_assert!(exponent[1..].bytes().all(|byte| byte.is_ascii_digit()));
                prop_assert!(!mantissa.ends_with('0') || !mantissa.contains('.'));
            } else if let Some((_, fraction)) = magnitude.split_once('.') {
                prop_assert!(!fraction.ends_with('0'));
            }
        }
    }
}

#[test]
fn hex_float_formats_every_supported_type_without_narrowing() {
    assert_eq!(format!("{:x}", hex_float(1.5_f32)), "0x1.8p+0");
    assert_eq!(format!("{:x}", hex_float(1.5_f64)), "0x1.8p+0");
    assert_eq!(
        format!("{:x}", hex_float(f128::f128::new(1.5_f64))),
        "0x1.8p+0"
    );
    assert_eq!(format!("{:X}", hex_float(26.5_f32)), "0X1.A8P+4");
    assert_eq!(format!("{:X}", hex_float(26.5_f64)), "0X1.A8P+4");
    assert_eq!(
        format!("{:X}", hex_float(f128::f128::new(26.5_f64))),
        "0X1.A8P+4"
    );
}

#[test]
fn hex_float_default_precision_is_exact_and_trims_fractional_zeros() {
    assert_eq!(format!("{:x}", hex_float(1.0_f64)), "0x1p+0");
    assert_eq!(format!("{:#x}", hex_float(1.0_f64)), "0x1.p+0");
    assert_eq!(format!("{:x}", hex_float(0.1_f64)), "0x1.999999999999ap-4");
    assert_eq!(format!("{:x}", hex_float(1.0_f32 / 10.0)), "0x1.99999ap-4");
    assert_eq!(
        format!("{:x}", hex_float(f128::f128::new(0.5_f64))),
        "0x1p-1"
    );
}

#[test]
fn hex_float_explicit_precision_rounds_ties_to_even_and_can_carry() {
    assert_eq!(format!("{:.0x}", hex_float(1.25_f64)), "0x1p+0");
    assert_eq!(format!("{:.0x}", hex_float(1.5_f64)), "0x2p+0");
    assert_eq!(format!("{:.1x}", hex_float(1.09375_f64)), "0x1.2p+0");
    assert_eq!(format!("{:.1x}", hex_float(1.15625_f64)), "0x1.2p+0");
    assert_eq!(format!("{:.2x}", hex_float(1.5_f64)), "0x1.80p+0");
    assert_eq!(format!("{:#.0X}", hex_float(1.5_f64)), "0X2.P+0");
}

#[test]
fn hex_float_sign_width_alignment_zero_padding_and_prefix_follow_printf() {
    assert_eq!(format!("{:x}", hex_float(-0.0_f64)), "-0x0p+0");
    assert_eq!(format!("{:+x}", hex_float(0.0_f64)), "+0x0p+0");
    assert_eq!(format!("{:x}", hex_float(0.0_f64).space_sign()), " 0x0p+0");
    assert_eq!(format!("{:+x}", hex_float(0.0_f64).space_sign()), "+0x0p+0");
    assert_eq!(format!("{:12.2x}", hex_float(1.5_f64)), "   0x1.80p+0");
    assert_eq!(format!("{:<12.2x}", hex_float(1.5_f64)), "0x1.80p+0   ");
    assert_eq!(format!("{:012.2x}", hex_float(-1.5_f64)), "-0x001.80p+0");
    assert_eq!(format!("{:+012.2X}", hex_float(1.5_f64)), "+0X001.80P+0");
    assert_eq!(format!("{:<012.2x}", hex_float(1.5_f64)), "0x1.80p+0   ");
}

#[test]
fn hex_float_f32_uses_the_promoted_binary64_representation() {
    assert_eq!(format!("{:x}", hex_float(f32::from_bits(1))), "0x1p-149");
    assert_eq!(format!("{:x}", hex_float(f32::MIN_POSITIVE)), "0x1p-126");
    assert_eq!(
        format!("{:x}", hex_float(f32::from_bits(0x3f80_0001))),
        "0x1.000002p+0"
    );
}

#[test]
fn hex_float_uses_the_gnu_subnormal_convention_and_handles_boundaries() {
    assert_eq!(
        format!("{:x}", hex_float(f64::from_bits(1))),
        "0x0.0000000000001p-1022"
    );
    assert_eq!(
        format!("{:x}", hex_float(f64::from_bits((1_u64 << 52) - 1))),
        "0x0.fffffffffffffp-1022"
    );
    assert_eq!(
        format!("{:.0x}", hex_float(f64::from_bits((1_u64 << 52) - 1))),
        "0x1p-1022"
    );
    assert_eq!(format!("{:x}", hex_float(f64::MIN_POSITIVE)), "0x1p-1022");
    assert_eq!(
        format!("{:x}", hex_float(f64::MAX)),
        "0x1.fffffffffffffp+1023"
    );

    let f128_min_subnormal = f128_from_bits(1);
    assert_eq!(
        format!("{:x}", hex_float(f128_min_subnormal)),
        format!("0x0.{}1p-16382", "0".repeat(27))
    );
    assert_eq!(
        format!("{:x}", hex_float(f128_from_bits(1_u128 << 112))),
        "0x1p-16382"
    );
    assert_eq!(
        format!("{:x}", hex_float(f128::f128::MAX)),
        format!("0x1.{}p+16383", "f".repeat(28))
    );
}

#[test]
fn hex_float_nonfinite_spelling_signs_and_padding_follow_printf() {
    assert_eq!(format!("{:x}", hex_float(f64::INFINITY)), "inf");
    assert_eq!(format!("{:X}", hex_float(f64::INFINITY)), "INF");
    assert_eq!(format!("{:x}", hex_float(f64::NEG_INFINITY)), "-inf");
    assert_eq!(format!("{:X}", hex_float(f64::NAN)), "NAN");
    assert_eq!(format!("{:+x}", hex_float(f64::INFINITY)), "+inf");
    assert_eq!(format!("{:x}", hex_float(f64::NAN).space_sign()), " nan");
    assert_eq!(format!("{:08x}", hex_float(f64::INFINITY)), "     inf");
    assert_eq!(format!("{:<8X}", hex_float(f64::NAN)), "NAN     ");
}

#[test]
fn hex_float_large_precision_and_cross_type_values_are_complete() {
    let precise = format!("{:#.129x}", hex_float(1.5_f64));
    assert_eq!(precise.len(), 136);
    assert!(precise.starts_with("0x1.8"));
    assert!(precise.ends_with("p+0"));
    assert_eq!(precise.matches('0').count(), 130);

    for value in [-16.0_f32, -1.5, -0.0, 0.0, 1.5, 16.0, 65_536.0] {
        let as_f64 = f64::from(value);
        let as_f128 = f128::f128::new(as_f64);
        for precision in [0, 1, 2, 13, 28] {
            let from_f32 = format!("{value:.precision$x}", value = hex_float(value));
            let from_f64 = format!("{value:.precision$x}", value = hex_float(as_f64));
            let from_f128 = format!("{value:.precision$x}", value = hex_float(as_f128));
            assert_eq!(from_f32, from_f64);
            assert_eq!(from_f64, from_f128);
        }
    }
}

proptest! {
    #![proptest_config(f128_invariant_config())]

    // Host `%La` is deliberately not used as an oracle because its ABI may
    // not be IEEE binary128.
    #[test]
    fn generated_f128_values_satisfy_hex_float_invariants(
        bits in any::<u128>(),
        precision in 0_usize..=32,
    ) {
        let value = f128_from_bits(bits);
        let lower = format!("{value:.precision$x}", value = hex_float(value));
        let upper = format!("{value:.precision$X}", value = hex_float(value));
        prop_assert_eq!(lower.to_ascii_uppercase(), upper.as_str());

        let negative = bits >> 127 != 0;
        prop_assert_eq!(lower.starts_with('-'), negative);

        let exponent_field = (bits >> 112) & 0x7fff;
        if exponent_field != 0x7fff {
            let magnitude = lower.strip_prefix('-').unwrap_or(&lower);
            prop_assert!(magnitude.starts_with("0x"));
            let (mantissa, exponent) = magnitude[2..]
                .split_once('p')
                .expect("finite hexadecimal output has an exponent marker");
            prop_assert!(matches!(exponent.as_bytes().first(), Some(b'+') | Some(b'-')));
            prop_assert!(exponent[1..].bytes().all(|byte| byte.is_ascii_digit()));
            if precision == 0 {
                prop_assert!(!mantissa.contains('.'));
            } else {
                let (_, fraction) = mantissa
                    .split_once('.')
                    .expect("nonzero precision has a radix point");
                prop_assert_eq!(fraction.len(), precision);
                prop_assert!(fraction.bytes().all(|byte| byte.is_ascii_hexdigit()));
            }
        }
    }
}

fn c_string_bytes(value: &str) -> Vec<i8> {
    value
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .map(|byte| byte as i8)
        .collect()
}

#[test]
fn byte_string_selects_ascii_bytes_through_the_first_nul() {
    let value = [
        b'h' as i8, b'e' as i8, b'l' as i8, b'l' as i8, b'o' as i8, 0,
    ];
    assert_eq!(format!("{}", byte_string(&value)), "hello");

    let embedded = [b'o' as i8, b'k' as i8, 0, b'x' as i8, 0];
    assert_eq!(format!("{}", byte_string(&embedded)), "ok");
    assert_eq!(format!("{:.4}", byte_string(&embedded)), "ok");
}

#[test]
fn byte_string_precision_can_bound_a_slice_without_a_nul() {
    let value = [b'a' as i8, b'b' as i8, b'c' as i8, 0xff_u8 as i8];
    assert_eq!(format!("{:.3}", byte_string(&value)), "abc");
    assert_eq!(format!("{:.0}", byte_string(&value)), "");
    assert_eq!(format!("{:.0}", byte_string(&[])), "");
}

#[test]
fn byte_string_width_alignment_and_precision_count_bytes() {
    let value = c_string_bytes("hello");
    assert_eq!(format!("{:10}", byte_string(&value)), "     hello");
    assert_eq!(format!("{:<10}", byte_string(&value)), "hello     ");
    assert_eq!(format!("{:10.3}", byte_string(&value)), "       hel");
    assert_eq!(format!("{:<10.3}", byte_string(&value)), "hel       ");

    let multibyte = c_string_bytes("é!");
    assert_eq!(
        format!("{}", byte_string(&multibyte)).as_bytes(),
        b"\xc3\xa9!"
    );
    assert_eq!(
        format!("{:5}", byte_string(&multibyte)).as_bytes(),
        b"  \xc3\xa9!"
    );
    assert_eq!(
        format!("{:<5}", byte_string(&multibyte)).as_bytes(),
        b"\xc3\xa9!  "
    );
    assert_eq!(
        format!("{:.2}", byte_string(&multibyte)).as_bytes(),
        b"\xc3\xa9"
    );
    assert_eq!(
        format!("{:5.2}", byte_string(&multibyte)).as_bytes(),
        b"   \xc3\xa9"
    );
}

#[test]
fn byte_string_handles_empty_large_fields_and_borrows_without_mutation() {
    let empty = [0_i8];
    assert_eq!(format!("{}", byte_string(&empty)), "");
    assert_eq!(format!("{:130}", byte_string(&empty)), " ".repeat(130));

    let value = c_string_bytes("chunked padding");
    let before = value.clone();
    let output = format!("{:200}", byte_string(&value));
    assert_eq!(output.len(), 200);
    assert!(output.starts_with(&" ".repeat(185)));
    assert!(output.ends_with("chunked padding"));
    assert_eq!(value, before);
}

fn byte_string_proptest_config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(0x4259_5445_5f53_5452),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(byte_string_proptest_config())]

    #[test]
    fn generated_valid_utf8_byte_strings_preserve_selected_bytes_and_byte_width(
        characters in proptest::collection::vec(
            prop_oneof![4 => any::<char>(), 1 => Just('\0')],
            0..=32,
        ),
        boundary_selector in any::<usize>(),
        width in 0_usize..=160,
    ) {
        let text: String = characters.into_iter().collect();
        let value = c_string_bytes(&text);
        let before = value.clone();
        let bytes = text.as_bytes();
        let nul = bytes.iter().position(|&byte| byte == 0).unwrap_or(bytes.len());
        let selected_text = &text[..nul];

        let output = format!("{wrapped:width$}", wrapped = byte_string(&value));
        let expected_padding = width.saturating_sub(selected_text.len());
        prop_assert_eq!(
            output.as_bytes(),
            [" ".repeat(expected_padding).as_bytes(), selected_text.as_bytes()].concat(),
        );
        let left_aligned = format!("{wrapped:<width$}", wrapped = byte_string(&value));
        prop_assert_eq!(
            left_aligned.as_bytes(),
            [selected_text.as_bytes(), " ".repeat(expected_padding).as_bytes()].concat(),
        );

        let mut boundaries: Vec<usize> = selected_text.char_indices().map(|(index, _)| index).collect();
        boundaries.push(selected_text.len());
        let precision = boundaries[boundary_selector % boundaries.len()];
        let selected = &selected_text.as_bytes()[..precision];
        let precise = format!(
            "{wrapped:width$.precision$}",
            wrapped = byte_string(&value),
        );
        let precise_padding = width.saturating_sub(selected.len());
        prop_assert_eq!(
            precise.as_bytes(),
            [" ".repeat(precise_padding).as_bytes(), selected].concat(),
        );
        prop_assert_eq!(value, before);
    }
}

#[cfg(target_os = "linux")]
struct StdioTestDir(PathBuf);

#[cfg(target_os = "linux")]
impl StdioTestDir {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "proctor-libc-stdio-test-{}-{id}",
                std::process::id()
            ));

            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("failed to create stdio test directory: {error}"),
            }
        }
    }

    fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }
}

#[cfg(target_os = "linux")]
impl Drop for StdioTestDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            if std::thread::panicking() {
                eprintln!(
                    "failed to clean up stdio test directory {}: {error}",
                    self.0.display()
                );
            } else {
                panic!(
                    "failed to clean up stdio test directory {}: {error}",
                    self.0.display()
                );
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn null_terminated_path(path: &Path) -> Vec<i8> {
    let mut bytes: Vec<i8> = path
        .as_os_str()
        .as_bytes()
        .iter()
        .map(|&byte| byte as i8)
        .collect();
    bytes.push(0);
    bytes
}

#[cfg(target_os = "linux")]
fn assert_path_missing(path: &Path) {
    assert_eq!(
        fs::symlink_metadata(path).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn standard_stream_child() {
    match std::env::var(STANDARD_STREAM_CHILD).as_deref() {
        Err(std::env::VarError::NotPresent) => {}
        Ok("getchar") => {
            assert_eq!(unwrap_stdio(getchar()), 0);
            assert_eq!(unwrap_stdio(getchar()), 127);
            assert_eq!(unwrap_stdio(getchar()), 128);
            assert_eq!(unwrap_stdio(getchar()), 255);
            assert_eq!(unwrap_stdio(getchar()), -1);
            assert_eq!(unwrap_stdio(getchar()), -1);
        }
        Ok("putchar") => {
            assert_eq!(unwrap_stdio(putchar(-1)), 255);
            assert_eq!(unwrap_stdio(putchar(256)), 0);
            assert_eq!(unwrap_stdio(putchar(65)), 65);
        }
        Ok("puts") => {
            assert_eq!(unwrap_stdio(puts(&[1, 2, 0, 3])), 0);
            assert_eq!(unwrap_stdio(puts(&[0])), 0);
            assert_eq!(unwrap_stdio(puts(&[-1, -128, 0])), 0);
        }
        Ok(mode) => panic!("unknown standard-stream child mode: {mode}"),
        Err(error) => panic!("invalid standard-stream child mode: {error}"),
    }
}

fn standard_stream_command(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "stdio::tests::standard_stream_child",
            "--nocapture",
        ])
        .env(STANDARD_STREAM_CHILD, mode);
    command
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn getchar_reads_unsigned_bytes_and_reports_end_of_file() {
    let mut child = standard_stream_command("getchar")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(&[0, 127, 128, 255])
        .unwrap();

    assert!(child.wait().unwrap().success());
}

#[test]
fn putchar_writes_and_returns_the_input_converted_to_an_unsigned_byte() {
    let output = standard_stream_command("putchar").output().unwrap();

    assert!(output.status.success());
    assert!(contains_subslice(&output.stdout, &[255, 0, 65]));
}

#[test]
fn puts_stops_at_null_and_appends_a_newline() {
    let output = standard_stream_command("puts").output().unwrap();

    assert!(output.status.success());
    assert!(contains_subslice(
        &output.stdout,
        &[1, 2, b'\n', b'\n', 255, 128, b'\n']
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn remove_deletes_a_regular_file_and_ignores_bytes_after_null() {
    let temp = StdioTestDir::new();
    let file = temp.join("file");
    fs::write(&file, b"contents").unwrap();
    let mut path = null_terminated_path(&file);
    path.extend([b'x' as i8, b'y' as i8]);

    assert_eq!(unwrap_stdio(remove(&path)), 0);
    assert_path_missing(&file);
}

#[cfg(target_os = "linux")]
#[test]
fn remove_deletes_an_empty_directory_with_a_trailing_slash() {
    let temp = StdioTestDir::new();
    let directory = temp.join("empty");
    fs::create_dir(&directory).unwrap();
    let mut path = null_terminated_path(&directory);
    path.pop();
    path.extend([b'/' as i8, 0]);

    assert_eq!(unwrap_stdio(remove(&path)), 0);
    assert_path_missing(&directory);
}

#[cfg(target_os = "linux")]
#[test]
fn remove_rejects_a_nonempty_directory_without_changing_it() {
    let temp = StdioTestDir::new();
    let directory = temp.join("nonempty");
    let child = directory.join("child");
    fs::create_dir(&directory).unwrap();
    fs::write(&child, b"contents").unwrap();

    let (value, status) = remove(&null_terminated_path(&directory));

    assert_eq!(value, -1);
    assert_eq!(status.unwrap_err().kind(), io::ErrorKind::DirectoryNotEmpty);
    assert!(directory.is_dir());
    assert_eq!(fs::read(child).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn remove_unlinks_a_symbolic_link_to_a_file_without_changing_the_target() {
    let temp = StdioTestDir::new();
    let target = temp.join("target-file");
    let link = temp.join("file-link");
    fs::write(&target, b"contents").unwrap();
    symlink(&target, &link).unwrap();

    assert_eq!(unwrap_stdio(remove(&null_terminated_path(&link))), 0);

    assert_path_missing(&link);
    assert_eq!(fs::read(target).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn remove_unlinks_a_symbolic_link_to_a_directory_without_changing_the_target() {
    let temp = StdioTestDir::new();
    let target = temp.join("target-directory");
    let child = target.join("child");
    let link = temp.join("directory-link");
    fs::create_dir(&target).unwrap();
    fs::write(&child, b"contents").unwrap();
    symlink(&target, &link).unwrap();

    assert_eq!(unwrap_stdio(remove(&null_terminated_path(&link))), 0);

    assert_path_missing(&link);
    assert!(target.is_dir());
    assert_eq!(fs::read(child).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn remove_unlinks_a_broken_symbolic_link() {
    let temp = StdioTestDir::new();
    let target = temp.join("missing-target");
    let link = temp.join("broken-link");
    symlink(&target, &link).unwrap();

    assert_eq!(unwrap_stdio(remove(&null_terminated_path(&link))), 0);

    assert_path_missing(&link);
    assert_path_missing(&target);
}

#[cfg(target_os = "linux")]
#[test]
fn remove_decrements_a_files_link_count_without_changing_its_other_link() {
    let temp = StdioTestDir::new();
    let file = temp.join("file");
    let other_link = temp.join("other-link");
    fs::write(&file, b"contents").unwrap();
    fs::hard_link(&file, &other_link).unwrap();

    assert_eq!(unwrap_stdio(remove(&null_terminated_path(&file))), 0);

    assert_path_missing(&file);
    assert_eq!(fs::read(other_link).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn remove_rejects_a_trailing_slash_on_a_regular_file_without_changing_it() {
    let temp = StdioTestDir::new();
    let file = temp.join("file");
    fs::write(&file, b"contents").unwrap();
    let mut path = null_terminated_path(&file);
    path.pop();
    path.extend([b'/' as i8, 0]);

    let (value, status) = remove(&path);

    assert_eq!(value, -1);
    assert_eq!(status.unwrap_err().kind(), io::ErrorKind::NotADirectory);
    assert_eq!(fs::read(file).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn remove_accepts_a_non_utf8_path() {
    let temp = StdioTestDir::new();
    let file = temp.join(OsString::from_vec(b"non-utf8-\xff".to_vec()));
    fs::write(&file, b"contents").unwrap();

    assert_eq!(unwrap_stdio(remove(&null_terminated_path(&file))), 0);
    assert_path_missing(&file);
}

#[cfg(target_os = "linux")]
#[test]
fn remove_reports_a_missing_path_without_changing_its_parent() {
    let temp = StdioTestDir::new();
    let missing = temp.join("missing");

    let (value, status) = remove(&null_terminated_path(&missing));

    assert_eq!(value, -1);
    assert_eq!(status.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert!(temp.0.is_dir());
    assert_path_missing(&missing);
}

#[cfg(target_os = "linux")]
#[test]
fn remove_reports_an_empty_path() {
    let (value, status) = remove(&[0]);

    assert_eq!(value, -1);
    assert_eq!(status.unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[cfg(target_os = "linux")]
#[test]
fn remove_unlinks_an_open_file_without_invalidating_the_open_handle() {
    let temp = StdioTestDir::new();
    let file = temp.join("open-file");
    fs::write(&file, b"contents").unwrap();
    let mut open_file = fs::File::open(&file).unwrap();

    assert_eq!(unwrap_stdio(remove(&null_terminated_path(&file))), 0);
    assert_path_missing(&file);

    let mut contents = String::new();
    open_file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_moves_a_regular_file_between_directories_and_ignores_bytes_after_each_null() {
    let temp = StdioTestDir::new();
    let old_parent = temp.join("old-parent");
    let new_parent = temp.join("new-parent");
    fs::create_dir(&old_parent).unwrap();
    fs::create_dir(&new_parent).unwrap();
    let old = old_parent.join("old");
    let new = new_parent.join("new");
    fs::write(&old, b"contents").unwrap();
    let mut old_path = null_terminated_path(&old);
    let mut new_path = null_terminated_path(&new);
    old_path.extend([b'x' as i8, b'y' as i8]);
    new_path.extend([b'a' as i8, b'b' as i8]);

    assert_eq!(unwrap_stdio(rename(&old_path, &new_path)), 0);

    assert_path_missing(&old);
    assert_eq!(fs::read(new).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_moves_a_populated_directory() {
    let temp = StdioTestDir::new();
    let old = temp.join("old-directory");
    let new = temp.join("new-directory");
    fs::create_dir(&old).unwrap();
    fs::write(old.join("child"), b"contents").unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert_eq!(fs::read(new.join("child")).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_replaces_an_existing_regular_file() {
    let temp = StdioTestDir::new();
    let old = temp.join("old");
    let new = temp.join("new");
    fs::write(&old, b"old contents").unwrap();
    fs::write(&new, b"new contents").unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert_eq!(fs::read(new).unwrap(), b"old contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_replaces_an_empty_directory() {
    let temp = StdioTestDir::new();
    let old = temp.join("old-directory");
    let new = temp.join("new-directory");
    fs::create_dir(&old).unwrap();
    fs::write(old.join("child"), b"contents").unwrap();
    fs::create_dir(&new).unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert_eq!(fs::read(new.join("child")).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_rejects_a_nonempty_destination_directory_without_changing_either_directory() {
    let temp = StdioTestDir::new();
    let old = temp.join("old-directory");
    let new = temp.join("new-directory");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&new).unwrap();
    fs::write(old.join("old-child"), b"old contents").unwrap();
    fs::write(new.join("new-child"), b"new contents").unwrap();

    let (value, status) = rename(&null_terminated_path(&old), &null_terminated_path(&new));

    assert_eq!(value, -1);
    assert_eq!(status.unwrap_err().kind(), io::ErrorKind::DirectoryNotEmpty);
    assert_eq!(fs::read(old.join("old-child")).unwrap(), b"old contents");
    assert_eq!(fs::read(new.join("new-child")).unwrap(), b"new contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_rejects_file_directory_type_mismatches_without_changing_either_path() {
    let temp = StdioTestDir::new();
    let old_file = temp.join("old-file");
    let new_directory = temp.join("new-directory");
    let old_directory = temp.join("old-directory");
    let new_file = temp.join("new-file");
    fs::write(&old_file, b"old file").unwrap();
    fs::create_dir(&new_directory).unwrap();
    fs::create_dir(&old_directory).unwrap();
    fs::write(old_directory.join("child"), b"old directory").unwrap();
    fs::write(&new_file, b"new file").unwrap();

    let (file_value, file_status) = rename(
        &null_terminated_path(&old_file),
        &null_terminated_path(&new_directory),
    );
    let (directory_value, directory_status) = rename(
        &null_terminated_path(&old_directory),
        &null_terminated_path(&new_file),
    );

    assert_eq!(file_value, -1);
    assert_eq!(file_status.unwrap_err().kind(), io::ErrorKind::IsADirectory);
    assert_eq!(directory_value, -1);
    assert_eq!(
        directory_status.unwrap_err().kind(),
        io::ErrorKind::NotADirectory
    );
    assert_eq!(fs::read(old_file).unwrap(), b"old file");
    assert!(new_directory.is_dir());
    assert_eq!(
        fs::read(old_directory.join("child")).unwrap(),
        b"old directory"
    );
    assert_eq!(fs::read(new_file).unwrap(), b"new file");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_moves_a_broken_symbolic_link_without_resolving_it() {
    let temp = StdioTestDir::new();
    let target = temp.join("missing-target");
    let old = temp.join("old-link");
    let new = temp.join("new-link");
    symlink(&target, &old).unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert_eq!(fs::read_link(&new).unwrap(), target);
    assert_path_missing(&target);
}

#[cfg(target_os = "linux")]
#[test]
fn rename_replaces_a_destination_symlink_without_changing_its_target() {
    let temp = StdioTestDir::new();
    let old = temp.join("old-file");
    let target = temp.join("target-directory");
    let new = temp.join("new-link");
    fs::write(&old, b"contents").unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(target.join("child"), b"target contents").unwrap();
    symlink(&target, &new).unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert!(!fs::symlink_metadata(&new).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(new).unwrap(), b"contents");
    assert_eq!(fs::read(target.join("child")).unwrap(), b"target contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_succeeds_without_changing_a_path_renamed_to_itself() {
    let temp = StdioTestDir::new();
    let file = temp.join("file");
    fs::write(&file, b"contents").unwrap();
    let path = null_terminated_path(&file);

    assert_eq!(unwrap_stdio(rename(&path, &path)), 0);
    assert_eq!(fs::read(file).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_succeeds_without_removing_different_hard_links_to_the_same_file() {
    let temp = StdioTestDir::new();
    let old = temp.join("old-link");
    let new = temp.join("new-link");
    fs::write(&old, b"contents").unwrap();
    fs::hard_link(&old, &new).unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_eq!(fs::read(old).unwrap(), b"contents");
    assert_eq!(fs::read(new).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_does_not_invalidate_an_open_handle_to_the_replaced_file() {
    let temp = StdioTestDir::new();
    let old = temp.join("old");
    let new = temp.join("new");
    fs::write(&old, b"old contents").unwrap();
    fs::write(&new, b"new contents").unwrap();
    let mut replaced_file = fs::File::open(&new).unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert_eq!(fs::read(&new).unwrap(), b"old contents");
    let mut contents = String::new();
    replaced_file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "new contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_accepts_non_utf8_paths() {
    let temp = StdioTestDir::new();
    let old = temp.join(OsString::from_vec(b"old-\xff".to_vec()));
    let new = temp.join(OsString::from_vec(b"new-\xfe".to_vec()));
    fs::write(&old, b"contents").unwrap();

    assert_eq!(
        unwrap_stdio(rename(
            &null_terminated_path(&old),
            &null_terminated_path(&new)
        )),
        0
    );

    assert_path_missing(&old);
    assert_eq!(fs::read(new).unwrap(), b"contents");
}

#[cfg(target_os = "linux")]
#[test]
fn rename_reports_empty_paths_without_changing_the_existing_file() {
    let temp = StdioTestDir::new();
    let file = temp.join("file");
    let missing = temp.join("missing");
    fs::write(&file, b"contents").unwrap();

    let (old_value, old_status) = rename(&[0], &null_terminated_path(&missing));
    let (new_value, new_status) = rename(&null_terminated_path(&file), &[0]);

    assert_eq!(old_value, -1);
    assert_eq!(old_status.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(new_value, -1);
    assert_eq!(new_status.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(fs::read(file).unwrap(), b"contents");
    assert_path_missing(&missing);
}

#[cfg(target_os = "linux")]
#[test]
fn rename_reports_missing_paths_without_changing_existing_entries() {
    let temp = StdioTestDir::new();
    let missing_old = temp.join("missing-old");
    let existing_new = temp.join("existing-new");
    let existing_old = temp.join("existing-old");
    let missing_parent = temp.join("missing-parent");
    let missing_new = missing_parent.join("new");
    fs::write(&existing_new, b"new contents").unwrap();
    fs::write(&existing_old, b"old contents").unwrap();

    let (old_value, old_status) = rename(
        &null_terminated_path(&missing_old),
        &null_terminated_path(&existing_new),
    );
    let (new_value, new_status) = rename(
        &null_terminated_path(&existing_old),
        &null_terminated_path(&missing_new),
    );

    assert_eq!(old_value, -1);
    assert_eq!(old_status.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(new_value, -1);
    assert_eq!(new_status.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_path_missing(&missing_old);
    assert_eq!(fs::read(existing_new).unwrap(), b"new contents");
    assert_eq!(fs::read(existing_old).unwrap(), b"old contents");
    assert_path_missing(&missing_parent);
}

#[cfg(target_os = "linux")]
#[test]
fn rename_rejects_trailing_slashes_on_regular_file_paths_without_changing_them() {
    let temp = StdioTestDir::new();
    let first_old = temp.join("first-old");
    let first_new = temp.join("first-new");
    let second_old = temp.join("second-old");
    let second_new = temp.join("second-new");
    fs::write(&first_old, b"first contents").unwrap();
    fs::write(&second_old, b"second contents").unwrap();
    let mut first_old_path = null_terminated_path(&first_old);
    first_old_path.pop();
    first_old_path.extend([b'/' as i8, 0]);
    let mut second_new_path = null_terminated_path(&second_new);
    second_new_path.pop();
    second_new_path.extend([b'/' as i8, 0]);

    let (old_value, old_status) = rename(&first_old_path, &null_terminated_path(&first_new));
    let (new_value, new_status) = rename(&null_terminated_path(&second_old), &second_new_path);

    assert_eq!(old_value, -1);
    assert_eq!(old_status.unwrap_err().kind(), io::ErrorKind::NotADirectory);
    assert_eq!(new_value, -1);
    assert_eq!(new_status.unwrap_err().kind(), io::ErrorKind::NotADirectory);
    assert_eq!(fs::read(first_old).unwrap(), b"first contents");
    assert_path_missing(&first_new);
    assert_eq!(fs::read(second_old).unwrap(), b"second contents");
    assert_path_missing(&second_new);
}

#[cfg(target_os = "linux")]
#[test]
fn rename_rejects_moving_a_directory_beneath_itself_without_changing_it() {
    let temp = StdioTestDir::new();
    let old = temp.join("directory");
    let child = old.join("child");
    let new = child.join("moved");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&child).unwrap();
    fs::write(child.join("file"), b"contents").unwrap();

    let (value, status) = rename(&null_terminated_path(&old), &null_terminated_path(&new));

    assert_eq!(value, -1);
    assert_eq!(status.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    assert_eq!(fs::read(child.join("file")).unwrap(), b"contents");
    assert_path_missing(&new);
}

#[test]
fn fseek_repositions_from_the_start_current_position_and_end() {
    let mut stream = Cursor::new([0; 8]);

    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::Start(3))), 0);
    assert_eq!(stream.position(), 3);
    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::Current(2))), 0);
    assert_eq!(stream.position(), 5);
    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::End(-1))), 0);
    assert_eq!(stream.position(), 7);
}

#[test]
fn fseek_allows_positions_beyond_the_end() {
    let mut stream = Cursor::new([0; 8]);

    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::Start(12))), 0);
    assert_eq!(unwrap_stdio(ftell(&mut stream)), 12);
}

#[test]
fn ftell_returns_the_current_position_without_changing_it() {
    let mut stream = Cursor::new([0; 8]);
    stream.set_position(5);

    assert_eq!(unwrap_stdio(ftell(&mut stream)), 5);
    assert_eq!(stream.position(), 5);
}

#[test]
fn rewind_sets_the_position_to_the_beginning() {
    let mut stream = Cursor::new([0; 8]);
    stream.set_position(5);

    rewind(&mut stream).unwrap();

    assert_eq!(stream.position(), 0);
}

#[test]
fn seek_functions_accept_dynamically_sized_streams() {
    let mut bytes = Cursor::new([0; 8]);
    let stream: &mut dyn Seek = &mut bytes;

    assert_eq!(unwrap_stdio(fseek(stream, SeekFrom::Start(6))), 0);
    assert_eq!(unwrap_stdio(ftell(stream)), 6);
    rewind(stream).unwrap();
    assert_eq!(unwrap_stdio(ftell(stream)), 0);
}

#[test]
fn seek_functions_propagate_seek_errors() {
    struct FailingSeek;

    impl Seek for FailingSeek {
        fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
            Err(io::Error::other("seek failed"))
        }
    }

    let (fseek_value, fseek_status) = fseek(&mut FailingSeek, SeekFrom::Start(1));
    let (ftell_value, ftell_status) = ftell(&mut FailingSeek);

    assert_eq!(fseek_value, -1);
    assert_eq!(ftell_value, -1);

    for error in [
        fseek_status.unwrap_err(),
        ftell_status.unwrap_err(),
        rewind(&mut FailingSeek).unwrap_err(),
    ] {
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(error.to_string(), "seek failed");
    }
}

#[test]
fn fseek_and_ftell_accept_i64_max_and_reject_larger_positions() {
    struct PositionSeek(u64);

    impl Seek for PositionSeek {
        fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
            Ok(self.0)
        }
    }

    let mut maximum = PositionSeek(i64::MAX as u64);
    assert_eq!(unwrap_stdio(fseek(&mut maximum, SeekFrom::Start(0))), 0);
    assert_eq!(unwrap_stdio(ftell(&mut maximum)), i64::MAX);

    let mut too_large = PositionSeek(i64::MAX as u64 + 1);
    let (fseek_value, fseek_status) = fseek(&mut too_large, SeekFrom::Start(0));
    let (ftell_value, ftell_status) = ftell(&mut too_large);

    assert_eq!(fseek_value, -1);
    assert_eq!(ftell_value, -1);

    assert_eq!(fseek_status.unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(ftell_status.unwrap_err().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn reads_bytes_as_unsigned_integers_and_advances_the_reader() {
    let mut reader = Cursor::new([0, 127, 128, 255]);

    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 0);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 127);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 128);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 255);
}

#[test]
fn returns_minus_one_at_end_of_file() {
    let mut reader = Cursor::new([42]);

    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 42);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), -1);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), -1);
}

#[test]
fn accepts_dynamically_sized_readers() {
    let mut bytes = Cursor::new([42]);
    let reader: &mut dyn Read = &mut bytes;

    assert_eq!(unwrap_stdio(fgetc(reader)), 42);
}

#[test]
fn propagates_read_errors() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    let (value, status) = fgetc(&mut FailingReader);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
}

#[test]
fn fputc_writes_bytes_and_returns_their_unsigned_values() {
    let mut writer = Vec::new();

    assert_eq!(unwrap_stdio(fputc(0, &mut writer)), 0);
    assert_eq!(unwrap_stdio(fputc(127, &mut writer)), 127);
    assert_eq!(unwrap_stdio(fputc(128, &mut writer)), 128);
    assert_eq!(unwrap_stdio(fputc(255, &mut writer)), 255);
    assert_eq!(writer, [0, 127, 128, 255]);
}

#[test]
fn fputc_converts_the_input_to_an_unsigned_byte() {
    let mut writer = Vec::new();

    assert_eq!(unwrap_stdio(fputc(-1, &mut writer)), 255);
    assert_eq!(unwrap_stdio(fputc(256, &mut writer)), 0);
    assert_eq!(unwrap_stdio(fputc(511, &mut writer)), 255);
    assert_eq!(writer, [255, 0, 255]);
}

#[test]
fn fputc_accepts_dynamically_sized_writers() {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;

    assert_eq!(unwrap_stdio(fputc(42, writer)), 42);
    assert_eq!(bytes, [42]);
}

#[test]
fn fputc_reports_a_writer_that_makes_no_progress() {
    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (value, status) = fputc(42, &mut ZeroWriter);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

#[test]
fn fputc_propagates_write_errors() {
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (value, status) = fputc(42, &mut FailingWriter);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "write failed");
}

#[test]
fn fputc_does_not_retry_an_interrupted_write() {
    struct InterruptedWriter {
        calls: usize,
    }

    impl Write for InterruptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Err(io::ErrorKind::Interrupted.into())
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = InterruptedWriter { calls: 0 };
    let (value, status) = fputc(42, &mut writer);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(writer.calls, 1);
}

#[test]
fn fputs_writes_bytes_before_the_first_null_and_returns_zero() {
    let mut writer = Vec::new();

    assert_eq!(
        unwrap_stdio(fputs(&[b'a' as i8, -128, -1, 0, b'b' as i8], &mut writer)),
        0
    );
    assert_eq!(writer, [b'a', 128, 255]);
}

#[test]
fn fputs_does_not_write_an_empty_string() {
    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!("write called for an empty string")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    assert_eq!(
        unwrap_stdio(fputs(&[0, b'a' as i8], &mut PanickingWriter)),
        0
    );
}

#[test]
fn fputs_completes_partial_writes_without_flushing() {
    struct PartialWriter {
        bytes: Vec<u8>,
        calls: usize,
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            let len = buf.len().min(2);
            self.bytes.extend_from_slice(&buf[..len]);
            Ok(len)
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let mut writer = PartialWriter {
        bytes: Vec::new(),
        calls: 0,
    };

    assert_eq!(unwrap_stdio(fputs(&[1, 2, 3, 4, 5, 0], &mut writer)), 0);
    assert_eq!(writer.bytes, [1, 2, 3, 4, 5]);
    assert_eq!(writer.calls, 3);
}

#[test]
fn fputs_reports_a_writer_that_makes_no_progress() {
    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (value, status) = fputs(&[1, 0], &mut ZeroWriter);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

#[test]
fn fputs_propagates_write_errors_after_partial_output() {
    struct FailingWriter {
        bytes: Vec<u8>,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&buf[..2]);
                Ok(2)
            } else {
                Err(io::Error::other("write failed"))
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = FailingWriter { bytes: Vec::new() };
    let (value, status) = fputs(&[1, 2, 3, 4, 0], &mut writer);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "write failed");
    assert_eq!(writer.bytes, [1, 2]);
}

#[test]
fn fputs_does_not_retry_an_interrupted_write() {
    struct InterruptedWriter {
        calls: usize,
    }

    impl Write for InterruptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Err(io::ErrorKind::Interrupted.into())
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = InterruptedWriter { calls: 0 };
    let (value, status) = fputs(&[1, 2, 0], &mut writer);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(writer.calls, 1);
}

#[test]
fn fputs_accepts_dynamically_sized_writers() {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;

    assert_eq!(unwrap_stdio(fputs(&[1, 2, 0], writer)), 0);
    assert_eq!(bytes, [1, 2]);
}

#[test]
fn fgets_reads_through_newline_and_returns_the_entire_buffer() {
    let mut reader = Cursor::new(b"first line\nsecond line");
    let mut buf = [99; 16];

    let result = unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap();

    assert_eq!(result.len(), 16);
    assert_eq!(
        result,
        &[
            102, 105, 114, 115, 116, 32, 108, 105, 110, 101, 10, 0, 99, 99, 99, 99
        ]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"second line");
}

#[test]
fn fgets_stops_when_the_buffer_is_full_without_overreading() {
    let mut reader = Cursor::new(b"abcdef");
    let mut first = [99; 4];
    let mut second = [99; 4];

    assert_eq!(
        unwrap_stdio(fgets(&mut first, &mut reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0]
    );
    assert_eq!(
        unwrap_stdio(fgets(&mut second, &mut reader)).unwrap(),
        &[b'd' as i8, b'e' as i8, b'f' as i8, 0]
    );
}

#[test]
fn fgets_leaves_a_newline_beyond_the_buffer_limit_for_the_next_call() {
    let mut reader = Cursor::new(b"abc\ndef");
    let mut first = [99; 4];
    let mut second = [99; 4];

    assert_eq!(
        unwrap_stdio(fgets(&mut first, &mut reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0]
    );
    assert_eq!(
        unwrap_stdio(fgets(&mut second, &mut reader)).unwrap(),
        &[b'\n' as i8, 0, 99, 99]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"def");
}

#[test]
fn fgets_reads_across_buffered_chunks() {
    let mut reader = BufReader::with_capacity(2, Cursor::new(b"abc\ndef"));
    let mut buf = [99; 8];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(),
        &[
            b'a' as i8,
            b'b' as i8,
            b'c' as i8,
            b'\n' as i8,
            0,
            99,
            99,
            99
        ]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"de");
}

#[test]
fn fgets_returns_none_at_eof_without_modifying_the_buffer() {
    let mut reader = Cursor::new([]);
    let mut buf = [99; 4];

    assert!(unwrap_stdio(fgets(&mut buf, &mut reader)).is_none());
    assert_eq!(buf, [99; 4]);
}

#[test]
fn fgets_returns_data_when_eof_follows_bytes() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [99; 5];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0, 99]
    );
}

#[test]
fn fgets_treats_null_and_high_bytes_as_input() {
    let mut reader = Cursor::new([0, 128, 255, b'\n', b'x']);
    let mut buf = [99; 6];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(),
        &[0, -128, -1, b'\n' as i8, 0, 99]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"x");
}

#[test]
fn fgets_with_one_byte_buffer_writes_only_null_without_reading() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [99];

    assert_eq!(unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(), &[0]);
    assert_eq!(reader.position(), 0);
}

#[test]
fn fgets_accepts_dynamically_sized_buffered_readers() {
    let mut bytes = Cursor::new(b"abc\n");
    let reader: &mut dyn BufRead = &mut bytes;
    let mut buf = [99; 5];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, b'\n' as i8, 0]
    );
}

#[test]
fn fgets_propagates_buffered_read_errors() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("read failed"))
        }

        fn consume(&mut self, _amt: usize) {}
    }

    let mut buf = [99; 4];
    let (value, status) = fgets(&mut buf, &mut FailingReader);
    let error = status.unwrap_err();

    assert!(value.is_none());
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
}

#[test]
fn fgets_propagates_an_error_after_copying_available_bytes() {
    struct PartialThenFail {
        consumed: bool,
    }

    impl Read for PartialThenFail {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    impl BufRead for PartialThenFail {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            if self.consumed {
                Err(io::Error::other("read failed"))
            } else {
                Ok(b"ab")
            }
        }

        fn consume(&mut self, amt: usize) {
            assert_eq!(amt, 2);
            self.consumed = true;
        }
    }

    let mut reader = PartialThenFail { consumed: false };
    let mut buf = [99; 4];
    let (value, status) = fgets(&mut buf, &mut reader);
    let error = status.unwrap_err();

    assert!(value.is_none());
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
    assert_eq!(buf, [b'a' as i8, b'b' as i8, 99, 99]);
}

#[test]
fn fread_reads_complete_elements() {
    let mut reader = Cursor::new([1, 2, 3, 4]);
    let mut buf = [0_u16; 2];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(
        buf,
        [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])]
    );
}

#[test]
fn fread_does_not_read_past_the_destination() {
    let mut reader = Cursor::new(b"abcde");
    let mut buf = [0_u8; 3];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(buf, *b"abc");
    assert_eq!(reader.fill_buf().unwrap(), b"de");
}

#[test]
fn fread_returns_only_complete_elements_at_end_of_file() {
    let mut reader = Cursor::new([1, 2, 3]);
    let mut buf = [u16::from_ne_bytes([9, 9]); 2];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 1);
    status.unwrap();
    assert_eq!(bytemuck::cast_slice::<_, u8>(&buf), &[1, 2, 3, 9]);
}

#[test]
fn fread_reads_elements_across_buffered_chunks() {
    let mut reader = BufReader::with_capacity(3, Cursor::new([1, 2, 3, 4, 5, 6, 7, 8]));
    let mut buf = [0_u32; 2];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(
        buf,
        [
            u32::from_ne_bytes([1, 2, 3, 4]),
            u32::from_ne_bytes([5, 6, 7, 8])
        ]
    );
}

#[test]
fn fread_does_not_read_for_an_empty_buffer() {
    let mut reader = Cursor::new(b"abc");

    let (count, status) = fread::<u8, _>(&mut [], &mut reader);

    assert_eq!(count, 0);
    status.unwrap();
    assert_eq!(reader.position(), 0);
}

#[test]
fn fread_does_not_read_zero_sized_elements() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [(); 3];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 0);
    status.unwrap();
    assert_eq!(reader.position(), 0);
}

#[test]
fn fread_accepts_dynamically_sized_buffered_readers() {
    let mut bytes = Cursor::new(b"abc");
    let reader: &mut dyn BufRead = &mut bytes;
    let mut buf = [0_u8; 3];

    let (count, status) = fread(&mut buf, reader);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(buf, *b"abc");
}

#[test]
fn fread_propagates_an_error_before_reading_any_bytes() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("read failed"))
        }

        fn consume(&mut self, _amt: usize) {}
    }

    let mut buf = [9_u8; 2];

    let (count, status) = fread(&mut buf, &mut FailingReader);
    let error = status.unwrap_err();

    assert_eq!(count, 0);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
    assert_eq!(buf, [9; 2]);
}

#[test]
fn fread_returns_an_error_without_discarding_the_element_count() {
    struct PartialThenFail {
        consumed: bool,
    }

    impl Read for PartialThenFail {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    impl BufRead for PartialThenFail {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            if self.consumed {
                Err(io::Error::other("read failed"))
            } else {
                Ok(b"abc")
            }
        }

        fn consume(&mut self, amt: usize) {
            assert_eq!(amt, 3);
            self.consumed = true;
        }
    }

    let mut reader = PartialThenFail { consumed: false };
    let mut buf = [u16::from_ne_bytes([9, 9]); 2];

    let (count, status) = fread(&mut buf, &mut reader);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
    assert_eq!(bytemuck::cast_slice::<_, u8>(&buf), &[b'a', b'b', b'c', 9]);
}

#[test]
fn fread_accepts_any_bit_pattern_types_with_padding() {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Padded {
        byte: u8,
        number: u16,
    }

    // SAFETY: All bit patterns, including zero, are valid for both fields, and
    // the type contains no pointers or interior mutability.
    unsafe impl bytemuck::Zeroable for Padded {}
    unsafe impl bytemuck::AnyBitPattern for Padded {}

    let mut reader = Cursor::new([1, 2, 3, 4]);
    let mut buf = [Padded { byte: 0, number: 0 }];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 1);
    status.unwrap();
    assert_eq!(buf[0].byte, 1);
    assert_eq!(buf[0].number, u16::from_ne_bytes([3, 4]));
}

#[test]
fn fwrite_writes_complete_elements() {
    let buf = [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])];
    let mut writer = Vec::new();

    let (count, status) = fwrite(&buf, &mut writer);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(writer, [1, 2, 3, 4]);
}

#[test]
fn fwrite_accepts_no_uninit_types_that_are_not_pod() {
    let mut writer = Vec::new();

    let (count, status) = fwrite(&[true, false], &mut writer);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(writer, [1, 0]);
}

#[test]
fn fwrite_does_not_write_an_empty_buffer() {
    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!("write called for an empty buffer")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let (count, status) = fwrite::<u8, _>(&[], &mut PanickingWriter);

    assert_eq!(count, 0);
    status.unwrap();
}

#[test]
fn fwrite_does_not_write_zero_sized_elements() {
    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!("write called for zero-sized elements")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let (count, status) = fwrite(&[(); 3], &mut PanickingWriter);

    assert_eq!(count, 0);
    status.unwrap();
}

#[test]
fn fwrite_completes_partial_writes_without_flushing() {
    struct PartialWriter {
        bytes: Vec<u8>,
        offered_lengths: Vec<usize>,
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.offered_lengths.push(buf.len());
            let len = buf.len().min(3);
            self.bytes.extend_from_slice(&buf[..len]);
            Ok(len)
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let buf = [
        u16::from_ne_bytes([1, 2]),
        u16::from_ne_bytes([3, 4]),
        u16::from_ne_bytes([5, 6]),
    ];
    let mut writer = PartialWriter {
        bytes: Vec::new(),
        offered_lengths: Vec::new(),
    };

    let (count, status) = fwrite(&buf, &mut writer);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(writer.bytes, [1, 2, 3, 4, 5, 6]);
    assert_eq!(writer.offered_lengths, [6, 3]);
}

#[test]
fn fwrite_reports_a_writer_that_makes_no_progress() {
    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (count, status) = fwrite(&[1_u8], &mut ZeroWriter);
    let error = status.unwrap_err();

    assert_eq!(count, 0);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

#[test]
fn fwrite_returns_only_complete_elements_when_writing_stalls() {
    struct PartialThenZero {
        bytes: Vec<u8>,
    }

    impl Write for PartialThenZero {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&buf[..3]);
                Ok(3)
            } else {
                Ok(0)
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let buf = [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])];
    let mut writer = PartialThenZero { bytes: Vec::new() };

    let (count, status) = fwrite(&buf, &mut writer);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
    assert_eq!(writer.bytes, [1, 2, 3]);
}

#[test]
fn fwrite_returns_an_error_without_discarding_the_element_count() {
    struct PartialThenFail {
        bytes: Vec<u8>,
    }

    impl Write for PartialThenFail {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&buf[..3]);
                Ok(3)
            } else {
                Err(io::Error::other("write failed"))
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let buf = [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])];
    let mut writer = PartialThenFail { bytes: Vec::new() };

    let (count, status) = fwrite(&buf, &mut writer);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "write failed");
    assert_eq!(writer.bytes, [1, 2, 3]);
}

#[test]
fn fwrite_does_not_retry_an_interrupted_write() {
    struct InterruptedWriter {
        calls: usize,
    }

    impl Write for InterruptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Ok(2)
            } else if self.calls == 2 {
                Err(io::ErrorKind::Interrupted.into())
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = InterruptedWriter { calls: 0 };

    let (count, status) = fwrite(&[1_u16, 2], &mut writer);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(writer.calls, 2);
}

#[test]
fn fwrite_accepts_dynamically_sized_writers() {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;

    let (count, status) = fwrite(&[1_u8, 2, 3], writer);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(bytes, [1, 2, 3]);
}
