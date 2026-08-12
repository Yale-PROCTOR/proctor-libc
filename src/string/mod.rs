use std::mem;

const WORD_BYTES: usize = mem::size_of::<usize>();
const LOW_BITS: usize = usize::MAX / 0xff;
const HIGH_BITS: usize = LOW_BITS * 0x80;

fn has_null_byte(word: usize) -> bool {
    word.wrapping_sub(LOW_BITS) & !word & HIGH_BITS != 0
}

fn compare(s1: &[i8], s2: &[i8], n: usize) -> i32 {
    let s1: &[u8] = bytemuck::cast_slice(s1);
    let s2: &[u8] = bytemuck::cast_slice(s2);
    let word_end = n.min(s1.len()).min(s2.len()) / WORD_BYTES * WORD_BYTES;
    let mut index = 0;

    while index < word_end {
        let word1 = usize::from_ne_bytes(s1[index..index + WORD_BYTES].try_into().unwrap());
        let word2 = usize::from_ne_bytes(s2[index..index + WORD_BYTES].try_into().unwrap());

        if word1 == word2 && !has_null_byte(word1) {
            index += WORD_BYTES;
            continue;
        }

        for offset in 0..WORD_BYTES {
            let byte1 = s1[index + offset];
            let byte2 = s2[index + offset];

            if byte1 != byte2 {
                return i32::from(byte1) - i32::from(byte2);
            }
            if byte1 == 0 {
                return 0;
            }
        }

        unreachable!();
    }

    while index < n {
        let byte1 = s1[index];
        let byte2 = s2[index];

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

/// Compares two null-terminated byte strings.
pub fn strcmp(s1: &[i8], s2: &[i8]) -> i32 {
    compare(s1, s2, usize::MAX)
}

/// Compares at most `n` bytes of two byte strings.
pub fn strncmp(s1: &[i8], s2: &[i8], n: usize) -> i32 {
    compare(s1, s2, n)
}

/// Returns the number of bytes preceding the first null byte in `s`.
pub fn strlen(s: &[i8]) -> usize {
    let bytes: &[u8] = bytemuck::cast_slice(s);
    let mut chunks = bytes.chunks_exact(WORD_BYTES);

    for (index, chunk) in chunks.by_ref().enumerate() {
        let word = usize::from_ne_bytes(chunk.try_into().unwrap());
        if has_null_byte(word) {
            return index * WORD_BYTES + chunk.iter().position(|&byte| byte == 0).unwrap();
        }
    }

    bytes.len() - chunks.remainder().len()
        + chunks
            .remainder()
            .iter()
            .position(|&byte| byte == 0)
            .unwrap()
}

#[cfg(test)]
mod tests;
