use std::io::{self, BufRead, Read};

/// Reads the next byte from `r` as an unsigned value, or returns `-1` at EOF.
pub fn fgetc<R: Read + ?Sized>(r: &mut R) -> io::Result<i32> {
    let mut byte = [0];

    match r.read(&mut byte)? {
        0 => Ok(-1),
        _ => Ok(i32::from(byte[0])),
    }
}

/// Reads a line from `r` into `buf`, including the newline and a trailing null byte.
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

#[cfg(test)]
mod tests;
