//! Safe equivalents of functions declared in C's `stdlib.h` header.

use num_bigint::BigUint;

/// An integer conversion error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrtoIntError {
    /// The requested base is not supported.
    InvalidBase,
    /// The converted value is outside the range of the return type.
    OutOfRange,
}

/// A floating-point conversion error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrtoFloatError {
    /// The converted value overflowed or underflowed the return type.
    OutOfRange,
}

const F32_FORMAT: FloatFormat = FloatFormat {
    precision: 24,
    fraction_bits: 23,
    exponent_bits: 8,
    exponent_bias: 127,
    minimum_exponent: -126,
    maximum_exponent: 127,
    minimum_decimal_order: -45,
    maximum_decimal_order: 38,
};
const F64_FORMAT: FloatFormat = FloatFormat {
    precision: 53,
    fraction_bits: 52,
    exponent_bits: 11,
    exponent_bias: 1023,
    minimum_exponent: -1022,
    maximum_exponent: 1023,
    minimum_decimal_order: -324,
    maximum_decimal_order: 308,
};
const F128_FORMAT: FloatFormat = FloatFormat {
    precision: 113,
    fraction_bits: 112,
    exponent_bits: 15,
    exponent_bias: 16383,
    minimum_exponent: -16382,
    maximum_exponent: 16383,
    minimum_decimal_order: -4966,
    maximum_decimal_order: 4932,
};

#[derive(Clone, Copy)]
struct FloatFormat {
    precision: u32,
    fraction_bits: u32,
    exponent_bits: u32,
    exponent_bias: i32,
    minimum_exponent: i64,
    maximum_exponent: i64,
    minimum_decimal_order: i64,
    maximum_decimal_order: i64,
}

enum FloatSubject {
    None,
    Infinity {
        negative: bool,
        end: usize,
    },
    Nan {
        negative: bool,
        end: usize,
    },
    Finite {
        negative: bool,
        digits: Vec<u8>,
        scale: FloatScale,
        end: usize,
    },
}

enum FloatScale {
    Decimal(i64),
    Binary(i64),
}

struct Mantissa {
    digits: Vec<u8>,
    saw_digit: bool,
    fraction_digits: usize,
    trailing_zeros: usize,
    end: usize,
}

/// Converts the initial floating-point number in `buf` to an `f64`.
pub fn atof(buf: &[i8]) -> f64 {
    strtod(buf).0.0
}

/// Converts the initial floating-point number in `buf` to an `f64`.
///
/// Returns the converted value, the unconsumed suffix, and the conversion
/// status. `StrtoFloatError::OutOfRange` is the only error variant and reports
/// overflow or inexact underflow.
pub fn strtod(buf: &[i8]) -> ((f64, &[i8]), Result<(), StrtoFloatError>) {
    let ((bits, suffix), status) = strto_float_bits(buf, F64_FORMAT);
    ((f64::from_bits(bits as u64), suffix), status)
}

/// Converts the initial floating-point number in `buf` to an `f32`.
///
/// Returns the converted value, the unconsumed suffix, and the conversion
/// status. `StrtoFloatError::OutOfRange` is the only error variant and reports
/// overflow or inexact underflow.
pub fn strtof(buf: &[i8]) -> ((f32, &[i8]), Result<(), StrtoFloatError>) {
    let ((bits, suffix), status) = strto_float_bits(buf, F32_FORMAT);
    ((f32::from_bits(bits as u32), suffix), status)
}

/// Converts the initial floating-point number in `buf` to [`struct@f128::f128`].
///
/// Returns the converted value, the unconsumed suffix, and the conversion
/// status. `StrtoFloatError::OutOfRange` is the only error variant and reports
/// overflow or inexact underflow.
pub fn strtold(buf: &[i8]) -> ((f128::f128, &[i8]), Result<(), StrtoFloatError>) {
    let ((bits, suffix), status) = strto_float_bits(buf, F128_FORMAT);
    ((f128_from_bits(bits), suffix), status)
}

fn strto_float_bits(
    buf: &[i8],
    format: FloatFormat,
) -> ((u128, &[i8]), Result<(), StrtoFloatError>) {
    match scan_float_subject(buf) {
        FloatSubject::None => ((0, buf), Ok(())),
        FloatSubject::Infinity { negative, end } => {
            ((infinity_bits(format, negative), &buf[end..]), Ok(()))
        }
        FloatSubject::Nan { negative, end } => ((nan_bits(format, negative), &buf[end..]), Ok(())),
        FloatSubject::Finite {
            negative,
            digits,
            scale,
            end,
        } => {
            if digits.is_empty() {
                return ((zero_bits(format, negative), &buf[end..]), Ok(()));
            }

            let (bits, out_of_range) = convert_finite_float(negative, &digits, scale, format);
            let status = if out_of_range {
                Err(StrtoFloatError::OutOfRange)
            } else {
                Ok(())
            };
            ((bits, &buf[end..]), status)
        }
    }
}

