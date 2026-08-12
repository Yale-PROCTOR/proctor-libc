use super::{strcmp, strlen, strncmp};

fn i8s(bytes: &[u8]) -> &[i8] {
    bytemuck::cast_slice(bytes)
}

#[test]
fn strlen_returns_the_number_of_bytes_before_the_null_byte() {
    assert_eq!(strlen(i8s(b"\0")), 0);
    assert_eq!(strlen(i8s(b"hello\0")), 5);
}

#[test]
fn strlen_stops_at_the_first_null_byte() {
    assert_eq!(strlen(i8s(b"hello\0world\0")), 5);
}

#[test]
fn strlen_counts_bytes_with_the_high_bit_set() {
    let expected = size_of::<usize>() + 2;
    let mut s = vec![-1; expected];
    s.extend_from_slice(&[0, 2]);

    assert_eq!(strlen(&s), expected);
}

#[test]
fn strlen_finds_null_bytes_at_each_word_offset() {
    let word_bytes = size_of::<usize>();

    for expected in 0..=word_bytes * 3 {
        let mut s = vec![1; expected];
        s.extend_from_slice(&[0, 2]);
        assert_eq!(strlen(&s), expected);
    }
}

#[test]
fn strcmp_compares_strings() {
    assert_eq!(strcmp(i8s(b"\0"), i8s(b"\0")), 0);
    assert_eq!(strcmp(i8s(b"hello\0"), i8s(b"hello\0")), 0);
    assert_eq!(strcmp(i8s(b"abc\0"), i8s(b"abd\0")), -1);
    assert_eq!(strcmp(i8s(b"abe\0"), i8s(b"abc\0")), 2);
}

#[test]
fn strcmp_compares_bytes_as_unsigned() {
    assert_eq!(strcmp(&[-1, 0], &[1, 0]), 254);
    assert_eq!(strcmp(&[1, 0], &[-1, 0]), -254);
}

#[test]
fn strcmp_stops_at_the_first_null_byte() {
    assert_eq!(strcmp(i8s(b"same\0a"), i8s(b"same\0b")), 0);
}

#[test]
fn strcmp_stops_at_a_null_byte_in_a_complete_word() {
    let word_bytes = size_of::<usize>();
    let mut s1 = vec![1; word_bytes + 1];
    let mut s2 = s1.clone();
    s1[word_bytes - 1] = 0;
    s2[word_bytes - 1] = 0;
    s1[word_bytes] = 2;
    s2[word_bytes] = 3;

    assert_eq!(strcmp(&s1, &s2), 0);
}

#[test]
fn strcmp_finds_differences_at_each_word_offset() {
    for offset in 0..=size_of::<usize>() * 3 {
        let mut s1 = vec![1; offset];
        let mut s2 = s1.clone();
        s1.extend_from_slice(&[2, 0]);
        s2.extend_from_slice(&[3, 0]);

        assert_eq!(strcmp(&s1, &s2), -1);
    }
}

#[test]
fn strncmp_respects_the_byte_limit() {
    assert_eq!(strncmp(i8s(b"abcx\0"), i8s(b"abcy\0"), 3), 0);
    assert_eq!(strncmp(i8s(b"abcx\0"), i8s(b"abcy\0"), 4), -1);
}

#[test]
fn strncmp_handles_a_finite_word_sized_limit() {
    let word_bytes = size_of::<usize>();
    let s1 = vec![1; word_bytes + 1];
    let mut s2 = s1.clone();
    s2[word_bytes] = 2;

    assert_eq!(strncmp(&s1, &s2, word_bytes), 0);
    assert_eq!(strncmp(&s1, &s2, word_bytes + 1), -1);
}

#[test]
fn strncmp_accepts_prefixes_without_null_bytes() {
    assert_eq!(strncmp(i8s(b"abc"), i8s(b"abc"), 3), 0);
    assert_eq!(strncmp(i8s(b"abc"), i8s(b"abd"), 3), -1);
}

#[test]
fn strncmp_stops_at_the_first_null_byte() {
    assert_eq!(strncmp(i8s(b"a\0"), i8s(b"a\0"), usize::MAX), 0);
    assert_eq!(strncmp(i8s(b"a\0x"), i8s(b"a\0y"), 3), 0);
}

#[test]
fn strncmp_with_zero_limit_compares_no_bytes() {
    assert_eq!(strncmp(&[], &[], 0), 0);
    assert_eq!(strncmp(i8s(b"a"), i8s(b"b"), 0), 0);
}

#[test]
fn strncmp_compares_bytes_as_unsigned() {
    assert_eq!(strncmp(&[-1], &[1], 1), 254);
    assert_eq!(strncmp(&[1], &[-1], 1), -254);
}
