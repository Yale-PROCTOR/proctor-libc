use super::{
    memchr, memchr_mut, memcmp, strcat, strchr, strchr_mut, strcmp, strcpy, strcspn, strdup,
    strlen, strncat, strncmp, strncpy, strndup, strrchr, strrchr_mut, strspn, strstr, strstr_mut,
};

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
fn strdup_copies_through_the_first_null_byte() {
    assert_eq!(&*strdup(i8s(b"hello\0ignored")), i8s(b"hello\0"));
}

#[test]
fn strdup_handles_an_empty_string() {
    assert_eq!(&*strdup(i8s(b"\0ignored")), i8s(b"\0"));
}

#[test]
fn strdup_finds_null_bytes_at_each_word_offset() {
    for length in 0..=size_of::<usize>() * 3 {
        let mut s = vec![-1; length];
        s.extend_from_slice(&[0, 2]);

        let result = strdup(&s);

        assert_eq!(&result[..length], vec![-1; length]);
        assert_eq!(result[length], 0);
        assert_eq!(result.len(), length + 1);
    }
}

#[test]
fn strndup_copies_at_most_n_bytes_and_appends_a_null_byte() {
    assert_eq!(&*strndup(i8s(b"abcdef"), 3), i8s(b"abc\0"));
}

#[test]
fn strndup_stops_at_the_first_null_byte() {
    assert_eq!(&*strndup(i8s(b"abc\0ignored"), usize::MAX), i8s(b"abc\0"));
}

#[test]
fn strndup_accepts_n_bytes_without_a_null_byte() {
    assert_eq!(&*strndup(i8s(b"abc"), 3), i8s(b"abc\0"));
}

#[test]
fn strndup_with_zero_limit_reads_no_bytes() {
    assert_eq!(&*strndup(&[], 0), i8s(b"\0"));
}

#[test]
fn strndup_finds_null_bytes_at_each_word_offset() {
    for length in 0..=size_of::<usize>() * 3 {
        let mut s = vec![-1; length];
        s.extend_from_slice(&[0, 2]);

        let result = strndup(&s, usize::MAX);

        assert_eq!(&result[..length], vec![-1; length]);
        assert_eq!(result[length], 0);
        assert_eq!(result.len(), length + 1);
    }
}

#[test]
fn memchr_returns_the_suffix_at_the_first_match() {
    assert_eq!(memchr(b"abca", b'a'.into()), Some(&b"abca"[..]));
    assert_eq!(memchr(b"abca", b'c'.into()), Some(&b"ca"[..]));
}

#[test]
fn memchr_returns_none_when_the_byte_is_not_found() {
    assert_eq!(memchr(b"abc", b'd'.into()), None);
    assert_eq!(memchr(b"", 0), None);
}

#[test]
fn memchr_searches_null_bytes_and_bytes_after_them() {
    assert_eq!(memchr(b"ab\0cd", 0), Some(&b"\0cd"[..]));
    assert_eq!(memchr(b"ab\0cd", b'd'.into()), Some(&b"d"[..]));
}

#[test]
fn memchr_converts_c_to_u8() {
    assert_eq!(memchr(&[255, 1], -1), Some(&[255, 1][..]));
    assert_eq!(memchr(&[1, 2], 257), Some(&[1, 2][..]));
}

#[test]
fn memchr_finds_bytes_at_each_word_offset() {
    for expected in 0..=size_of::<usize>() * 3 {
        let mut buf = vec![1; expected];
        buf.extend_from_slice(&[2, 3]);

        let result = memchr(&buf, 2).unwrap();

        assert_eq!(result.as_ptr(), buf[expected..].as_ptr());
        assert_eq!(result, &buf[expected..]);
    }
}

#[test]
fn memchr_returns_none_for_complete_words_without_a_match() {
    let buf = vec![1; size_of::<usize>() * 3];

    assert_eq!(memchr(&buf, 2), None);
}

#[test]
fn memchr_mut_returns_a_mutable_suffix() {
    let mut buf = b"abc".to_vec();

    let result = memchr_mut(&mut buf, b'b'.into()).unwrap();
    result[0] = b'B';

    assert_eq!(buf, b"aBc");
}

#[test]
fn memchr_mut_returns_none_without_modifying_buf() {
    let mut buf = b"abc".to_vec();

    assert_eq!(memchr_mut(&mut buf, b'd'.into()), None);
    assert_eq!(buf, b"abc");
}