fn scan_float_subject(buf: &[i8]) -> FloatSubject {
    let limit = buf.iter().position(|&byte| byte == 0).unwrap_or(buf.len());
    let mut index = 0;
    while index < limit && is_c_whitespace(buf[index]) {
        index += 1;
    }

    let negative = match buf.get(index).copied() {
        Some(byte) if byte == b'-' as i8 => {
            index += 1;
            true
        }
        Some(byte) if byte == b'+' as i8 => {
            index += 1;
            false
        }
        _ => false,
    };
    let number_start = index;

    if starts_ignore_ascii_case(buf, index, limit, b"inf") {
        index += 3;
        if starts_ignore_ascii_case(buf, index, limit, b"inity") {
            index += 5;
        }
        return FloatSubject::Infinity {
            negative,
            end: index,
        };
    }

    if starts_ignore_ascii_case(buf, index, limit, b"nan") {
        index += 3;
        if buf.get(index).copied() == Some(b'(' as i8) {
            let mut payload_end = index + 1;
            while payload_end < limit && is_nan_character(buf[payload_end]) {
                payload_end += 1;
            }
            if buf.get(payload_end).copied() == Some(b')' as i8) {
                index = payload_end + 1;
            }
        }
        return FloatSubject::Nan {
            negative,
            end: index,
        };
    }

    if index + 2 <= limit
        && buf.get(index).copied() == Some(b'0' as i8)
        && matches!(buf.get(index + 1).copied(), Some(byte) if byte == b'x' as i8 || byte == b'X' as i8)
    {
        let mantissa = scan_mantissa(buf, index + 2, limit, 16);
        if mantissa.saw_digit {
            let (end, exponent) = scan_float_exponent(buf, mantissa.end, limit, b'p', b'P');
            let scale = adjust_scale(
                exponent,
                mantissa.fraction_digits,
                mantissa.trailing_zeros,
                4,
            );
            return FloatSubject::Finite {
                negative,
                digits: mantissa.digits,
                scale: FloatScale::Binary(scale),
                end,
            };
        }
    }

    let mantissa = scan_mantissa(buf, number_start, limit, 10);
    if !mantissa.saw_digit {
        return FloatSubject::None;
    }
    let (end, exponent) = scan_float_exponent(buf, mantissa.end, limit, b'e', b'E');
    FloatSubject::Finite {
        negative,
        digits: mantissa.digits,
        scale: FloatScale::Decimal(adjust_scale(
            exponent,
            mantissa.fraction_digits,
            mantissa.trailing_zeros,
            1,
        )),
        end,
    }
}

fn scan_mantissa(buf: &[i8], start: usize, limit: usize, radix: u32) -> Mantissa {
    let mut digits = Vec::new();
    let mut saw_digit = false;
    let mut fraction_digits = 0;
    let mut pending_zeros = 0;
    let mut index = start;
    let mut after_point = false;

    while index < limit {
        if !after_point && buf[index] == b'.' as i8 {
            after_point = true;
            index += 1;
            continue;
        }
        let Some(digit) = digit_value(buf[index]).filter(|&digit| digit < radix) else {
            break;
        };
        saw_digit = true;
        if digit == 0 {
            if !digits.is_empty() {
                pending_zeros += 1;
            }
        } else {
            digits.extend(std::iter::repeat_n(0, pending_zeros));
            pending_zeros = 0;
            digits.push(digit as u8);
        }
        if after_point {
            fraction_digits += 1;
        }
        index += 1;
    }

    Mantissa {
        digits,
        saw_digit,
        fraction_digits,
        trailing_zeros: pending_zeros,
        end: index,
    }
}

fn scan_float_exponent(
    buf: &[i8],
    marker: usize,
    limit: usize,
    lower: u8,
    upper: u8,
) -> (usize, i64) {
    if marker >= limit || !matches!(buf[marker] as u8, byte if byte == lower || byte == upper) {
        return (marker, 0);
    }

    let mut index = marker + 1;
    let negative = match buf.get(index).copied() {
        Some(byte) if byte == b'-' as i8 => {
            index += 1;
            true
        }
        Some(byte) if byte == b'+' as i8 => {
            index += 1;
            false
        }
        _ => false,
    };
    let first_digit = index;
    let mut exponent = 0_i64;
    while index < limit {
        let byte = buf[index] as u8;
        if !byte.is_ascii_digit() {
            break;
        }
        exponent = exponent
            .saturating_mul(10)
            .saturating_add(i64::from(byte - b'0'));
        index += 1;
    }
    if index == first_digit {
        return (marker, 0);
    }
    if negative {
        exponent = -exponent;
    }
    (index, exponent)
}

