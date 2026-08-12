use super::strlen;

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
