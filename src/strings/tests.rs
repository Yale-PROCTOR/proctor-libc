use super::{lowercase_word, strcasecmp, strncasecmp};

fn i8s(bytes: &[u8]) -> &[i8] {
    bytemuck::cast_slice(bytes)
}

#[test]
fn strcasecmp_compares_strings_while_ignoring_ascii_case() {
    assert_eq!(strcasecmp(i8s(b"\0"), i8s(b"\0")), 0);
    assert_eq!(strcasecmp(i8s(b"Hello\0"), i8s(b"hELLo\0")), 0);
    assert_eq!(strcasecmp(i8s(b"ABC\0"), i8s(b"abd\0")), -1);
    assert_eq!(strcasecmp(i8s(b"abE\0"), i8s(b"ABC\0")), 2);
}

#[test]
fn strcasecmp_compares_folded_bytes_as_unsigned() {
    assert_eq!(strcasecmp(&[-1, 0], &[1, 0]), 254);
    assert_eq!(strcasecmp(&[1, 0], &[-1, 0]), -254);
}

#[test]
fn strcasecmp_does_not_fold_non_ascii_bytes() {
    assert_eq!(strcasecmp(&[-64, 0], &[-32, 0]), -32);
}

#[test]
fn strcasecmp_folds_only_ascii_uppercase_bytes() {
    for byte in u8::MIN..=u8::MAX {
        let expected = if byte.is_ascii_uppercase() {
            byte | 0x20
        } else {
            byte
        };

        assert_eq!(
            strcasecmp(&[byte as i8, 0], &[expected as i8, 0]),
            0,
            "byte {byte:#04x}"
        );
    }
}

#[test]
fn lowercase_word_folds_each_byte_lane_independently() {
    const EDGE_BYTES: [u8; 8] = [0, b'A' - 1, b'A', b'Z', b'Z' + 1, 0x7f, 0x80, 0xff];

    for fill in EDGE_BYTES {
        for offset in 0..size_of::<usize>() {
            for byte in u8::MIN..=u8::MAX {
                let mut input = [fill; size_of::<usize>()];
                input[offset] = byte;
                let expected = input.map(|byte| byte.to_ascii_lowercase());

                assert_eq!(
                    lowercase_word(usize::from_ne_bytes(input)),
                    usize::from_ne_bytes(expected),
                    "fill {fill:#04x}, offset {offset}, byte {byte:#04x}"
                );
            }
        }
    }
}

#[test]
fn strcasecmp_stops_at_the_first_null_byte() {
    assert_eq!(strcasecmp(i8s(b"SaMe\0a"), i8s(b"sAmE\0b")), 0);
}

#[test]
fn strcasecmp_stops_at_a_null_byte_in_a_complete_word() {
    let word_bytes = size_of::<usize>();
    let mut s1 = vec![b'A' as i8; word_bytes + 1];
    let mut s2 = s1.clone();
    s1[word_bytes - 1] = 0;
    s2[word_bytes - 1] = 0;
    s1[word_bytes] = b'x' as i8;
    s2[word_bytes] = b'y' as i8;

    assert_eq!(strcasecmp(&s1, &s2), 0);
}

#[test]
fn strcasecmp_finds_case_insensitive_differences_at_each_word_offset() {
    for offset in 0..=size_of::<usize>() * 3 {
        let mut s1 = vec![b'A' as i8; offset];
        let mut s2 = vec![b'a' as i8; offset];
        s1.extend_from_slice(&[b'B' as i8, 0]);
        s2.extend_from_slice(&[b'c' as i8, 0]);

        assert_eq!(strcasecmp(&s1, &s2), -1);
    }
}

#[test]
fn strncasecmp_respects_the_byte_limit() {
    assert_eq!(strncasecmp(i8s(b"ABCx\0"), i8s(b"abcy\0"), 3), 0);
    assert_eq!(strncasecmp(i8s(b"ABCx\0"), i8s(b"abcy\0"), 4), -1);
}

#[test]
fn strncasecmp_handles_a_finite_word_sized_limit() {
    let word_bytes = size_of::<usize>();
    let s1 = vec![b'A' as i8; word_bytes + 1];
    let mut s2 = vec![b'a' as i8; word_bytes + 1];
    s2[word_bytes] = b'B' as i8;

    assert_eq!(strncasecmp(&s1, &s2, word_bytes), 0);
    assert_eq!(strncasecmp(&s1, &s2, word_bytes + 1), -1);
}

#[test]
fn strncasecmp_accepts_prefixes_without_null_bytes() {
    assert_eq!(strncasecmp(i8s(b"ABC"), i8s(b"abc"), 3), 0);
    assert_eq!(strncasecmp(i8s(b"ABC"), i8s(b"abd"), 3), -1);
}

#[test]
fn strncasecmp_stops_at_the_first_null_byte() {
    assert_eq!(strncasecmp(i8s(b"A\0x"), i8s(b"a\0y"), 3), 0);
}

#[test]
fn strncasecmp_with_zero_limit_compares_no_bytes() {
    assert_eq!(strncasecmp(&[], &[], 0), 0);
    assert_eq!(strncasecmp(i8s(b"a"), i8s(b"b"), 0), 0);
}