fn convert_finite_float(
    negative: bool,
    digits: &[u8],
    scale: FloatScale,
    format: FloatFormat,
) -> (u128, bool) {
    let (numerator, denominator, binary_exponent) = match scale {
        FloatScale::Decimal(decimal_exponent) => {
            let decimal_order = usize_to_i64(digits.len())
                .saturating_sub(1)
                .saturating_add(decimal_exponent);
            if decimal_order > format.maximum_decimal_order {
                return (infinity_bits(format, negative), true);
            }
            if decimal_order < format.minimum_decimal_order.saturating_sub(1) {
                return (zero_bits(format, negative), true);
            }

            let significand = BigUint::from_radix_be(digits, 10).expect("decimal digits are valid");
            if decimal_exponent >= 0 {
                let five = BigUint::from(5_u8).pow(decimal_exponent as u32);
                (significand * five, BigUint::from(1_u8), decimal_exponent)
            } else {
                let magnitude = decimal_exponent.unsigned_abs() as u32;
                (
                    significand,
                    BigUint::from(5_u8).pow(magnitude),
                    decimal_exponent,
                )
            }
        }
        FloatScale::Binary(binary_exponent) => {
            let leading_digit_exponent = i64::from(7 - digits[0].leading_zeros());
            let exponent = usize_to_i64(digits.len().saturating_sub(1))
                .saturating_mul(4)
                .saturating_add(leading_digit_exponent)
                .saturating_add(binary_exponent);
            let minimum_subnormal_exponent =
                format.minimum_exponent - i64::from(format.precision - 1);
            if exponent > format.maximum_exponent {
                return (infinity_bits(format, negative), true);
            }
            if exponent < minimum_subnormal_exponent - 1 {
                return (zero_bits(format, negative), true);
            }

            (
                BigUint::from_radix_be(digits, 16).expect("hexadecimal digits are valid"),
                BigUint::from(1_u8),
                binary_exponent,
            )
        }
    };

    encode_ratio(negative, &numerator, &denominator, binary_exponent, format)
}

fn encode_ratio(
    negative: bool,
    numerator: &BigUint,
    denominator: &BigUint,
    binary_exponent: i64,
    format: FloatFormat,
) -> (u128, bool) {
    let ratio_exponent = floor_log2_ratio(numerator, denominator);
    let mut exponent = ratio_exponent.saturating_add(binary_exponent);
    let minimum_subnormal_exponent = format.minimum_exponent - i64::from(format.precision - 1);

    if exponent >= format.minimum_exponent {
        let shift = i64::from(format.precision - 1) - ratio_exponent;
        let (mut significand, _) = round_ratio(numerator, denominator, shift);
        if significand.bits() > u64::from(format.precision) {
            significand >>= 1_usize;
            exponent += 1;
        }
        if exponent > format.maximum_exponent {
            return (infinity_bits(format, negative), true);
        }

        let significand = biguint_to_u128(&significand);
        let implicit_bit = 1_u128 << (format.precision - 1);
        let exponent_field = (exponent + i64::from(format.exponent_bias)) as u128;
        let bits = sign_bits(format, negative)
            | (exponent_field << format.fraction_bits)
            | (significand - implicit_bit);
        return (bits, false);
    }

    let shift = binary_exponent.saturating_sub(minimum_subnormal_exponent);
    let (significand, inexact) = round_ratio(numerator, denominator, shift);
    let significand = biguint_to_u128(&significand);
    let minimum_normal_significand = 1_u128 << (format.precision - 1);
    if significand >= minimum_normal_significand {
        let bits = sign_bits(format, negative) | (1_u128 << format.fraction_bits);
        return (bits, inexact);
    }

    (sign_bits(format, negative) | significand, inexact)
}

fn floor_log2_ratio(numerator: &BigUint, denominator: &BigUint) -> i64 {
    let exponent = (numerator.bits() as i64) - (denominator.bits() as i64);
    let below_power = if exponent >= 0 {
        numerator < &(denominator << exponent as usize)
    } else {
        &(numerator << exponent.unsigned_abs() as usize) < denominator
    };
    if below_power { exponent - 1 } else { exponent }
}

