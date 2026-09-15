//! Safe equivalents of functions declared in C's `ctype.h` header.

fn in_range(c: i32, start: u8, end: u8) -> bool {
    i32::from(start) <= c && c <= i32::from(end)
}

/// Tests whether `c` is an alphanumeric character in the C locale, returning `1` or `0`.
pub fn isalnum(c: i32) -> i32 {
    (isalpha(c) != 0 || isdigit(c) != 0) as i32
}

/// Tests whether `c` is an alphabetic character in the C locale, returning `1` or `0`.
pub fn isalpha(c: i32) -> i32 {
    (isupper(c) != 0 || islower(c) != 0) as i32
}

/// Tests whether `c` is a blank character in the C locale, returning `1` or `0`.
pub fn isblank(c: i32) -> i32 {
    (c == i32::from(b' ') || c == i32::from(b'\t')) as i32
}

/// Tests whether `c` is a control character in the C locale, returning `1` or `0`.
pub fn iscntrl(c: i32) -> i32 {
    (in_range(c, 0, 0x1f) || c == 0x7f) as i32
}

/// Tests whether `c` is a decimal digit in the C locale, returning `1` or `0`.
pub fn isdigit(c: i32) -> i32 {
    in_range(c, b'0', b'9') as i32
}

/// Tests whether `c` is a graphical character in the C locale, returning `1` or `0`.
pub fn isgraph(c: i32) -> i32 {
    in_range(c, 0x21, 0x7e) as i32
}

/// Tests whether `c` is a lowercase letter in the C locale, returning `1` or `0`.
pub fn islower(c: i32) -> i32 {
    in_range(c, b'a', b'z') as i32
}

/// Tests whether `c` is a printable character in the C locale, returning `1` or `0`.
pub fn isprint(c: i32) -> i32 {
    in_range(c, 0x20, 0x7e) as i32
}

/// Tests whether `c` is a punctuation character in the C locale, returning `1` or `0`.
pub fn ispunct(c: i32) -> i32 {
    (isgraph(c) != 0 && isalnum(c) == 0) as i32
}

/// Tests whether `c` is a whitespace character in the C locale, returning `1` or `0`.
pub fn isspace(c: i32) -> i32 {
    (c == i32::from(b' ') || in_range(c, b'\t', b'\r')) as i32
}

/// Tests whether `c` is an uppercase letter in the C locale, returning `1` or `0`.
pub fn isupper(c: i32) -> i32 {
    in_range(c, b'A', b'Z') as i32
}

/// Tests whether `c` is a hexadecimal digit in the C locale, returning `1` or `0`.
pub fn isxdigit(c: i32) -> i32 {
    (isdigit(c) != 0 || in_range(c, b'A', b'F') || in_range(c, b'a', b'f')) as i32
}

/// Converts an uppercase letter to lowercase in the C locale.
pub fn tolower(c: i32) -> i32 {
    if isupper(c) != 0 {
        c + i32::from(b'a' - b'A')
    } else {
        c
    }
}

/// Converts a lowercase letter to uppercase in the C locale.
pub fn toupper(c: i32) -> i32 {
    if islower(c) != 0 {
        c - i32::from(b'a' - b'A')
    } else {
        c
    }
}

#[cfg(test)]
mod tests;
