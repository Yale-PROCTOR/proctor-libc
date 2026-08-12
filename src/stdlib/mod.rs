/// An error reported by [`strtol`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrtolError {
    /// The requested base is not supported.
    InvalidBase,
    /// The converted value is outside the range of an `i64`.
    OutOfRange,
}

/// Converts the initial integer in `buf` using `base`.
///
/// Returns the converted value, the unconsumed suffix, and the conversion
/// status. No conversion returns zero and leaves all of `buf` unconsumed.
/// `StrtolError::InvalidBase` indicates that `base` is neither zero nor in
/// `2..=36`. `StrtolError::OutOfRange` indicates that the result does not fit
/// in `i64`; the returned value is clamped to [`i64::MIN`] or [`i64::MAX`].
pub fn strtol(buf: &[i8], base: i32) -> ((i64, &[i8]), Result<(), StrtolError>) {
    if base != 0 && !(2..=36).contains(&base) {
        return ((0, buf), Err(StrtolError::InvalidBase));
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

    let first_digit = index;
    let limit = if negative {
        i64::MAX as u64 + 1
    } else {
        i64::MAX as u64
    };
    let cutoff = limit / u64::from(radix);
    let cutlim = limit % u64::from(radix);
    let mut magnitude = 0_u64;
    let mut out_of_range = false;

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

    if index == first_digit {
        return ((0, buf), Ok(()));
    }

    if out_of_range {
        let value = if negative { i64::MIN } else { i64::MAX };
        return ((value, &buf[index..]), Err(StrtolError::OutOfRange));
    }

    let value = if negative {
        if magnitude == i64::MAX as u64 + 1 {
            i64::MIN
        } else {
            -(magnitude as i64)
        }
    } else {
        magnitude as i64
    };

    ((value, &buf[index..]), Ok(()))
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