fn round_ratio(numerator: &BigUint, denominator: &BigUint, shift: i64) -> (BigUint, bool) {
    let (scaled_numerator, scaled_denominator) = if shift >= 0 {
        (numerator << shift as usize, denominator.clone())
    } else {
        (
            numerator.clone(),
            denominator << shift.unsigned_abs() as usize,
        )
    };
    let mut quotient = &scaled_numerator / &scaled_denominator;
    let remainder = &scaled_numerator % &scaled_denominator;
    let inexact = remainder.bits() != 0;
    let doubled_remainder = &remainder << 1_usize;
    if doubled_remainder > scaled_denominator
        || (doubled_remainder == scaled_denominator && quotient.bit(0))
    {
        quotient += 1_u8;
    }
    (quotient, inexact)
}

fn biguint_to_u128(value: &BigUint) -> u128 {
    let digits = value.to_u64_digits();
    u128::from(digits.first().copied().unwrap_or(0))
        | (u128::from(digits.get(1).copied().unwrap_or(0)) << 64)
}

fn zero_bits(format: FloatFormat, negative: bool) -> u128 {
    sign_bits(format, negative)
}

fn infinity_bits(format: FloatFormat, negative: bool) -> u128 {
    sign_bits(format, negative) | (maximum_exponent_field(format) << format.fraction_bits)
}

fn nan_bits(format: FloatFormat, negative: bool) -> u128 {
    infinity_bits(format, negative) | (1_u128 << (format.fraction_bits - 1))
}

fn sign_bits(format: FloatFormat, negative: bool) -> u128 {
    u128::from(negative) << (format.fraction_bits + format.exponent_bits)
}

fn maximum_exponent_field(format: FloatFormat) -> u128 {
    (1_u128 << format.exponent_bits) - 1
}

fn f128_from_bits(bits: u128) -> f128::f128 {
    // SAFETY: `f128::f128` is a `repr(C)` wrapper around `[u8; 16]`, and all
    // IEEE binary128 bit patterns are valid values of that type.
    unsafe { std::mem::transmute(bits.to_ne_bytes()) }
}

fn starts_ignore_ascii_case(buf: &[i8], start: usize, limit: usize, expected: &[u8]) -> bool {
    start + expected.len() <= limit
        && buf[start..start + expected.len()]
            .iter()
            .zip(expected)
            .all(|(&actual, &expected)| (actual as u8).eq_ignore_ascii_case(&expected))
}

