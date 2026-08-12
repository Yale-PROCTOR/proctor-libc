use super::{StrtolError, strtol};

fn i8s(bytes: &[u8]) -> &[i8] {
    bytemuck::cast_slice(bytes)
}

#[test]
fn strtol_parses_decimal_and_returns_the_suffix() {
    let buf = i8s(b" \t\n\x0b\x0c\r-42xyz\0ignored");

    assert_eq!(strtol(buf, 10), ((-42, i8s(b"xyz\0ignored")), Ok(())));
}

#[test]
fn strtol_accepts_a_plus_sign() {
    assert_eq!(strtol(i8s(b"+17"), 10), ((17, i8s(b"")), Ok(())));
}

#[test]
fn strtol_detects_the_base() {
    assert_eq!(strtol(i8s(b"123"), 0), ((123, i8s(b"")), Ok(())));
    assert_eq!(strtol(i8s(b"0779"), 0), ((63, i8s(b"9")), Ok(())));
    assert_eq!(strtol(i8s(b"0x1fZ"), 0), ((31, i8s(b"Z")), Ok(())));
    assert_eq!(strtol(i8s(b"0XAf"), 0), ((175, i8s(b"")), Ok(())));
}

#[test]
fn strtol_uses_explicit_bases() {
    assert_eq!(strtol(i8s(b"0X10!"), 16), ((16, i8s(b"!")), Ok(())));
    assert_eq!(strtol(i8s(b"zZ?"), 36), ((1295, i8s(b"?")), Ok(())));
    assert_eq!(strtol(i8s(b"1012"), 2), ((5, i8s(b"2")), Ok(())));
}

#[test]
fn strtol_only_consumes_a_hex_prefix_followed_by_a_digit() {
    assert_eq!(strtol(i8s(b"0x"), 0), ((0, i8s(b"x")), Ok(())));
    assert_eq!(strtol(i8s(b"-0xg"), 16), ((0, i8s(b"xg")), Ok(())));
}

#[test]
fn strtol_does_not_accept_a_binary_prefix() {
    assert_eq!(strtol(i8s(b"0b10"), 0), ((0, i8s(b"b10")), Ok(())));
    assert_eq!(strtol(i8s(b"0b10"), 2), ((0, i8s(b"b10")), Ok(())));
}

#[test]
fn strtol_returns_the_original_slice_when_there_are_no_digits() {
    for buf in [i8s(b""), i8s(b"   "), i8s(b"-"), i8s(b"+q"), i8s(b"\0")] {
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
        let buf = i8s(b"10");
        let ((value, suffix), status) = strtol(buf, base);

        assert_eq!(value, 0);
        assert_eq!(suffix.as_ptr(), buf.as_ptr());
        assert_eq!(suffix.len(), buf.len());
        assert_eq!(status, Err(StrtolError::InvalidBase));
    }
}

#[test]
fn strtol_accepts_exact_i64_limits() {
    assert_eq!(
        strtol(i8s(b"9223372036854775807"), 10),
        ((i64::MAX, i8s(b"")), Ok(()))
    );
    assert_eq!(
        strtol(i8s(b"-9223372036854775808"), 10),
        ((i64::MIN, i8s(b"")), Ok(()))
    );
    assert_eq!(
        strtol(i8s(b"7fffffffffffffff"), 16),
        ((i64::MAX, i8s(b"")), Ok(()))
    );
    assert_eq!(
        strtol(i8s(b"-8000000000000000"), 16),
        ((i64::MIN, i8s(b"")), Ok(()))
    );
}

#[test]
fn strtol_clamps_overflow_and_consumes_all_digits() {
    assert_eq!(
        strtol(i8s(b"9223372036854775808"), 10),
        ((i64::MAX, i8s(b"")), Err(StrtolError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"-9223372036854775809"), 10),
        ((i64::MIN, i8s(b"")), Err(StrtolError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"8000000000000000"), 16),
        ((i64::MAX, i8s(b"")), Err(StrtolError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"-8000000000000001"), 16),
        ((i64::MIN, i8s(b"")), Err(StrtolError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"9223372036854775808123rest"), 10),
        ((i64::MAX, i8s(b"rest")), Err(StrtolError::OutOfRange))
    );
    assert_eq!(
        strtol(i8s(b"-9223372036854775809123!"), 10),
        ((i64::MIN, i8s(b"!")), Err(StrtolError::OutOfRange))
    );
}

#[test]
fn strtol_rejects_non_ascii_bytes() {
    let buf = [-1_i8, b'1' as i8];
    let ((value, suffix), status) = strtol(&buf, 10);

    assert_eq!(value, 0);
    assert_eq!(suffix.as_ptr(), buf.as_ptr());
    assert_eq!(status, Ok(()));
}
