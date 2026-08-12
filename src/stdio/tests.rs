use std::io::{self, BufRead, BufReader, Cursor, Read};

use super::{fgetc, fgets, fread};

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

#[test]
fn fread_reads_complete_elements() {
    let mut reader = Cursor::new([1, 2, 3, 4]);
    let mut buf = [0_u16; 2];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(
        buf,
        [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])]
    );
}

#[test]
fn fread_does_not_read_past_the_destination() {
    let mut reader = Cursor::new(b"abcde");
    let mut buf = [0_u8; 3];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(buf, *b"abc");
    assert_eq!(reader.fill_buf().unwrap(), b"de");
}

#[test]
fn fread_returns_only_complete_elements_at_end_of_file() {
    let mut reader = Cursor::new([1, 2, 3]);
    let mut buf = [u16::from_ne_bytes([9, 9]); 2];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 1);
    status.unwrap();
    assert_eq!(bytemuck::cast_slice::<_, u8>(&buf), &[1, 2, 3, 9]);
}

#[test]
fn fread_reads_elements_across_buffered_chunks() {
    let mut reader = BufReader::with_capacity(3, Cursor::new([1, 2, 3, 4, 5, 6, 7, 8]));
    let mut buf = [0_u32; 2];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(
        buf,
        [
            u32::from_ne_bytes([1, 2, 3, 4]),
            u32::from_ne_bytes([5, 6, 7, 8])
        ]
    );
}

#[test]
fn fread_does_not_read_for_an_empty_buffer() {
    let mut reader = Cursor::new(b"abc");

    let (count, status) = fread::<u8, _>(&mut [], &mut reader);

    assert_eq!(count, 0);
    status.unwrap();
    assert_eq!(reader.position(), 0);
}

#[test]
fn fread_does_not_read_zero_sized_elements() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [(); 3];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 0);
    status.unwrap();
    assert_eq!(reader.position(), 0);
}

#[test]
fn fread_accepts_dynamically_sized_buffered_readers() {
    let mut bytes = Cursor::new(b"abc");
    let reader: &mut dyn BufRead = &mut bytes;
    let mut buf = [0_u8; 3];

    let (count, status) = fread(&mut buf, reader);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(buf, *b"abc");
}

#[test]
fn fread_propagates_an_error_before_reading_any_bytes() {
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

    let mut buf = [9_u8; 2];

    let (count, status) = fread(&mut buf, &mut FailingReader);
    let error = status.unwrap_err();

    assert_eq!(count, 0);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
    assert_eq!(buf, [9; 2]);
}

#[test]
fn fread_returns_an_error_without_discarding_the_element_count() {
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
                Ok(b"abc")
            }
        }

        fn consume(&mut self, amt: usize) {
            assert_eq!(amt, 3);
            self.consumed = true;
        }
    }

    let mut reader = PartialThenFail { consumed: false };
    let mut buf = [u16::from_ne_bytes([9, 9]); 2];

    let (count, status) = fread(&mut buf, &mut reader);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
    assert_eq!(bytemuck::cast_slice::<_, u8>(&buf), &[b'a', b'b', b'c', 9]);
}

#[test]
fn fread_accepts_any_bit_pattern_types_with_padding() {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Padded {
        byte: u8,
        number: u16,
    }

    // SAFETY: All bit patterns, including zero, are valid for both fields, and
    // the type contains no pointers or interior mutability.
    unsafe impl bytemuck::Zeroable for Padded {}
    unsafe impl bytemuck::AnyBitPattern for Padded {}

    let mut reader = Cursor::new([1, 2, 3, 4]);
    let mut buf = [Padded { byte: 0, number: 0 }];

    let (count, status) = fread(&mut buf, &mut reader);

    assert_eq!(count, 1);
    status.unwrap();
    assert_eq!(buf[0].byte, 1);
    assert_eq!(buf[0].number, u16::from_ne_bytes([3, 4]));
}
