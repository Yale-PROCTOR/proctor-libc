use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::{mem, ptr};

/// Reads the next byte from `r`.
///
/// Returns the byte and success, `-1` and success at end-of-file, or `-1` and
/// the I/O error.
pub fn fgetc<R: Read + ?Sized>(r: &mut R) -> (i32, io::Result<()>) {
    let mut byte = [0];

    match r.read(&mut byte) {
        Ok(0) => (-1, Ok(())),
        Ok(_) => (i32::from(byte[0]), Ok(())),
        Err(error) => (-1, Err(error)),
    }
}

/// Writes `c`, converted to an unsigned byte, to `w`.
///
/// Returns the written byte and success, or `-1` and the I/O error.
pub fn fputc<W: Write + ?Sized>(c: i32, w: &mut W) -> (i32, io::Result<()>) {
    let byte = c as u8;

    match w.write(&[byte]) {
        Ok(0) => (
            -1,
            Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "failed to write byte",
            )),
        ),
        Ok(_) => (i32::from(byte), Ok(())),
        Err(error) => (-1, Err(error)),
    }
}

/// Writes the bytes in `buf` preceding the first null byte to `w`.
///
/// Returns zero and success, or `-1` and the I/O error.
pub fn fputs<W: Write + ?Sized>(buf: &[i8], w: &mut W) -> (i32, io::Result<()>) {
    let len = buf.iter().position(|&byte| byte == 0).unwrap_or(buf.len());
    let mut bytes: &[u8] = bytemuck::cast_slice(&buf[..len]);

    while !bytes.is_empty() {
        match w.write(bytes) {
            Ok(0) => {
                return (
                    -1,
                    Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write string",
                    )),
                );
            }
            Ok(written) => bytes = &bytes[written..],
            Err(error) => return (-1, Err(error)),
        }
    }

    (0, Ok(()))
}

/// Reads the next byte from standard input.
///
/// Returns the byte and success, `-1` and success at end-of-file, or `-1` and
/// the I/O error.
pub fn getchar() -> (i32, io::Result<()>) {
    let stdin = io::stdin();
    fgetc(&mut stdin.lock())
}

/// Writes `c`, converted to an unsigned byte, to standard output.
///
/// Returns the written byte and success, or `-1` and the I/O error.
pub fn putchar(c: i32) -> (i32, io::Result<()>) {
    let stdout = io::stdout();
    fputc(c, &mut stdout.lock())
}

/// Writes the bytes in `buf` preceding the first null byte, followed by a
/// newline, to standard output.
///
/// Returns zero and success, or `-1` and the I/O error.
pub fn puts(buf: &[i8]) -> (i32, io::Result<()>) {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if let Err(error) = fputs(buf, &mut stdout).1 {
        return (-1, Err(error));
    }
    if let Err(error) = fputc(i32::from(b'\n'), &mut stdout).1 {
        return (-1, Err(error));
    }

    (0, Ok(()))
}

/// Sets the position of `s` according to `pos`.
///
/// Returns zero and success, or `-1` and the seek or position-conversion error.
pub fn fseek<S: Seek + ?Sized>(s: &mut S, pos: SeekFrom) -> (i32, io::Result<()>) {
    let position = match s.seek(pos) {
        Ok(position) => position,
        Err(error) => return (-1, Err(error)),
    };
    if i64::try_from(position).is_err() {
        return (
            -1,
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stream position does not fit in i64",
            )),
        );
    }

    (0, Ok(()))
}

/// Returns the current position of `s` as a byte offset from its beginning.
///
/// Returns the offset and success, or `-1` and the seek or position-conversion
/// error.
pub fn ftell<S: Seek + ?Sized>(s: &mut S) -> (i64, io::Result<()>) {
    let position = match s.stream_position() {
        Ok(position) => position,
        Err(error) => return (-1, Err(error)),
    };

    match i64::try_from(position) {
        Ok(position) => (position, Ok(())),
        Err(_) => (
            -1,
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stream position does not fit in i64",
            )),
        ),
    }
}

/// Sets the position of `s` to its beginning.
pub fn rewind<S: Seek + ?Sized>(s: &mut S) -> io::Result<()> {
    fseek(s, SeekFrom::Start(0)).1
}

/// Reads a line from `r` into `buf`, including the newline and a trailing null byte.
///
/// Returns the buffer and success when it writes a null terminator, `None` and
/// success at end-of-file before any input, or `None` and an error.
pub fn fgets<'buf, R: BufRead + ?Sized>(
    buf: &'buf mut [i8],
    r: &mut R,
) -> (Option<&'buf mut [i8]>, io::Result<()>) {
    if buf.is_empty() {
        return (
            None,
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer must have room for a null byte",
            )),
        );
    }

    if buf.len() == 1 {
        buf[0] = 0;
        return (Some(buf), Ok(()));
    }

    let bytes: &mut [u8] = bytemuck::cast_slice_mut(buf);
    let mut written = 0;

    while written < bytes.len() - 1 {
        let available = match r.fill_buf() {
            Ok(available) => available,
            Err(error) => return (None, Err(error)),
        };
        if available.is_empty() {
            if written == 0 {
                return (None, Ok(()));
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
    (Some(buf), Ok(()))
}

/// Reads binary input from `r` into `buf`.
///
/// Returns the number of complete elements read and any I/O error. A short
/// count without an error means end-of-file.
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
