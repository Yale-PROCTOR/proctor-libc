use super::{strcat, strcmp, strcpy, strlen, strncat, strncmp, strncpy};

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

#[test]
fn strcpy_copies_through_the_first_null_byte() {
    let mut s1 = i8s(b"old contents\0").to_vec();

    let result = strcpy(&mut s1, i8s(b"new\0ignored"));

    assert_eq!(result, i8s(b"new\0contents\0"));
}

#[test]
fn strcpy_handles_an_empty_source() {
    let mut s1 = i8s(b"unchanged\0").to_vec();

    let result = strcpy(&mut s1, i8s(b"\0ignored"));

    assert_eq!(result, i8s(b"\0nchanged\0"));
}

#[test]
fn strcpy_accepts_a_destination_ending_at_the_copied_null_byte() {
    let mut s1 = vec![1; 4];

    let result = strcpy(&mut s1, i8s(b"new\0ignored"));

    assert_eq!(result, i8s(b"new\0"));
}

#[test]
fn strcpy_returns_the_complete_destination_slice() {
    let mut s1 = i8s(b"old\0tail").to_vec();
    let pointer = s1.as_ptr();
    let length = s1.len();

    let result = strcpy(&mut s1, i8s(b"new\0"));

    assert_eq!(result.as_ptr(), pointer);
    assert_eq!(result.len(), length);
}

#[test]
fn strcpy_finds_source_null_bytes_at_each_word_offset() {
    for length in 0..=size_of::<usize>() * 3 {
        let mut s1 = vec![3; length + 2];
        let mut s2 = vec![1; length];
        s2.extend_from_slice(&[0, 2]);

        let result = strcpy(&mut s1, &s2);

        assert_eq!(&result[..length], vec![1; length]);
        assert_eq!(result[length], 0);
        assert_eq!(result[length + 1], 3);
    }
}

#[test]
fn strncpy_copies_exactly_n_bytes_without_adding_a_null_byte() {
    let mut s1 = i8s(b"unchanged").to_vec();

    let result = strncpy(&mut s1, i8s(b"abc"), 3);

    assert_eq!(result, i8s(b"abchanged"));
}

#[test]
fn strncpy_pads_the_destination_with_null_bytes() {
    let mut s1 = i8s(b"unchanged").to_vec();

    let result = strncpy(&mut s1, i8s(b"ab\0ignored"), 5);

    assert_eq!(result, i8s(b"ab\0\0\0nged"));
}

#[test]
fn strncpy_accepts_a_source_ending_at_its_null_byte() {
    let mut s1 = vec![1; 5];

    let result = strncpy(&mut s1, i8s(b"ab\0"), 5);

    assert_eq!(result, i8s(b"ab\0\0\0"));
}

#[test]
fn strncpy_with_zero_limit_reads_and_writes_no_bytes() {
    let mut s1 = i8s(b"unchanged").to_vec();

    let result = strncpy(&mut s1, &[], 0);

    assert_eq!(result, i8s(b"unchanged"));
}

#[test]
fn strncpy_returns_the_complete_destination_slice() {
    let mut s1 = i8s(b"old\0tail").to_vec();
    let pointer = s1.as_ptr();
    let length = s1.len();

    let result = strncpy(&mut s1, i8s(b"new\0"), 3);

    assert_eq!(result.as_ptr(), pointer);
    assert_eq!(result.len(), length);
}

#[test]
fn strncpy_finds_source_null_bytes_at_each_word_offset() {
    for length in 0..=size_of::<usize>() * 3 {
        let n = length + 3;
        let mut s1 = vec![3; n + 1];
        let mut s2 = vec![1; length];
        s2.extend_from_slice(&[0, 2]);

        let result = strncpy(&mut s1, &s2, n);

        assert_eq!(&result[..length], vec![1; length]);
        assert_eq!(&result[length..n], vec![0; n - length]);
        assert_eq!(result[n], 3);
    }
}

#[test]
fn strcat_appends_the_source_and_its_null_byte() {
    let mut s1 = i8s(b"hello\0unchanged").to_vec();

    let result = strcat(&mut s1, i8s(b" world\0ignored"));

    assert_eq!(result, i8s(b"hello world\0ged"));
}

#[test]
fn strcat_handles_empty_strings() {
    let mut empty_destination = i8s(b"\0tail").to_vec();
    assert_eq!(
        strcat(&mut empty_destination, i8s(b"abc\0")),
        i8s(b"abc\0l")
    );

    let mut empty_source = i8s(b"abc\0tail").to_vec();
    assert_eq!(
        strcat(&mut empty_source, i8s(b"\0ignored")),
        i8s(b"abc\0tail")
    );
}

#[test]
fn strcat_returns_the_complete_destination_slice() {
    let mut s1 = i8s(b"a\0tail").to_vec();
    let pointer = s1.as_ptr();
    let length = s1.len();

    let result = strcat(&mut s1, i8s(b"b\0"));

    assert_eq!(result.as_ptr(), pointer);
    assert_eq!(result.len(), length);
}

#[test]
fn strcat_finds_null_bytes_at_word_boundaries() {
    for s1_len in 0..=size_of::<usize>() * 2 {
        for s2_len in 0..=size_of::<usize>() * 2 {
            let mut s1 = vec![1; s1_len];
            s1.push(0);
            s1.resize(s1_len + s2_len + 2, 3);
            let mut s2 = vec![2; s2_len];
            s2.extend_from_slice(&[0, 4]);

            let result = strcat(&mut s1, &s2);

            assert_eq!(&result[..s1_len], vec![1; s1_len]);
            assert_eq!(&result[s1_len..s1_len + s2_len], vec![2; s2_len]);
            assert_eq!(result[s1_len + s2_len], 0);
            assert_eq!(result[s1_len + s2_len + 1], 3);
        }
    }
}

#[test]
fn strncat_appends_at_most_n_bytes_and_a_null_byte() {
    let mut s1 = i8s(b"abc\0unchanged").to_vec();

    let result = strncat(&mut s1, i8s(b"defghi\0"), 3);

    assert_eq!(result, i8s(b"abcdef\0hanged"));
}

#[test]
fn strncat_stops_at_the_source_null_byte() {
    let mut s1 = i8s(b"a\0tail").to_vec();

    let result = strncat(&mut s1, i8s(b"b\0"), usize::MAX);

    assert_eq!(result, i8s(b"ab\0ail"));
}

#[test]
fn strncat_accepts_n_bytes_without_a_source_null_byte() {
    let mut s1 = i8s(b"a\0tail").to_vec();

    let result = strncat(&mut s1, i8s(b"bc"), 2);

    assert_eq!(result, i8s(b"abc\0il"));
}

#[test]
fn strncat_with_zero_limit_only_rewrites_the_destination_null_byte() {
    let mut s1 = i8s(b"a\0tail").to_vec();

    let result = strncat(&mut s1, &[], 0);

    assert_eq!(result, i8s(b"a\0tail"));
}

#[test]
fn strncat_returns_the_complete_destination_slice() {
    let mut s1 = i8s(b"a\0tail").to_vec();
    let pointer = s1.as_ptr();
    let length = s1.len();

    let result = strncat(&mut s1, i8s(b"b\0"), 1);

    assert_eq!(result.as_ptr(), pointer);
    assert_eq!(result.len(), length);
}
