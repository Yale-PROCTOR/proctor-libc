use std::io::{self, Cursor, Read};

use super::fgetc;

#[test]
fn reads_bytes_as_unsigned_integers_and_advances_the_reader() {
    let mut reader = Cursor::new([0, 127, 128, 255]);

    assert_eq!(fgetc(&mut reader).unwrap(), 0);
    assert_eq!(fgetc(&mut reader).unwrap(), 127);
    assert_eq!(fgetc(&mut reader).unwrap(), 128);
    assert_eq!(fgetc(&mut reader).unwrap(), 255);
}

#[test]
fn returns_minus_one_at_end_of_file() {
    let mut reader = Cursor::new([42]);

    assert_eq!(fgetc(&mut reader).unwrap(), 42);
    assert_eq!(fgetc(&mut reader).unwrap(), -1);
    assert_eq!(fgetc(&mut reader).unwrap(), -1);
}

#[test]
fn accepts_dynamically_sized_readers() {
    let mut bytes = Cursor::new([42]);
    let reader: &mut dyn Read = &mut bytes;

    assert_eq!(fgetc(reader).unwrap(), 42);
}

#[test]
fn propagates_read_errors() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    let error = fgetc(&mut FailingReader).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
}
