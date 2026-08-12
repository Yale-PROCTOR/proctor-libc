use super::{
    StrtoFloatError, StrtoIntError, atof, atoi, atol, strtod, strtof, strtol, strtold, strtoul,
};

fn i8s(bytes: &[u8]) -> &[i8] {
    bytemuck::cast_slice(bytes)
}

fn f128_bits(value: f128::f128) -> u128 {
    u128::from_ne_bytes(value.inner())
}

#[test]
fn atof_converts_a_floating_point_prefix() {
    assert_eq!(atof(i8s(b" \t\n\x0b\x0c\r-12.5e+1rest\0ignored")), -125.0);
    assert_eq!(atof(i8s(b"0x1.8p+2!\0")), 6.0);
}

#[test]
fn atof_returns_zero_when_no_conversion_is_possible() {
    assert_eq!(atof(i8s(b"not a number\0")), 0.0);
    assert_eq!(atof(i8s(b"\0")), 0.0);
}

#[test]
fn atof_preserves_special_values_and_signed_zero() {
    assert_eq!(atof(i8s(b"-INFINITY\0ignored")), f64::NEG_INFINITY);
    assert!(atof(i8s(b"nan(payload)rest\0")).is_nan());
    assert_eq!(atof(i8s(b"-0.0\0")).to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn strtod_parses_decimal_and_returns_the_suffix() {
    assert_eq!(
        strtod(i8s(b" \t\n\x0b\x0c\r-12.5e+1rest\0ignored")),
        ((-125.0, i8s(b"rest\0ignored")), Ok(()))
    );
    assert_eq!(strtod(i8s(b".25!\0")), ((0.25, i8s(b"!\0")), Ok(())));
    assert_eq!(strtod(i8s(b"2.\0")), ((2.0, i8s(b"\0")), Ok(())));
    assert_eq!(strtod(i8s(b"1000\0")), ((1000.0, i8s(b"\0")), Ok(())));
    assert_eq!(strtod(i8s(b"1200.00\0")), ((1200.0, i8s(b"\0")), Ok(())));
    assert_eq!(strtod(i8s(b"1.00e2\0")), ((100.0, i8s(b"\0")), Ok(())));
}

#[test]
fn floating_conversions_return_the_original_slice_if_there_is_no_subject() {
    for buf in [
        i8s(b"   \0"),
        i8s(b"-\0"),
        i8s(b".\0"),
        i8s(b"+.e2\0"),
        i8s(b"\0"),
    ] {
        let ((value, suffix), status) = strtod(buf);
        assert_eq!(value.to_bits(), 0);
        assert_eq!(suffix.as_ptr(), buf.as_ptr());
        assert_eq!(suffix.len(), buf.len());
        assert_eq!(status, Ok(()));
    }
}

#[test]
fn floating_conversions_only_consume_complete_exponents() {
    assert_eq!(strtod(i8s(b"1e\0")), ((1.0, i8s(b"e\0")), Ok(())));
    assert_eq!(strtod(i8s(b"1e+z\0")), ((1.0, i8s(b"e+z\0")), Ok(())));
    assert_eq!(strtod(i8s(b"0x1p-!\0")), ((1.0, i8s(b"p-!\0")), Ok(())));
    assert_eq!(strtod(i8s(b"0xg\0")), ((0.0, i8s(b"xg\0")), Ok(())));
}

#[test]
fn strtod_parses_hexadecimal_floats_and_rounds_ties_to_even() {
    assert_eq!(strtod(i8s(b"0x1.8p+2x\0")), ((6.0, i8s(b"x\0")), Ok(())));
    assert_eq!(strtod(i8s(b"0X10\0")), ((16.0, i8s(b"\0")), Ok(())));
    assert_eq!(strtod(i8s(b"0x1000\0")), ((4096.0, i8s(b"\0")), Ok(())));

    let ((even_down, _), _) = strtod(i8s(b"0x1.00000000000008p0\0"));
    let ((above_half, _), _) = strtod(i8s(b"0x1.000000000000081p0\0"));
    let ((even_up, _), _) = strtod(i8s(b"0x1.00000000000018p0\0"));
    assert_eq!(even_down.to_bits(), 1.0_f64.to_bits());
    assert_eq!(above_half.to_bits(), 1.0_f64.to_bits() + 1);
    assert_eq!(even_up.to_bits(), 1.0_f64.to_bits() + 2);
}

#[test]
fn strtod_rounds_decimal_ties_to_even() {
    let ((even_down, _), _) = strtod(i8s(
        b"1.00000000000000011102230246251565404236316680908203125\0",
    ));
    let ((even_up, _), _) = strtod(i8s(
        b"1.00000000000000033306690738754696212708950042724609375\0",
    ));
    assert_eq!(even_down.to_bits(), 1.0_f64.to_bits());
    assert_eq!(even_up.to_bits(), 1.0_f64.to_bits() + 2);
}

#[test]
fn strtof_rounds_directly_to_f32() {
    let ((even_down, _), _) = strtof(i8s(b"0x1.000001p0\0"));
    let ((above_half, _), _) = strtof(i8s(b"0x1.0000011p0\0"));
    let ((even_up, _), _) = strtof(i8s(b"0x1.000003p0\0"));
    assert_eq!(even_down.to_bits(), 1.0_f32.to_bits());
    assert_eq!(above_half.to_bits(), 1.0_f32.to_bits() + 1);
    assert_eq!(even_up.to_bits(), 1.0_f32.to_bits() + 2);

    assert_eq!(
        strtof(i8s(b"16777217\0")),
        ((16_777_216.0, i8s(b"\0")), Ok(()))
    );
}

#[test]
fn floating_conversions_handle_infinity_and_nan_subjects() {
    assert_eq!(
        strtod(i8s(b"-InFiNiTy!\0")),
        ((f64::NEG_INFINITY, i8s(b"!\0")), Ok(()))
    );
    assert_eq!(
        strtod(i8s(b"infinite\0")),
        ((f64::INFINITY, i8s(b"inite\0")), Ok(()))
    );

    let ((nan, suffix), status) = strtod(i8s(b"-NAN(payload_1)rest\0"));
    assert!(nan.is_nan());
    assert!(nan.is_sign_negative());
    assert_eq!(suffix, i8s(b"rest\0"));
    assert_eq!(status, Ok(()));

    let ((nan, suffix), status) = strtod(i8s(b"nan(bad-payload)\0"));
    assert!(nan.is_nan());
    assert_eq!(suffix, i8s(b"(bad-payload)\0"));
    assert_eq!(status, Ok(()));
}

#[test]
fn floating_conversions_preserve_signed_zero() {
    let ((decimal, _), _) = strtod(i8s(b"-0.0\0"));
    let ((hexadecimal, _), _) = strtof(i8s(b"-0x0p100\0"));
    assert_eq!(decimal.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(hexadecimal.to_bits(), (-0.0_f32).to_bits());
}

#[test]
fn strtod_reports_overflow_and_inexact_underflow() {
    assert_eq!(
        strtod(i8s(b"0x1.fffffffffffffp1023\0")),
        ((f64::MAX, i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtod(i8s(b"0x1.fffffffffffff8p1023rest\0")),
        (
            (f64::INFINITY, i8s(b"rest\0")),
            Err(StrtoFloatError::OutOfRange)
        )
    );
    assert_eq!(
        strtod(i8s(b"1e999999\0")),
        (
            (f64::INFINITY, i8s(b"\0")),
            Err(StrtoFloatError::OutOfRange)
        )
    );

    assert_eq!(
        strtod(i8s(b"0x1p-1074\0")),
        ((f64::from_bits(1), i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtod(i8s(b"0x1p-1075\0")),
        ((0.0, i8s(b"\0")), Err(StrtoFloatError::OutOfRange))
    );
    assert_eq!(
        strtod(i8s(b"-0x1.8p-1075\0")),
        (
            (f64::from_bits((1_u64 << 63) | 1), i8s(b"\0")),
            Err(StrtoFloatError::OutOfRange)
        )
    );
    assert_eq!(
        strtod(i8s(b"0x0.fffffffffffff8p-1022\0")),
        (
            (f64::MIN_POSITIVE, i8s(b"\0")),
            Err(StrtoFloatError::OutOfRange)
        )
    );
}

#[test]
fn strtof_uses_the_f32_range() {
    assert_eq!(
        strtof(i8s(b"0x1.fffffep127\0")),
        ((f32::MAX, i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtof(i8s(b"0x1.ffffffp127\0")),
        (
            (f32::INFINITY, i8s(b"\0")),
            Err(StrtoFloatError::OutOfRange)
        )
    );
    assert_eq!(
        strtof(i8s(b"0x1p-149\0")),
        ((f32::from_bits(1), i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtof(i8s(b"1e-1000\0")),
        ((0.0, i8s(b"\0")), Err(StrtoFloatError::OutOfRange))
    );
    assert_eq!(
        strtof(i8s(b"0x0.ffffffp-126\0")),
        (
            (f32::MIN_POSITIVE, i8s(b"\0")),
            Err(StrtoFloatError::OutOfRange)
        )
    );
}

#[test]
fn strtold_returns_ieee_binary128_values() {
    let ((value, suffix), status) = strtold(i8s(b"-0x1.8p1rest\0"));
    assert_eq!(
        f128_bits(value),
        (1_u128 << 127) | (0x4000_u128 << 112) | (1_u128 << 111)
    );
    assert_eq!(suffix, i8s(b"rest\0"));
    assert_eq!(status, Ok(()));

    let ((minimum_subnormal, _), status) = strtold(i8s(b"0x1p-16494\0"));
    assert_eq!(f128_bits(minimum_subnormal), 1);
    assert_eq!(status, Ok(()));

    let ((underflow, _), status) = strtold(i8s(b"0x1p-16495\0"));
    assert_eq!(f128_bits(underflow), 0);
    assert_eq!(status, Err(StrtoFloatError::OutOfRange));

    let ((overflow, _), status) = strtold(i8s(b"0x1.ffffffffffffffffffffffffffff8p16383\0"));
    assert_eq!(f128_bits(overflow), 0x7fff_u128 << 112);
    assert_eq!(status, Err(StrtoFloatError::OutOfRange));

    let ((rounded_to_minimum_normal, _), status) =
        strtold(i8s(b"0x0.ffffffffffffffffffffffffffff8p-16382\0"));
    assert_eq!(f128_bits(rounded_to_minimum_normal), 1_u128 << 112);
    assert_eq!(status, Err(StrtoFloatError::OutOfRange));
}

#[test]
fn strtol_parses_decimal_and_returns_the_suffix() {
    let buf = i8s(b" \t\n\x0b\x0c\r-42xyz\0ignored");

    assert_eq!(strtol(buf, 10), ((-42, i8s(b"xyz\0ignored")), Ok(())));
}

#[test]
fn atoi_converts_a_decimal_prefix() {
    assert_eq!(atoi(i8s(b" \t\n\x0b\x0c\r-42rest\0ignored")), -42);
    assert_eq!(atoi(i8s(b"+010!\0")), 10);
    assert_eq!(atoi(i8s(b"0x10\0")), 0);
}

#[test]
fn atoi_returns_zero_when_no_conversion_is_possible() {
    assert_eq!(atoi(i8s(b"not a number\0")), 0);
    assert_eq!(atoi(i8s(b"\0")), 0);
}

#[test]
fn atoi_accepts_exact_i32_limits() {
    assert_eq!(atoi(i8s(b"2147483647\0")), i32::MAX);
    assert_eq!(atoi(i8s(b"-2147483648\0")), i32::MIN);
}

#[test]
fn atol_converts_a_decimal_prefix() {
    assert_eq!(atol(i8s(b" \t\n\x0b\x0c\r+42rest\0ignored")), 42);
    assert_eq!(atol(i8s(b"010!\0")), 10);
    assert_eq!(atol(i8s(b"0x10\0")), 0);
}

#[test]
fn atol_returns_zero_when_no_conversion_is_possible() {
    assert_eq!(atol(i8s(b"not a number\0")), 0);
    assert_eq!(atol(i8s(b"\0")), 0);
}

#[test]
fn atol_accepts_exact_i64_limits() {
    assert_eq!(atol(i8s(b"9223372036854775807\0")), i64::MAX);
    assert_eq!(atol(i8s(b"-9223372036854775808\0")), i64::MIN);
}

#[test]
fn strtol_accepts_a_plus_sign() {
    assert_eq!(strtol(i8s(b"+17\0"), 10), ((17, i8s(b"\0")), Ok(())));
}

#[test]
fn strtol_detects_the_base() {
    assert_eq!(strtol(i8s(b"123\0"), 0), ((123, i8s(b"\0")), Ok(())));
    assert_eq!(strtol(i8s(b"0779\0"), 0), ((63, i8s(b"9\0")), Ok(())));
    assert_eq!(strtol(i8s(b"0x1fZ\0"), 0), ((31, i8s(b"Z\0")), Ok(())));
    assert_eq!(strtol(i8s(b"0XAf\0"), 0), ((175, i8s(b"\0")), Ok(())));
}

#[test]
fn strtol_uses_explicit_bases() {
    assert_eq!(strtol(i8s(b"0X10!\0"), 16), ((16, i8s(b"!\0")), Ok(())));
    assert_eq!(strtol(i8s(b"zZ?\0"), 36), ((1295, i8s(b"?\0")), Ok(())));
    assert_eq!(strtol(i8s(b"1012\0"), 2), ((5, i8s(b"2\0")), Ok(())));
}

#[test]
fn strtol_only_consumes_a_hex_prefix_followed_by_a_digit() {
    assert_eq!(strtol(i8s(b"0x\0"), 0), ((0, i8s(b"x\0")), Ok(())));
    assert_eq!(strtol(i8s(b"-0xg\0"), 16), ((0, i8s(b"xg\0")), Ok(())));
}

#[test]
fn strtol_does_not_accept_a_binary_prefix() {
    assert_eq!(strtol(i8s(b"0b10\0"), 0), ((0, i8s(b"b10\0")), Ok(())));
    assert_eq!(strtol(i8s(b"0b10\0"), 2), ((0, i8s(b"b10\0")), Ok(())));
}

#[test]
fn strtol_returns_the_original_slice_when_there_are_no_digits() {
    for buf in [i8s(b"   \0"), i8s(b"-\0"), i8s(b"+q\0"), i8s(b"\0")] {
        let ((value, suffix), status) = strtol(buf, 10);

        assert_eq!(value, 0);
        assert_eq!(suffix.as_ptr(), buf.as_ptr());
        assert_eq!(suffix.len(), buf.len());
        assert_eq!(status, Ok(()));
    }
}

#[test]
fn strtol_stops_at_the_first_null_byte() {
    assert_eq!(
        strtol(i8s(b"123\x00456"), 10),
        ((123, i8s(b"\x00456")), Ok(()))
    );
}

#[test]
fn strtol_rejects_unsupported_bases() {
    for base in [-1, 1, 37] {
        let buf = i8s(b"10\0");
        let ((value, suffix), status) = strtol(buf, base);

        assert_eq!(value, 0);
        assert_eq!(suffix.as_ptr(), buf.as_ptr());
        assert_eq!(suffix.len(), buf.len());
        assert_eq!(status, Err(StrtoIntError::InvalidBase));
    }
}

#[test]
fn strtol_accepts_exact_i64_limits() {
    assert_eq!(
        strtol(i8s(b"9223372036854775807\0"), 10),
        ((i64::MAX, i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtol(i8s(b"-9223372036854775808\0"), 10),
        ((i64::MIN, i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtol(i8s(b"7fffffffffffffff\0"), 16),
        ((i64::MAX, i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtol(i8s(b"-8000000000000000\0"), 16),
        ((i64::MIN, i8s(b"\0")), Ok(()))
    );
}

#[test]
fn strtol_clamps_overflow_and_consumes_all_digits() {
    assert_eq!(
        strtol(i8s(b"9223372036854775808\0"), 10),
        ((i64::MAX, i8s(b"\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"-9223372036854775809\0"), 10),
        ((i64::MIN, i8s(b"\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"8000000000000000\0"), 16),
        ((i64::MAX, i8s(b"\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"-8000000000000001\0"), 16),
        ((i64::MIN, i8s(b"\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"9223372036854775808123rest\0"), 10),
        ((i64::MAX, i8s(b"rest\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"-9223372036854775809123!\0"), 10),
        ((i64::MIN, i8s(b"!\0")), Err(StrtoIntError::OutOfRange))
    );
}

#[test]
fn strtol_rejects_non_ascii_bytes() {
    let buf = [-1_i8, b'1' as i8, 0];
    let ((value, suffix), status) = strtol(&buf, 10);

    assert_eq!(value, 0);
    assert_eq!(suffix.as_ptr(), buf.as_ptr());
    assert_eq!(status, Ok(()));
}

#[test]
fn strtoul_parses_decimal_and_returns_the_suffix() {
    let buf = i8s(b" \t\n\x0b\x0c\r+42xyz\0ignored");

    assert_eq!(strtoul(buf, 10), ((42, i8s(b"xyz\0ignored")), Ok(())));
}

#[test]
fn strtoul_detects_the_base_and_accepts_explicit_bases() {
    assert_eq!(strtoul(i8s(b"123\0"), 0), ((123, i8s(b"\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"0779\0"), 0), ((63, i8s(b"9\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"0x1fZ\0"), 0), ((31, i8s(b"Z\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"0X10!\0"), 16), ((16, i8s(b"!\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"zZ?\0"), 36), ((1295, i8s(b"?\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"1012\0"), 2), ((5, i8s(b"2\0")), Ok(())));
}

#[test]
fn strtoul_only_consumes_a_hex_prefix_followed_by_a_digit() {
    assert_eq!(strtoul(i8s(b"0x\0"), 0), ((0, i8s(b"x\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"-0xg\0"), 16), ((0, i8s(b"xg\0")), Ok(())));
}

#[test]
fn strtoul_does_not_accept_a_binary_prefix() {
    assert_eq!(strtoul(i8s(b"0b10\0"), 0), ((0, i8s(b"b10\0")), Ok(())));
    assert_eq!(strtoul(i8s(b"0b10\0"), 2), ((0, i8s(b"b10\0")), Ok(())));
}

#[test]
fn strtoul_returns_the_original_slice_when_there_are_no_digits() {
    for buf in [i8s(b"   \0"), i8s(b"-\0"), i8s(b"+q\0"), i8s(b"\0")] {
        let ((value, suffix), status) = strtoul(buf, 10);

        assert_eq!(value, 0);
        assert_eq!(suffix.as_ptr(), buf.as_ptr());
        assert_eq!(suffix.len(), buf.len());
        assert_eq!(status, Ok(()));
    }
}

#[test]
fn strtoul_stops_at_the_first_null_byte() {
    assert_eq!(
        strtoul(i8s(b"123\x00456"), 10),
        ((123, i8s(b"\x00456")), Ok(()))
    );
}

#[test]
fn strtoul_rejects_unsupported_bases() {
    for base in [-1, 1, 37] {
        let buf = i8s(b"10\0");
        let ((value, suffix), status) = strtoul(buf, base);

        assert_eq!(value, 0);
        assert_eq!(suffix.as_ptr(), buf.as_ptr());
        assert_eq!(suffix.len(), buf.len());
        assert_eq!(status, Err(StrtoIntError::InvalidBase));
    }
}

#[test]
fn strtoul_accepts_exact_u64_limits() {
    assert_eq!(
        strtoul(i8s(b"18446744073709551615\0"), 10),
        ((u64::MAX, i8s(b"\0")), Ok(()))
    );
    assert_eq!(
        strtoul(i8s(b"ffffffffffffffff\0"), 16),
        ((u64::MAX, i8s(b"\0")), Ok(()))
    );
}

#[test]
fn strtoul_negates_in_the_unsigned_return_type() {
    assert_eq!(strtoul(i8s(b"-1\0"), 10), ((u64::MAX, i8s(b"\0")), Ok(())));
    assert_eq!(
        strtoul(i8s(b"-18446744073709551615\0"), 10),
        ((1, i8s(b"\0")), Ok(()))
    );
    assert_eq!(strtoul(i8s(b"-0\0"), 10), ((0, i8s(b"\0")), Ok(())));
}

#[test]
fn strtoul_clamps_overflow_and_consumes_all_digits() {
    assert_eq!(
        strtoul(i8s(b"18446744073709551616\0"), 10),
        ((u64::MAX, i8s(b"\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtoul(i8s(b"-18446744073709551616\0"), 10),
        ((u64::MAX, i8s(b"\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtoul(i8s(b"10000000000000000rest\0"), 16),
        ((u64::MAX, i8s(b"rest\0")), Err(StrtoIntError::OutOfRange))
    );
    assert_eq!(
        strtoul(i8s(b"18446744073709551616123!\0"), 10),
        ((u64::MAX, i8s(b"!\0")), Err(StrtoIntError::OutOfRange))
    );
}

#[test]
fn strtoul_rejects_non_ascii_bytes() {
    let buf = [-1_i8, b'1' as i8, 0];
    let ((value, suffix), status) = strtoul(&buf, 10);

    assert_eq!(value, 0);
    assert_eq!(suffix.as_ptr(), buf.as_ptr());
    assert_eq!(status, Ok(()));
}