#[test]
fn memcmp_compares_bytes() {
    assert_eq!(memcmp(b"", b"", 0), 0);
    assert_eq!(memcmp(b"hello", b"hello", 5), 0);
    assert_eq!(memcmp(b"abc", b"abd", 3), -1);
    assert_eq!(memcmp(b"abe", b"abc", 3), 2);
}

#[test]
fn memcmp_respects_the_byte_limit() {
    assert_eq!(memcmp(b"abcx", b"abcy", 3), 0);
    assert_eq!(memcmp(b"abcx", b"abcy", 4), -1);
}

#[test]
fn memcmp_compares_bytes_as_unsigned() {
    assert_eq!(memcmp(&[255], &[1], 1), 254);
    assert_eq!(memcmp(&[1], &[255], 1), -254);
}

#[test]
fn memcmp_compares_bytes_after_null_bytes() {
    assert_eq!(memcmp(&[1, 0, 2], &[1, 0, 3], 3), -1);
}

#[test]
fn memcmp_uses_the_first_differing_byte() {
    let mut buf1 = vec![0; size_of::<usize>()];
    let mut buf2 = buf1.clone();
    buf1[0] = 1;
    buf2[0] = 2;
    buf1[size_of::<usize>() - 1] = 255;

    assert!(memcmp(&buf1, &buf2, buf1.len()) < 0);
}

