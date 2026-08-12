use std::io::{self, BufRead, BufReader, Cursor, Read};

use super::{fgetc, fgets};

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

#[test]
fn fgets_reads_through_newline_and_returns_the_entire_buffer() {
    let mut reader = Cursor::new(b"first line\nsecond line");
    let mut buf = [99; 16];

    let result = fgets(&mut buf, &mut reader).unwrap().unwrap();

    assert_eq!(result.len(), 16);
    assert_eq!(
        result,
        &[
            102, 105, 114, 115, 116, 32, 108, 105, 110, 101, 10, 0, 99, 99, 99, 99
        ]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"second line");
}

#[test]
fn fgets_stops_when_the_buffer_is_full_without_overreading() {
    let mut reader = Cursor::new(b"abcdef");
    let mut first = [99; 4];
    let mut second = [99; 4];

    assert_eq!(
        fgets(&mut first, &mut reader).unwrap().unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0]
    );
    assert_eq!(
        fgets(&mut second, &mut reader).unwrap().unwrap(),
        &[b'd' as i8, b'e' as i8, b'f' as i8, 0]
    );
}

#[test]
fn fgets_leaves_a_newline_beyond_the_buffer_limit_for_the_next_call() {
    let mut reader = Cursor::new(b"abc\ndef");
    let mut first = [99; 4];
    let mut second = [99; 4];

    assert_eq!(
        fgets(&mut first, &mut reader).unwrap().unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0]
    );
    assert_eq!(
        fgets(&mut second, &mut reader).unwrap().unwrap(),
        &[b'\n' as i8, 0, 99, 99]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"def");
}

#[test]
fn fgets_reads_across_buffered_chunks() {
    let mut reader = BufReader::with_capacity(2, Cursor::new(b"abc\ndef"));
    let mut buf = [99; 8];

    assert_eq!(
        fgets(&mut buf, &mut reader).unwrap().unwrap(),
        &[
            b'a' as i8,
            b'b' as i8,
            b'c' as i8,
            b'\n' as i8,
            0,
            99,
            99,
            99
        ]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"de");
}

#[test]
fn fgets_returns_none_at_eof_without_modifying_the_buffer() {
    let mut reader = Cursor::new([]);
    let mut buf = [99; 4];

    assert!(fgets(&mut buf, &mut reader).unwrap().is_none());
    assert_eq!(buf, [99; 4]);
}

#[test]
fn fgets_returns_data_when_eof_follows_bytes() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [99; 5];

    assert_eq!(
        fgets(&mut buf, &mut reader).unwrap().unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0, 99]
    );
}

#[test]
fn fgets_treats_null_and_high_bytes_as_input() {
    let mut reader = Cursor::new([0, 128, 255, b'\n', b'x']);
    let mut buf = [99; 6];

    assert_eq!(
        fgets(&mut buf, &mut reader).unwrap().unwrap(),
        &[0, -128, -1, b'\n' as i8, 0, 99]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"x");
}

#[test]
fn fgets_with_one_byte_buffer_writes_only_null_without_reading() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [99];

    assert_eq!(fgets(&mut buf, &mut reader).unwrap().unwrap(), &[0]);
    assert_eq!(reader.position(), 0);
}

#[test]
fn fgets_rejects_an_empty_buffer_without_reading() {
    let mut reader = Cursor::new(b"abc");
    let error = fgets(&mut [], &mut reader).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
}

#[test]
fn fgets_accepts_dynamically_sized_buffered_readers() {
    let mut bytes = Cursor::new(b"abc\n");
    let reader: &mut dyn BufRead = &mut bytes;
    let mut buf = [99; 5];

    assert_eq!(
        fgets(&mut buf, reader).unwrap().unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, b'\n' as i8, 0]
    );
}

#[test]
fn fgets_propagates_buffered_read_errors() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("read failed"))
        }

        fn consume(&mut self, _amt: usize) {}
    }

    let mut buf = [99; 4];
    let error = fgets(&mut buf, &mut FailingReader).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
}

#[test]
fn fgets_propagates_an_error_after_copying_available_bytes() {
    struct PartialThenFail {
        consumed: bool,
    }

    impl Read for PartialThenFail {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    impl BufRead for PartialThenFail {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            if self.consumed {
                Err(io::Error::other("read failed"))
            } else {
                Ok(b"ab")
            }
        }

        fn consume(&mut self, amt: usize) {
            assert_eq!(amt, 2);
            self.consumed = true;
        }
    }

    let mut reader = PartialThenFail { consumed: false };
    let mut buf = [99; 4];
    let error = fgets(&mut buf, &mut reader).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
    assert_eq!(buf, [b'a' as i8, b'b' as i8, 99, 99]);
}
