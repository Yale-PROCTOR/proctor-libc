use std::io::{self, BufRead, Read};
use std::{mem, ptr};

/// Reads the next byte from `r`.
///
/// Returns the byte as an unsigned value, `-1` at end-of-file, or an I/O error.
pub fn fgetc<R: Read + ?Sized>(r: &mut R) -> io::Result<i32> {
    let mut byte = [0];

    match r.read(&mut byte)? {
        0 => Ok(-1),
        _ => Ok(i32::from(byte[0])),
    }
}

/// Reads a line from `r` into `buf`, including the newline and a trailing null byte.
///
/// Returns `Some(buf)` when it produces a null-terminated buffer, `None` at
/// end-of-file before any input, or an I/O error. An empty buffer is invalid.
pub fn fgets<'buf, R: BufRead + ?Sized>(
    buf: &'buf mut [i8],
    r: &mut R,
) -> io::Result<Option<&'buf mut [i8]>> {
    if buf.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "buffer must have room for a null byte",
        ));
    }

    if buf.len() == 1 {
        buf[0] = 0;
        return Ok(Some(buf));
    }

    let bytes: &mut [u8] = bytemuck::cast_slice_mut(buf);
    let mut written = 0;

    while written < bytes.len() - 1 {
        let available = r.fill_buf()?;
        if available.is_empty() {
            if written == 0 {
                return Ok(None);
            }
            break;
        }

        let available_len = available.len().min(bytes.len() - 1 - written);
        let newline = available[..available_len]
            .iter()
            .position(|&byte| byte == b'\n');
        let copied = newline.map_or(available_len, |index| index + 1);

        bytes[written..written + copied].copy_from_slice(&available[..copied]);
        written += copied;
        r.consume(copied);

        if newline.is_some() {
            break;
        }
    }

    bytes[written] = 0;
    Ok(Some(buf))
}

/// Reads binary input from `r` into `buf`.
///
/// Returns the number of complete elements read and any I/O error. A short
/// count without an error means end-of-file, except for zero-sized elements.
pub fn fread<T: bytemuck::AnyBitPattern, R: BufRead + ?Sized>(
    buf: &mut [T],
    r: &mut R,
) -> (usize, io::Result<()>) {
    let element_size = mem::size_of::<T>();
    if element_size == 0 || buf.is_empty() {
        return (0, Ok(()));
    }

    let byte_len = mem::size_of_val(buf);
    let destination = buf.as_mut_ptr().cast::<u8>();
    let mut bytes_read = 0;

    while bytes_read < byte_len {
        let available = match r.fill_buf() {
            Ok([]) => break,
            Ok(available) => available,
            Err(error) => return (bytes_read / element_size, Err(error)),
        };
        let copied = available.len().min(byte_len - bytes_read);

        // SAFETY: `destination` covers all `byte_len` bytes occupied by `buf`,
        // and `copied` is limited to the unwritten portion. `AnyBitPattern`
        // guarantees that changing any subset of an element's bytes leaves a
        // valid `T`, including when the read ends with a partial element.
        unsafe {
            ptr::copy_nonoverlapping(available.as_ptr(), destination.add(bytes_read), copied);
        }

        bytes_read += copied;
        r.consume(copied);
    }

    (bytes_read / element_size, Ok(()))
}

#[cfg(test)]
mod tests;