#[test]
fn memcmp_finds_differences_at_each_word_offset() {
    for offset in 0..=size_of::<usize>() * 3 {
        let mut buf1 = vec![1; offset];
        let mut buf2 = buf1.clone();
        buf1.push(2);
        buf2.push(3);

        assert_eq!(memcmp(&buf1, &buf2, offset + 1), -1);
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

#[test]
fn strchr_returns_the_suffix_at_the_first_match() {
    assert_eq!(
        strchr(i8s(b"abca\0tail"), b'a'.into()),
        Some(i8s(b"abca\0tail"))
    );
    assert_eq!(
        strchr(i8s(b"abca\0tail"), b'c'.into()),
        Some(i8s(b"ca\0tail"))
    );
}

#[test]
fn strchr_does_not_search_after_the_null_byte() {
    assert_eq!(strchr(i8s(b"abc\0d"), b'd'.into()), None);
}

#[test]
fn strchr_stops_at_a_null_byte_before_a_match_in_a_complete_word() {
    let mut s = vec![1; size_of::<usize>()];
    s[0] = 0;
    s[1] = 2;

    assert_eq!(strchr(&s, 2), None);
}

#[test]
fn strchr_can_find_the_null_byte() {
    assert_eq!(strchr(i8s(b"abc\0tail"), 0), Some(i8s(b"\0tail")));
}

#[test]
fn strchr_converts_c_to_i8() {
    assert_eq!(strchr(&[-1, 0], 255), Some(&[-1, 0][..]));
    assert_eq!(strchr(&[1, 0], 257), Some(&[1, 0][..]));
}

#[test]
fn strchr_finds_bytes_at_each_word_offset() {
    for expected in 0..=size_of::<usize>() * 3 {
        let mut s = vec![1; expected];
        s.extend_from_slice(&[2, 0, 3]);

        let result = strchr(&s, 2).unwrap();

        assert_eq!(result.as_ptr(), s[expected..].as_ptr());
        assert_eq!(result, &s[expected..]);
    }
}

#[test]
fn strchr_mut_returns_a_mutable_suffix() {
    let mut s = i8s(b"abc\0tail").to_vec();

    let result = strchr_mut(&mut s, b'b'.into()).unwrap();
    result[0] = b'B' as i8;

    assert_eq!(s, i8s(b"aBc\0tail"));
}

#[test]
fn strchr_mut_returns_none_without_modifying_s() {
    let mut s = i8s(b"abc\0tail").to_vec();

    assert_eq!(strchr_mut(&mut s, b'd'.into()), None);
    assert_eq!(s, i8s(b"abc\0tail"));
}

#[test]
fn strchr_mut_can_find_the_null_byte() {
    let mut s = i8s(b"abc\0tail").to_vec();

    let result = strchr_mut(&mut s, 0).unwrap();
    result[1] = b'T' as i8;

    assert_eq!(s, i8s(b"abc\0Tail"));
}

#[test]
fn strrchr_returns_the_suffix_at_the_last_match() {
    assert_eq!(
        strrchr(i8s(b"abca\0tail"), b'a'.into()),
        Some(i8s(b"a\0tail"))
    );
    assert_eq!(
        strrchr(i8s(b"abca\0tail"), b'b'.into()),
        Some(i8s(b"bca\0tail"))
    );
}

#[test]
fn strrchr_does_not_search_after_the_null_byte() {
    let mut s = vec![1; size_of::<usize>()];
    s[0] = 0;
    s[1] = 2;

    assert_eq!(strrchr(&s, 2), None);
}

#[test]
fn strrchr_can_find_the_null_byte() {
    assert_eq!(strrchr(i8s(b"abc\0tail\0"), 0), Some(i8s(b"\0tail\0")));
}

#[test]
fn strrchr_converts_c_to_i8() {
    assert_eq!(strrchr(&[-1, 1, -1, 0], 255), Some(&[-1, 0][..]));
    assert_eq!(strrchr(&[1, 2, 1, 0], 257), Some(&[1, 0][..]));
}

#[test]
fn strrchr_finds_bytes_at_each_word_offset() {
    for expected in 0..=size_of::<usize>() * 3 {
        let mut s = vec![1; expected];
        s.extend_from_slice(&[2, 0, 3]);

        let result = strrchr(&s, 2).unwrap();

        assert_eq!(result.as_ptr(), s[expected..].as_ptr());
        assert_eq!(result, &s[expected..]);
    }
}

#[test]
fn strrchr_returns_none_when_the_byte_is_not_found() {
    assert_eq!(strrchr(i8s(b"abc\0tail"), b'd'.into()), None);
}

#[test]
fn strrchr_mut_returns_a_mutable_suffix() {
    let mut s = i8s(b"abcabc\0tail").to_vec();

    let result = strrchr_mut(&mut s, b'b'.into()).unwrap();
    result[0] = b'B' as i8;

    assert_eq!(s, i8s(b"abcaBc\0tail"));
}

#[test]
fn strrchr_mut_handles_null_and_missing_bytes() {
    let mut null_target = i8s(b"abc\0tail").to_vec();
    let result = strrchr_mut(&mut null_target, 0).unwrap();
    result[1] = b'T' as i8;
    assert_eq!(null_target, i8s(b"abc\0Tail"));

    let mut missing = i8s(b"abc\0tail").to_vec();
    assert_eq!(strrchr_mut(&mut missing, b'd'.into()), None);
    assert_eq!(missing, i8s(b"abc\0tail"));
}

#[test]
fn strspn_returns_the_length_of_the_initial_matching_segment() {
    assert_eq!(strspn(i8s(b"abcde\0tail"), i8s(b"cba\0ignored")), 3);
    assert_eq!(strspn(i8s(b"abc\0tail"), i8s(b"abc\0ignored")), 3);
    assert_eq!(strspn(i8s(b"abc\0tail"), i8s(b"xyz\0ignored")), 0);
}

#[test]
fn strspn_handles_empty_strings_and_sets() {
    assert_eq!(strspn(i8s(b"\0tail"), i8s(b"abc\0")), 0);
    assert_eq!(strspn(i8s(b"abc\0tail"), i8s(b"\0ignored")), 0);
}

#[test]
fn strspn_ignores_duplicates_and_bytes_after_null_bytes() {
    assert_eq!(strspn(i8s(b"aab\0c"), i8s(b"aa\0b")), 2);
}

#[test]
fn strspn_handles_bytes_with_the_high_bit_set() {
    assert_eq!(strspn(&[-1, -128, 1, 0], &[-128, -1, 0]), 2);
}

#[test]
fn strspn_handles_initial_segments_at_each_word_offset() {
    for expected in 0..=size_of::<usize>() * 3 {
        let mut s1 = vec![1; expected];
        s1.extend_from_slice(&[0, 2]);

        assert_eq!(strspn(&s1, &[1, 0]), expected);
    }
}

#[test]
fn strcspn_returns_the_length_of_the_initial_nonmatching_segment() {
    assert_eq!(strcspn(i8s(b"abcde\0tail"), i8s(b"dx\0ignored")), 3);
    assert_eq!(strcspn(i8s(b"abc\0tail"), i8s(b"xyz\0ignored")), 3);
    assert_eq!(strcspn(i8s(b"abc\0tail"), i8s(b"cba\0ignored")), 0);
}

#[test]
fn strcspn_handles_empty_strings_and_sets() {
    assert_eq!(strcspn(i8s(b"\0tail"), i8s(b"abc\0")), 0);
    assert_eq!(strcspn(i8s(b"abc\0tail"), i8s(b"\0ignored")), 3);
}

#[test]
fn strcspn_ignores_duplicates_and_bytes_after_null_bytes() {
    assert_eq!(strcspn(i8s(b"abc\0d"), i8s(b"xx\0b")), 3);
}

#[test]
fn strcspn_handles_bytes_with_the_high_bit_set() {
    assert_eq!(strcspn(&[-1, -128, 1, 0], &[-128, 0]), 1);
}

#[test]
fn span_functions_classify_every_non_null_byte() {
    for byte in 1_u8..=u8::MAX {
        let s = [byte as i8, 0];

        assert_eq!(strspn(&s, &s), 1);
        assert_eq!(strcspn(&s, &s), 0);
    }
}

#[test]
fn strcspn_handles_initial_segments_at_each_word_offset() {
    for expected in 0..=size_of::<usize>() * 3 {
        let mut s1 = vec![1; expected];
        s1.extend_from_slice(&[0, 2]);

        assert_eq!(strcspn(&s1, &[2, 0]), expected);
    }
}

#[test]
fn strstr_returns_the_suffix_at_the_first_match() {
    assert_eq!(
        strstr(i8s(b"abcabc\0tail"), i8s(b"bca\0ignored")),
        Some(i8s(b"bcabc\0tail"))
    );
}

#[test]
fn strstr_returns_none_when_the_needle_is_not_found() {
    assert_eq!(strstr(i8s(b"abc\0"), i8s(b"abd\0")), None);
    assert_eq!(strstr(i8s(b"abc\0"), i8s(b"abcd\0")), None);
}

#[test]
fn strstr_does_not_search_after_null_bytes() {
    assert_eq!(strstr(i8s(b"abc\0def\0"), i8s(b"def\0")), None);
    assert_eq!(
        strstr(i8s(b"abcdef\0"), i8s(b"cd\0wrong")),
        Some(i8s(b"cdef\0"))
    );
}

#[test]
fn strstr_with_an_empty_needle_returns_s1() {
    let s1 = i8s(b"abc\0tail");

    let result = strstr(s1, i8s(b"\0ignored")).unwrap();

    assert_eq!(result.as_ptr(), s1.as_ptr());
    assert_eq!(result.len(), s1.len());
}

#[test]
fn strstr_handles_an_empty_haystack() {
    assert_eq!(strstr(i8s(b"\0tail"), i8s(b"a\0")), None);
    assert_eq!(
        strstr(i8s(b"\0tail"), i8s(b"\0ignored")),
        Some(i8s(b"\0tail"))
    );
}

#[test]
fn strstr_matches_high_bit_bytes() {
    assert_eq!(strstr(&[1, -1, 2, 0], &[-1, 2, 0]), Some(&[-1, 2, 0][..]));
}

#[test]
fn strstr_finds_matches_at_each_word_offset() {
    for expected in 0..=size_of::<usize>() * 3 {
        let mut s1 = vec![1; expected];
        s1.extend_from_slice(&[2, 3, 0, 4]);

        let result = strstr(&s1, &[2, 3, 0]).unwrap();

        assert_eq!(result.as_ptr(), s1[expected..].as_ptr());
        assert_eq!(result, &s1[expected..]);
    }
}

#[test]
fn strstr_checks_later_candidates_after_partial_matches() {
    assert_eq!(
        strstr(i8s(b"aaab\0tail"), i8s(b"aab\0")),
        Some(i8s(b"aab\0tail"))
    );
}

#[test]
fn strstr_mut_returns_a_mutable_suffix() {
    let mut s1 = i8s(b"abcabc\0tail").to_vec();

    let result = strstr_mut(&mut s1, i8s(b"cab\0")).unwrap();
    result[0] = b'C' as i8;

    assert_eq!(s1, i8s(b"abCabc\0tail"));
}

#[test]
fn strstr_mut_handles_empty_and_missing_needles() {
    let mut empty_needle_haystack = i8s(b"abc\0tail").to_vec();
    let pointer = empty_needle_haystack.as_ptr();
    let length = empty_needle_haystack.len();

    let result = strstr_mut(&mut empty_needle_haystack, i8s(b"\0ignored")).unwrap();
    assert_eq!(result.as_ptr(), pointer);
    assert_eq!(result.len(), length);

    let mut missing_needle_haystack = i8s(b"abc\0tail").to_vec();
    assert_eq!(strstr_mut(&mut missing_needle_haystack, i8s(b"d\0")), None);
    assert_eq!(missing_needle_haystack, i8s(b"abc\0tail"));
}
