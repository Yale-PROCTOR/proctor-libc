//! Safe equivalents of functions declared in C's `strings.h` header.

use std::mem;

const WORD_BYTES: usize = mem::size_of::<usize>();
const LOW_BITS: usize = usize::MAX / 0xff;
const HIGH_BITS: usize = LOW_BITS * 0x80;

fn has_null_byte(word: usize) -> bool {
    word.wrapping_sub(LOW_BITS) & !word & HIGH_BITS != 0
}

fn lowercase(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte | 0x20
    } else {
        byte
    }
}

fn bytes_less_than(word: usize, byte: u8) -> usize {
    let byte = LOW_BITS * usize::from(byte);
    !word & !(word | HIGH_BITS).wrapping_sub(byte) & HIGH_BITS
}

fn lowercase_word(word: usize) -> usize {
    let below_a = bytes_less_than(word, b'A');
    let at_most_z = bytes_less_than(word, b'Z' + 1);
    let uppercase = at_most_z & !below_a;

    word | (uppercase >> 2)
}

fn compare(s1: &[i8], s2: &[i8], n: usize) -> i32 {
    let s1: &[u8] = bytemuck::cast_slice(s1);
    let s2: &[u8] = bytemuck::cast_slice(s2);
    let word_end = n.min(s1.len()).min(s2.len()) / WORD_BYTES * WORD_BYTES;
    let mut index = 0;

    while index < word_end {
        let word1 = usize::from_ne_bytes(s1[index..index + WORD_BYTES].try_into().unwrap());
        let word2 = usize::from_ne_bytes(s2[index..index + WORD_BYTES].try_into().unwrap());

        if lowercase_word(word1) == lowercase_word(word2) && !has_null_byte(word1) {
            index += WORD_BYTES;
            continue;
        }

        for offset in 0..WORD_BYTES {
            let byte1 = lowercase(s1[index + offset]);
            let byte2 = lowercase(s2[index + offset]);

            if byte1 != byte2 {
                return i32::from(byte1) - i32::from(byte2);
            }
            if byte1 == 0 {
                return 0;
            }
        }

        index += WORD_BYTES;
    }

    while index < n {
        let byte1 = lowercase(s1[index]);
        let byte2 = lowercase(s2[index]);

        if byte1 != byte2 {
            return i32::from(byte1) - i32::from(byte2);
        }
        if byte1 == 0 {
            return 0;
        }

        index += 1;
    }

    0
}

/// Compares two null-terminated byte strings while ignoring ASCII case.
pub fn strcasecmp(s1: &[i8], s2: &[i8]) -> i32 {
    compare(s1, s2, usize::MAX)
}

/// Compares at most `n` bytes of two byte strings while ignoring ASCII case.
pub fn strncasecmp(s1: &[i8], s2: &[i8], n: usize) -> i32 {
    compare(s1, s2, n)
}

#[cfg(test)]
mod tests;
