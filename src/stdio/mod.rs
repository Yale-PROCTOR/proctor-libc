use std::io::{self, BufRead, Read, Write};
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

/// Writes `c`, converted to an unsigned byte, to `w`.
///
/// Returns the written byte as an unsigned value or an I/O error.
pub fn fputc<W: Write + ?Sized>(c: i32, w: &mut W) -> io::Result<i32> {
    let byte = c as u8;

    if w.write(&[byte])? == 0 {
        return Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "failed to write byte",
        ));
    }

    Ok(i32::from(byte))
}

/// Writes the bytes in `buf` preceding the first null byte to `w`.
///
/// If `buf` has no null byte, writes the entire slice. Returns zero on success
/// or an I/O error.
pub fn fputs<W: Write + ?Sized>(buf: &[i8], w: &mut W) -> io::Result<i32> {
    let len = buf.iter().position(|&byte| byte == 0).unwrap_or(buf.len());
    let mut bytes: &[u8] = bytemuck::cast_slice(&buf[..len]);

    while !bytes.is_empty() {
        match w.write(bytes)? {
            0 => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write string",
                ));
            }
            written => bytes = &bytes[written..],
        }
    }

    Ok(0)
}

/// Reads the next byte from standard input.
///
/// Returns the byte as an unsigned value, `-1` at end-of-file, or an I/O error.
pub fn getchar() -> io::Result<i32> {
    let stdin = io::stdin();
    fgetc(&mut stdin.lock())
}

/// Writes `c`, converted to an unsigned byte, to standard output.
///
/// Returns the written byte as an unsigned value or an I/O error.
pub fn putchar(c: i32) -> io::Result<i32> {
    let stdout = io::stdout();
    fputc(c, &mut stdout.lock())
}

/// Writes the bytes in `buf` preceding the first null byte, followed by a
/// newline, to standard output.
///
/// If `buf` has no null byte, writes the entire slice. Returns zero on success
/// or an I/O error.
pub fn puts(buf: &[i8]) -> io::Result<i32> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    fputs(buf, &mut stdout)?;
    fputc(i32::from(b'\n'), &mut stdout)?;

    Ok(0)
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

/// Writes binary output from `buf` to `w`.
///
/// Returns the number of complete elements written and any I/O error.
pub fn fwrite<T: bytemuck::NoUninit, W: Write + ?Sized>(
    buf: &[T],
    w: &mut W,
) -> (usize, io::Result<()>) {
    let element_size = mem::size_of::<T>();
    if element_size == 0 || buf.is_empty() {
        return (0, Ok(()));
    }

    let bytes: &[u8] = bytemuck::cast_slice(buf);
    let mut bytes_written = 0;

    while bytes_written < bytes.len() {
        match w.write(&bytes[bytes_written..]) {
            Ok(0) => {
                return (
                    bytes_written / element_size,
                    Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write buffer",
                    )),
                );
            }
            Ok(written) => bytes_written += written,
            Err(error) => return (bytes_written / element_size, Err(error)),
        }
    }

    (buf.len(), Ok(()))
}

#[cfg(test)]
mod tests;