fn is_nan_character(byte: i8) -> bool {
    matches!(byte as u8, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_')
}

fn usize_to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn adjust_scale(
    exponent: i64,
    fractional_digits: usize,
    trailing_zeros: usize,
    radix_bits: i64,
) -> i64 {
    if fractional_digits >= trailing_zeros {
        let adjustment =
            usize_to_i64(fractional_digits - trailing_zeros).saturating_mul(radix_bits);
        exponent.saturating_sub(adjustment)
    } else {
        let adjustment =
            usize_to_i64(trailing_zeros - fractional_digits).saturating_mul(radix_bits);
        exponent.saturating_add(adjustment)
    }
}

enum IntegerSubject {
    None,
    Digits {
        negative: bool,
        radix: u32,
        start: usize,
    },
}

struct IntegerMagnitude {
    magnitude: u64,
    end: usize,
    out_of_range: bool,
}

/// Converts the initial decimal integer in `buf` to an `i32`.
pub fn atoi(buf: &[i8]) -> i32 {
    strtol(buf, 10).0.0 as i32
}

/// Converts the initial decimal integer in `buf` to an `i64`.
pub fn atol(buf: &[i8]) -> i64 {
    strtol(buf, 10).0.0
}

/// Converts the initial integer in `buf` using `base`.
///
/// Returns the converted value, the unconsumed suffix, and the conversion
/// status. `StrtoIntError::InvalidBase` and `StrtoIntError::OutOfRange` are the
/// only error variants; they report an unsupported base and overflow,
/// respectively.
pub fn strtol(buf: &[i8], base: i32) -> ((i64, &[i8]), Result<(), StrtoIntError>) {
    let subject = match scan_integer_subject(buf, base) {
        Ok(subject) => subject,
        Err(error) => return ((0, buf), Err(error)),
    };
    let IntegerSubject::Digits {
        negative,
        radix,
        start,
    } = subject
    else {
        return ((0, buf), Ok(()));
    };
    let limit = if negative {
        i64::MAX as u64 + 1
    } else {
        i64::MAX as u64
    };
    let conversion = convert_integer_magnitude(buf, start, radix, limit);

    if conversion.out_of_range {
        let value = if negative { i64::MIN } else { i64::MAX };
        return (
            (value, &buf[conversion.end..]),
            Err(StrtoIntError::OutOfRange),
        );
    }

    let value = if negative {
        if conversion.magnitude == i64::MAX as u64 + 1 {
            i64::MIN
        } else {
            -(conversion.magnitude as i64)
        }
    } else {
        conversion.magnitude as i64
    };

    ((value, &buf[conversion.end..]), Ok(()))
}

/// Converts the initial unsigned integer in `buf` using `base`.
///
/// Returns the converted value, the unconsumed suffix, and the conversion
/// status. `StrtoIntError::InvalidBase` and `StrtoIntError::OutOfRange` are the
/// only error variants; they report an unsupported base and overflow,
/// respectively.
pub fn strtoul(buf: &[i8], base: i32) -> ((u64, &[i8]), Result<(), StrtoIntError>) {
    let subject = match scan_integer_subject(buf, base) {
        Ok(subject) => subject,
        Err(error) => return ((0, buf), Err(error)),
    };
    let IntegerSubject::Digits {
        negative,
        radix,
        start,
    } = subject
    else {
        return ((0, buf), Ok(()));
    };
    let conversion = convert_integer_magnitude(buf, start, radix, u64::MAX);

    if conversion.out_of_range {
        return (
            (u64::MAX, &buf[conversion.end..]),
            Err(StrtoIntError::OutOfRange),
        );
    }

    let value = if negative {
        0_u64.wrapping_sub(conversion.magnitude)
    } else {
        conversion.magnitude
    };

    ((value, &buf[conversion.end..]), Ok(()))
}

fn scan_integer_subject(buf: &[i8], base: i32) -> Result<IntegerSubject, StrtoIntError> {
    if base != 0 && !(2..=36).contains(&base) {
        return Err(StrtoIntError::InvalidBase);
    }

    let mut index = 0;
    while index < buf.len() && is_c_whitespace(buf[index]) {
        index += 1;
    }

    let negative = match buf.get(index).copied() {
        Some(byte) if byte == b'-' as i8 => {
            index += 1;
            true
        }
        Some(byte) if byte == b'+' as i8 => {
            index += 1;
            false
        }
        _ => false,
    };

    let radix = if base == 0 {
        if has_hex_prefix(buf, index) {
            index += 2;
            16
        } else if buf.get(index).copied() == Some(b'0' as i8) {
            8
        } else {
            10
        }
    } else {
        let radix = base as u32;
        if radix == 16 && has_hex_prefix(buf, index) {
            index += 2;
        }
        radix
    };

    if !matches!(buf.get(index).copied().and_then(digit_value), Some(digit) if digit < radix) {
        return Ok(IntegerSubject::None);
    }

    Ok(IntegerSubject::Digits {
        negative,
        radix,
        start: index,
    })
}

fn convert_integer_magnitude(buf: &[i8], start: usize, radix: u32, limit: u64) -> IntegerMagnitude {
    let cutoff = limit / u64::from(radix);
    let cutlim = limit % u64::from(radix);
    let mut magnitude = 0_u64;
    let mut out_of_range = false;
    let mut index = start;

    while let Some(digit) = buf.get(index).copied().and_then(digit_value) {
        if digit >= radix {
            break;
        }

        if !out_of_range {
            let digit = u64::from(digit);
            if magnitude > cutoff || (magnitude == cutoff && digit > cutlim) {
                out_of_range = true;
            } else {
                magnitude = magnitude * u64::from(radix) + digit;
            }
        }
        index += 1;
    }

    IntegerMagnitude {
        magnitude,
        end: index,
        out_of_range,
    }
}

fn is_c_whitespace(byte: i8) -> bool {
    matches!(byte as u8, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r')
}

fn has_hex_prefix(buf: &[i8], index: usize) -> bool {
    buf.get(index).copied() == Some(b'0' as i8)
        && matches!(buf.get(index + 1).copied(), Some(byte) if byte == b'x' as i8 || byte == b'X' as i8)
        && matches!(
            buf.get(index + 2).copied().and_then(digit_value),
            Some(0..=15)
        )
}

fn digit_value(byte: i8) -> Option<u32> {
    match byte as u8 {
        byte @ b'0'..=b'9' => Some(u32::from(byte - b'0')),
        byte @ b'a'..=b'z' => Some(u32::from(byte - b'a') + 10),
        byte @ b'A'..=b'Z' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
