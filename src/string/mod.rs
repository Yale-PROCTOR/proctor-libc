use std::mem;

const WORD_BYTES: usize = mem::size_of::<usize>();
const LOW_BITS: usize = usize::MAX / 0xff;
const HIGH_BITS: usize = LOW_BITS * 0x80;

fn has_null_byte(word: usize) -> bool {
    word.wrapping_sub(LOW_BITS) & !word & HIGH_BITS != 0
}

fn find_byte(s: &[i8], byte: i8) -> Option<usize> {
    let bytes: &[u8] = bytemuck::cast_slice(s);
    let byte = byte as u8;
    let repeated_byte = LOW_BITS * usize::from(byte);
    let mut chunks = bytes.chunks_exact(WORD_BYTES);

    for (index, chunk) in chunks.by_ref().enumerate() {
        let word = usize::from_ne_bytes(chunk.try_into().unwrap());
        if has_null_byte(word) || has_null_byte(word ^ repeated_byte) {
            for (offset, &current) in chunk.iter().enumerate() {
                if current == byte {
                    return Some(index * WORD_BYTES + offset);
                }
                if current == 0 {
                    return None;
                }
            }
        }
    }

    let remainder_start = bytes.len() - chunks.remainder().len();
    for (offset, &current) in chunks.remainder().iter().enumerate() {
        if current == byte {
            return Some(remainder_start + offset);
        }
        if current == 0 {
            return None;
        }
    }

    None
}

fn rfind_byte(s: &[i8], byte: i8) -> Option<usize> {
    let bytes: &[u8] = bytemuck::cast_slice(s);
    let byte = byte as u8;
    let repeated_byte = LOW_BITS * usize::from(byte);
    let mut chunks = bytes.chunks_exact(WORD_BYTES);
    let mut last = None;

    for (index, chunk) in chunks.by_ref().enumerate() {
        let word = usize::from_ne_bytes(chunk.try_into().unwrap());
        if has_null_byte(word) || has_null_byte(word ^ repeated_byte) {
            for (offset, &current) in chunk.iter().enumerate() {
                if current == byte {
                    last = Some(index * WORD_BYTES + offset);
                }
                if current == 0 {
                    return last;
                }
            }
        }
    }

    let remainder_start = bytes.len() - chunks.remainder().len();
    for (offset, &current) in chunks.remainder().iter().enumerate() {
        if current == byte {
            last = Some(remainder_start + offset);
        }
        if current == 0 {
            return last;
        }
    }

    last
}

fn find_null(s: &[i8], n: usize) -> Option<usize> {
    let bytes: &[u8] = bytemuck::cast_slice(s);
    let bytes = &bytes[..n.min(bytes.len())];
    let mut chunks = bytes.chunks_exact(WORD_BYTES);

    for (index, chunk) in chunks.by_ref().enumerate() {
        let word = usize::from_ne_bytes(chunk.try_into().unwrap());
        if has_null_byte(word) {
            return Some(index * WORD_BYTES + chunk.iter().position(|&byte| byte == 0).unwrap());
        }
    }

    let remainder_start = bytes.len() - chunks.remainder().len();
    chunks
        .remainder()
        .iter()
        .position(|&byte| byte == 0)
        .map(|index| remainder_start + index)
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

/// Copies the null-terminated byte string `s2`, including its null byte, into `s1`.
pub fn strcpy<'s>(s1: &'s mut [i8], s2: &[i8]) -> &'s mut [i8] {
    let s2_len = strlen(s2);
    s1[..=s2_len].copy_from_slice(&s2[..=s2_len]);
    s1
}

/// Copies at most `n` bytes from `s2` into `s1`, padding with null bytes when needed.
pub fn strncpy<'s>(s1: &'s mut [i8], s2: &[i8], n: usize) -> &'s mut [i8] {
    let copied = find_null(s2, n).unwrap_or(n);
    s1[..copied].copy_from_slice(&s2[..copied]);
    s1[copied..n].fill(0);
    s1
}

/// Appends the null-terminated byte string `s2` to `s1`.
pub fn strcat<'s>(s1: &'s mut [i8], s2: &[i8]) -> &'s mut [i8] {
    let s1_len = strlen(s1);
    let s2_len = strlen(s2);
    s1[s1_len..s1_len + s2_len + 1].copy_from_slice(&s2[..=s2_len]);
    s1
}

/// Appends at most `n` bytes from `s2` to the null-terminated byte string `s1`.
pub fn strncat<'s>(s1: &'s mut [i8], s2: &[i8], n: usize) -> &'s mut [i8] {
    let s1_len = strlen(s1);
    let s2_len = find_null(s2, n).unwrap_or(n);

    s1[s1_len..s1_len + s2_len].copy_from_slice(&s2[..s2_len]);
    s1[s1_len + s2_len] = 0;
    s1
}

/// Returns the number of bytes preceding the first null byte in `s`.
pub fn strlen(s: &[i8]) -> usize {
    find_null(s, s.len()).unwrap()
}

/// Finds `c`, converted to `i8`, in null-terminated `s` and returns its suffix, or `None` if not found.
pub fn strchr(s: &[i8], c: i32) -> Option<&[i8]> {
    find_byte(s, c as i8).map(|index| &s[index..])
}

/// Finds `c`, converted to `i8`, in null-terminated `s` and returns its mutable suffix, or `None` if not found.
pub fn strchr_mut(s: &mut [i8], c: i32) -> Option<&mut [i8]> {
    find_byte(s, c as i8).map(|index| &mut s[index..])
}

/// Finds the last `c`, converted to `i8`, in null-terminated `s` and returns its suffix, or `None` if not found.
pub fn strrchr(s: &[i8], c: i32) -> Option<&[i8]> {
    rfind_byte(s, c as i8).map(|index| &s[index..])
}

/// Finds the last `c`, converted to `i8`, in null-terminated `s` and returns its mutable suffix, or `None` if not found.
pub fn strrchr_mut(s: &mut [i8], c: i32) -> Option<&mut [i8]> {
    rfind_byte(s, c as i8).map(|index| &mut s[index..])
}

fn find_substring(s1: &[i8], s2: &[i8]) -> Option<usize> {
    let s1_len = strlen(s1);
    let s2_len = strlen(s2);

    if s2_len == 0 {
        return Some(0);
    }
    if s2_len > s1_len {
        return None;
    }

    let first = s2[0];
    let last_start = s1_len - s2_len;
    let mut start = 0;

    while start <= last_start {
        start += find_byte(&s1[start..=last_start], first)?;
        if s1[start..start + s2_len] == s2[..s2_len] {
            return Some(start);
        }
        start += 1;
    }

    None
}

/// Finds null-terminated `s2` in null-terminated `s1` and returns its suffix, or `None` if not found.
pub fn strstr<'s>(s1: &'s [i8], s2: &[i8]) -> Option<&'s [i8]> {
    find_substring(s1, s2).map(|index| &s1[index..])
}

/// Finds null-terminated `s2` in null-terminated `s1` and returns its mutable suffix, or `None` if not found.
pub fn strstr_mut<'s>(s1: &'s mut [i8], s2: &[i8]) -> Option<&'s mut [i8]> {
    find_substring(s1, s2).map(|index| &mut s1[index..])
}

#[cfg(test)]
mod tests;
