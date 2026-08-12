use std::mem;

/// Returns the number of bytes preceding the first null byte in `s`.
pub fn strlen(s: &[i8]) -> usize {
    const WORD_BYTES: usize = mem::size_of::<usize>();
    const LOW_BITS: usize = usize::MAX / 0xff;
    const HIGH_BITS: usize = LOW_BITS * 0x80;

    let bytes: &[u8] = bytemuck::cast_slice(s);
    let mut chunks = bytes.chunks_exact(WORD_BYTES);

    for (index, chunk) in chunks.by_ref().enumerate() {
        let word = usize::from_ne_bytes(chunk.try_into().unwrap());
        if word.wrapping_sub(LOW_BITS) & !word & HIGH_BITS != 0 {
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
