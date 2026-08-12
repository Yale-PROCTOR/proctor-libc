use std::io::{self, Read};

/// Reads the next byte from `r` as an unsigned value, or returns `-1` at EOF.
pub fn fgetc<R: Read + ?Sized>(r: &mut R) -> io::Result<i32> {
    let mut byte = [0];

    match r.read(&mut byte)? {
        0 => Ok(-1),
        _ => Ok(i32::from(byte[0])),
    }
}

#[cfg(test)]
mod tests;
