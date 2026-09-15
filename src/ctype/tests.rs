use super::{
    isalnum, isalpha, isblank, iscntrl, isdigit, isgraph, islower, isprint, ispunct, isspace,
    isupper, isxdigit, tolower, toupper,
};

#[cfg(unix)]
mod differential;

fn assert_classification(function: fn(i32) -> i32, members: &[u8], nonmembers: &[u8]) {
    for &c in members {
        assert_eq!(function(i32::from(c)), 1, "expected {c:#04x} to match");
    }
    for &c in nonmembers {
        assert_eq!(function(i32::from(c)), 0, "expected {c:#04x} not to match");
    }
}

#[test]
fn classifies_letters_and_digits() {
    assert_classification(isalnum, b"09AZaz", b" /:@[`{");
    assert_classification(isalpha, b"AZaz", b"09@[`{");
    assert_classification(isdigit, b"09", b" /:AZaz");
    assert_classification(islower, b"az", b"`{AZ09");
    assert_classification(isupper, b"AZ", b"@[az09");
    assert_classification(isxdigit, b"09AFaf", b"/G`g");
}

#[test]
fn classifies_blank_whitespace_and_control_characters() {
    assert_classification(isblank, b"\t ", b"\n\r\x0b\x0c!");
    assert_classification(isspace, b"\t\n\x0b\x0c\r ", b"\x08\x0e!");
    assert_classification(iscntrl, &[0x00, 0x1f, 0x7f], &[0x20, 0x21, 0x7e]);
}

#[test]
fn classifies_graphical_printable_and_punctuation_characters() {
    assert_classification(isgraph, b"!~0AZaz", b" \x7f");
    assert_classification(isprint, b" !~0AZaz", b"\x1f\x7f");
    assert_classification(ispunct, b"!/:@[`{~", b" 09AZaz\x7f");
}

#[test]
fn classification_functions_reject_eof_and_non_ascii_bytes() {
    let functions = [
        isalnum, isalpha, isblank, iscntrl, isdigit, isgraph, islower, isprint, ispunct, isspace,
        isupper, isxdigit,
    ];

    for function in functions {
        assert_eq!(function(-1), 0);
        assert_eq!(function(0x80), 0);
        assert_eq!(function(0xff), 0);
    }
}

#[test]
fn tolower_converts_only_uppercase_letters() {
    assert_eq!(tolower(i32::from(b'A')), i32::from(b'a'));
    assert_eq!(tolower(i32::from(b'Z')), i32::from(b'z'));
    for c in [-1, 0, i32::from(b'@'), i32::from(b'['), 0x80, 0xff] {
        assert_eq!(tolower(c), c);
    }
}

#[test]
fn toupper_converts_only_lowercase_letters() {
    assert_eq!(toupper(i32::from(b'a')), i32::from(b'A'));
    assert_eq!(toupper(i32::from(b'z')), i32::from(b'Z'));
    for c in [-1, 0, i32::from(b'`'), i32::from(b'{'), 0x80, 0xff] {
        assert_eq!(toupper(c), c);
    }
}
