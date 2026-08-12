use std::io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::process::{Command, Stdio};

use super::{
    fgetc, fgets, fputc, fputs, fread, fseek, ftell, fwrite, getchar, putchar, puts, rewind,
};

const STANDARD_STREAM_CHILD: &str = "PROCTOR_LIBC_STANDARD_STREAM_CHILD";

fn unwrap_stdio<T>((value, status): (T, io::Result<()>)) -> T {
    status.unwrap();
    value
}

#[test]
fn standard_stream_child() {
    match std::env::var(STANDARD_STREAM_CHILD).as_deref() {
        Err(std::env::VarError::NotPresent) => {}
        Ok("getchar") => {
            assert_eq!(unwrap_stdio(getchar()), 0);
            assert_eq!(unwrap_stdio(getchar()), 127);
            assert_eq!(unwrap_stdio(getchar()), 128);
            assert_eq!(unwrap_stdio(getchar()), 255);
            assert_eq!(unwrap_stdio(getchar()), -1);
            assert_eq!(unwrap_stdio(getchar()), -1);
        }
        Ok("putchar") => {
            assert_eq!(unwrap_stdio(putchar(-1)), 255);
            assert_eq!(unwrap_stdio(putchar(256)), 0);
            assert_eq!(unwrap_stdio(putchar(65)), 65);
        }
        Ok("puts") => {
            assert_eq!(unwrap_stdio(puts(&[1, 2, 0, 3])), 0);
            assert_eq!(unwrap_stdio(puts(&[0])), 0);
            assert_eq!(unwrap_stdio(puts(&[-1, -128, 0])), 0);
        }
        Ok(mode) => panic!("unknown standard-stream child mode: {mode}"),
        Err(error) => panic!("invalid standard-stream child mode: {error}"),
    }
}

fn standard_stream_command(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "stdio::tests::standard_stream_child",
            "--nocapture",
        ])
        .env(STANDARD_STREAM_CHILD, mode);
    command
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn getchar_reads_unsigned_bytes_and_reports_end_of_file() {
    let mut child = standard_stream_command("getchar")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(&[0, 127, 128, 255])
        .unwrap();

    assert!(child.wait().unwrap().success());
}

#[test]
fn putchar_writes_and_returns_the_input_converted_to_an_unsigned_byte() {
    let output = standard_stream_command("putchar").output().unwrap();

    assert!(output.status.success());
    assert!(contains_subslice(&output.stdout, &[255, 0, 65]));
}

#[test]
fn puts_stops_at_null_and_appends_a_newline() {
    let output = standard_stream_command("puts").output().unwrap();

    assert!(output.status.success());
    assert!(contains_subslice(
        &output.stdout,
        &[1, 2, b'\n', b'\n', 255, 128, b'\n']
    ));
}

#[test]
fn fseek_repositions_from_the_start_current_position_and_end() {
    let mut stream = Cursor::new([0; 8]);

    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::Start(3))), 0);
    assert_eq!(stream.position(), 3);
    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::Current(2))), 0);
    assert_eq!(stream.position(), 5);
    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::End(-1))), 0);
    assert_eq!(stream.position(), 7);
}

#[test]
fn fseek_allows_positions_beyond_the_end() {
    let mut stream = Cursor::new([0; 8]);

    assert_eq!(unwrap_stdio(fseek(&mut stream, SeekFrom::Start(12))), 0);
    assert_eq!(unwrap_stdio(ftell(&mut stream)), 12);
}

#[test]
fn ftell_returns_the_current_position_without_changing_it() {
    let mut stream = Cursor::new([0; 8]);
    stream.set_position(5);

    assert_eq!(unwrap_stdio(ftell(&mut stream)), 5);
    assert_eq!(stream.position(), 5);
}

#[test]
fn rewind_sets_the_position_to_the_beginning() {
    let mut stream = Cursor::new([0; 8]);
    stream.set_position(5);

    rewind(&mut stream).unwrap();

    assert_eq!(stream.position(), 0);
}

#[test]
fn seek_functions_accept_dynamically_sized_streams() {
    let mut bytes = Cursor::new([0; 8]);
    let stream: &mut dyn Seek = &mut bytes;

    assert_eq!(unwrap_stdio(fseek(stream, SeekFrom::Start(6))), 0);
    assert_eq!(unwrap_stdio(ftell(stream)), 6);
    rewind(stream).unwrap();
    assert_eq!(unwrap_stdio(ftell(stream)), 0);
}

#[test]
fn seek_functions_propagate_seek_errors() {
    struct FailingSeek;

    impl Seek for FailingSeek {
        fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
            Err(io::Error::other("seek failed"))
        }
    }

    let (fseek_value, fseek_status) = fseek(&mut FailingSeek, SeekFrom::Start(1));
    let (ftell_value, ftell_status) = ftell(&mut FailingSeek);

    assert_eq!(fseek_value, -1);
    assert_eq!(ftell_value, -1);

    for error in [
        fseek_status.unwrap_err(),
        ftell_status.unwrap_err(),
        rewind(&mut FailingSeek).unwrap_err(),
    ] {
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(error.to_string(), "seek failed");
    }
}

#[test]
fn fseek_and_ftell_accept_i64_max_and_reject_larger_positions() {
    struct PositionSeek(u64);

    impl Seek for PositionSeek {
        fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
            Ok(self.0)
        }
    }

    let mut maximum = PositionSeek(i64::MAX as u64);
    assert_eq!(unwrap_stdio(fseek(&mut maximum, SeekFrom::Start(0))), 0);
    assert_eq!(unwrap_stdio(ftell(&mut maximum)), i64::MAX);

    let mut too_large = PositionSeek(i64::MAX as u64 + 1);
    let (fseek_value, fseek_status) = fseek(&mut too_large, SeekFrom::Start(0));
    let (ftell_value, ftell_status) = ftell(&mut too_large);

    assert_eq!(fseek_value, -1);
    assert_eq!(ftell_value, -1);

    assert_eq!(fseek_status.unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(ftell_status.unwrap_err().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn reads_bytes_as_unsigned_integers_and_advances_the_reader() {
    let mut reader = Cursor::new([0, 127, 128, 255]);

    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 0);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 127);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 128);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 255);
}

#[test]
fn returns_minus_one_at_end_of_file() {
    let mut reader = Cursor::new([42]);

    assert_eq!(unwrap_stdio(fgetc(&mut reader)), 42);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), -1);
    assert_eq!(unwrap_stdio(fgetc(&mut reader)), -1);
}

#[test]
fn accepts_dynamically_sized_readers() {
    let mut bytes = Cursor::new([42]);
    let reader: &mut dyn Read = &mut bytes;

    assert_eq!(unwrap_stdio(fgetc(reader)), 42);
}

#[test]
fn propagates_read_errors() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    let (value, status) = fgetc(&mut FailingReader);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "read failed");
}

#[test]
fn fputc_writes_bytes_and_returns_their_unsigned_values() {
    let mut writer = Vec::new();

    assert_eq!(unwrap_stdio(fputc(0, &mut writer)), 0);
    assert_eq!(unwrap_stdio(fputc(127, &mut writer)), 127);
    assert_eq!(unwrap_stdio(fputc(128, &mut writer)), 128);
    assert_eq!(unwrap_stdio(fputc(255, &mut writer)), 255);
    assert_eq!(writer, [0, 127, 128, 255]);
}

#[test]
fn fputc_converts_the_input_to_an_unsigned_byte() {
    let mut writer = Vec::new();

    assert_eq!(unwrap_stdio(fputc(-1, &mut writer)), 255);
    assert_eq!(unwrap_stdio(fputc(256, &mut writer)), 0);
    assert_eq!(unwrap_stdio(fputc(511, &mut writer)), 255);
    assert_eq!(writer, [255, 0, 255]);
}

#[test]
fn fputc_accepts_dynamically_sized_writers() {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;

    assert_eq!(unwrap_stdio(fputc(42, writer)), 42);
    assert_eq!(bytes, [42]);
}

#[test]
fn fputc_reports_a_writer_that_makes_no_progress() {
    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (value, status) = fputc(42, &mut ZeroWriter);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

#[test]
fn fputc_propagates_write_errors() {
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (value, status) = fputc(42, &mut FailingWriter);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "write failed");
}

#[test]
fn fputc_does_not_retry_an_interrupted_write() {
    struct InterruptedWriter {
        calls: usize,
    }

    impl Write for InterruptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Err(io::ErrorKind::Interrupted.into())
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = InterruptedWriter { calls: 0 };
    let (value, status) = fputc(42, &mut writer);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(writer.calls, 1);
}

#[test]
fn fputs_writes_bytes_before_the_first_null_and_returns_zero() {
    let mut writer = Vec::new();

    assert_eq!(
        unwrap_stdio(fputs(&[b'a' as i8, -128, -1, 0, b'b' as i8], &mut writer)),
        0
    );
    assert_eq!(writer, [b'a', 128, 255]);
}

#[test]
fn fputs_does_not_write_an_empty_string() {
    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!("write called for an empty string")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    assert_eq!(
        unwrap_stdio(fputs(&[0, b'a' as i8], &mut PanickingWriter)),
        0
    );
}

#[test]
fn fputs_completes_partial_writes_without_flushing() {
    struct PartialWriter {
        bytes: Vec<u8>,
        calls: usize,
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            let len = buf.len().min(2);
            self.bytes.extend_from_slice(&buf[..len]);
            Ok(len)
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let mut writer = PartialWriter {
        bytes: Vec::new(),
        calls: 0,
    };

    assert_eq!(unwrap_stdio(fputs(&[1, 2, 3, 4, 5, 0], &mut writer)), 0);
    assert_eq!(writer.bytes, [1, 2, 3, 4, 5]);
    assert_eq!(writer.calls, 3);
}

#[test]
fn fputs_reports_a_writer_that_makes_no_progress() {
    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (value, status) = fputs(&[1, 0], &mut ZeroWriter);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

#[test]
fn fputs_propagates_write_errors_after_partial_output() {
    struct FailingWriter {
        bytes: Vec<u8>,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&buf[..2]);
                Ok(2)
            } else {
                Err(io::Error::other("write failed"))
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = FailingWriter { bytes: Vec::new() };
    let (value, status) = fputs(&[1, 2, 3, 4, 0], &mut writer);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "write failed");
    assert_eq!(writer.bytes, [1, 2]);
}

#[test]
fn fputs_does_not_retry_an_interrupted_write() {
    struct InterruptedWriter {
        calls: usize,
    }

    impl Write for InterruptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Err(io::ErrorKind::Interrupted.into())
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = InterruptedWriter { calls: 0 };
    let (value, status) = fputs(&[1, 2, 0], &mut writer);
    let error = status.unwrap_err();

    assert_eq!(value, -1);
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(writer.calls, 1);
}

#[test]
fn fputs_accepts_dynamically_sized_writers() {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;

    assert_eq!(unwrap_stdio(fputs(&[1, 2, 0], writer)), 0);
    assert_eq!(bytes, [1, 2]);
}

#[test]
fn fgets_reads_through_newline_and_returns_the_entire_buffer() {
    let mut reader = Cursor::new(b"first line\nsecond line");
    let mut buf = [99; 16];

    let result = unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap();

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
        unwrap_stdio(fgets(&mut first, &mut reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0]
    );
    assert_eq!(
        unwrap_stdio(fgets(&mut second, &mut reader)).unwrap(),
        &[b'd' as i8, b'e' as i8, b'f' as i8, 0]
    );
}

#[test]
fn fgets_leaves_a_newline_beyond_the_buffer_limit_for_the_next_call() {
    let mut reader = Cursor::new(b"abc\ndef");
    let mut first = [99; 4];
    let mut second = [99; 4];

    assert_eq!(
        unwrap_stdio(fgets(&mut first, &mut reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0]
    );
    assert_eq!(
        unwrap_stdio(fgets(&mut second, &mut reader)).unwrap(),
        &[b'\n' as i8, 0, 99, 99]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"def");
}

#[test]
fn fgets_reads_across_buffered_chunks() {
    let mut reader = BufReader::with_capacity(2, Cursor::new(b"abc\ndef"));
    let mut buf = [99; 8];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(),
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

    assert!(unwrap_stdio(fgets(&mut buf, &mut reader)).is_none());
    assert_eq!(buf, [99; 4]);
}

#[test]
fn fgets_returns_data_when_eof_follows_bytes() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [99; 5];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(),
        &[b'a' as i8, b'b' as i8, b'c' as i8, 0, 99]
    );
}

#[test]
fn fgets_treats_null_and_high_bytes_as_input() {
    let mut reader = Cursor::new([0, 128, 255, b'\n', b'x']);
    let mut buf = [99; 6];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(),
        &[0, -128, -1, b'\n' as i8, 0, 99]
    );
    assert_eq!(reader.fill_buf().unwrap(), b"x");
}

#[test]
fn fgets_with_one_byte_buffer_writes_only_null_without_reading() {
    let mut reader = Cursor::new(b"abc");
    let mut buf = [99];

    assert_eq!(unwrap_stdio(fgets(&mut buf, &mut reader)).unwrap(), &[0]);
    assert_eq!(reader.position(), 0);
}

#[test]
fn fgets_accepts_dynamically_sized_buffered_readers() {
    let mut bytes = Cursor::new(b"abc\n");
    let reader: &mut dyn BufRead = &mut bytes;
    let mut buf = [99; 5];

    assert_eq!(
        unwrap_stdio(fgets(&mut buf, reader)).unwrap(),
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
    let (value, status) = fgets(&mut buf, &mut FailingReader);
    let error = status.unwrap_err();

    assert!(value.is_none());
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
    let (value, status) = fgets(&mut buf, &mut reader);
    let error = status.unwrap_err();

    assert!(value.is_none());
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

#[test]
fn fwrite_writes_complete_elements() {
    let buf = [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])];
    let mut writer = Vec::new();

    let (count, status) = fwrite(&buf, &mut writer);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(writer, [1, 2, 3, 4]);
}

#[test]
fn fwrite_accepts_no_uninit_types_that_are_not_pod() {
    let mut writer = Vec::new();

    let (count, status) = fwrite(&[true, false], &mut writer);

    assert_eq!(count, 2);
    status.unwrap();
    assert_eq!(writer, [1, 0]);
}

#[test]
fn fwrite_does_not_write_an_empty_buffer() {
    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!("write called for an empty buffer")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let (count, status) = fwrite::<u8, _>(&[], &mut PanickingWriter);

    assert_eq!(count, 0);
    status.unwrap();
}

#[test]
fn fwrite_does_not_write_zero_sized_elements() {
    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            panic!("write called for zero-sized elements")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let (count, status) = fwrite(&[(); 3], &mut PanickingWriter);

    assert_eq!(count, 0);
    status.unwrap();
}

#[test]
fn fwrite_completes_partial_writes_without_flushing() {
    struct PartialWriter {
        bytes: Vec<u8>,
        offered_lengths: Vec<usize>,
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.offered_lengths.push(buf.len());
            let len = buf.len().min(3);
            self.bytes.extend_from_slice(&buf[..len]);
            Ok(len)
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("flush called")
        }
    }

    let buf = [
        u16::from_ne_bytes([1, 2]),
        u16::from_ne_bytes([3, 4]),
        u16::from_ne_bytes([5, 6]),
    ];
    let mut writer = PartialWriter {
        bytes: Vec::new(),
        offered_lengths: Vec::new(),
    };

    let (count, status) = fwrite(&buf, &mut writer);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(writer.bytes, [1, 2, 3, 4, 5, 6]);
    assert_eq!(writer.offered_lengths, [6, 3]);
}

#[test]
fn fwrite_reports_a_writer_that_makes_no_progress() {
    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let (count, status) = fwrite(&[1_u8], &mut ZeroWriter);
    let error = status.unwrap_err();

    assert_eq!(count, 0);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
}

#[test]
fn fwrite_returns_only_complete_elements_when_writing_stalls() {
    struct PartialThenZero {
        bytes: Vec<u8>,
    }

    impl Write for PartialThenZero {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&buf[..3]);
                Ok(3)
            } else {
                Ok(0)
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let buf = [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])];
    let mut writer = PartialThenZero { bytes: Vec::new() };

    let (count, status) = fwrite(&buf, &mut writer);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::WriteZero);
    assert_eq!(writer.bytes, [1, 2, 3]);
}

#[test]
fn fwrite_returns_an_error_without_discarding_the_element_count() {
    struct PartialThenFail {
        bytes: Vec<u8>,
    }

    impl Write for PartialThenFail {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&buf[..3]);
                Ok(3)
            } else {
                Err(io::Error::other("write failed"))
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let buf = [u16::from_ne_bytes([1, 2]), u16::from_ne_bytes([3, 4])];
    let mut writer = PartialThenFail { bytes: Vec::new() };

    let (count, status) = fwrite(&buf, &mut writer);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "write failed");
    assert_eq!(writer.bytes, [1, 2, 3]);
}

#[test]
fn fwrite_does_not_retry_an_interrupted_write() {
    struct InterruptedWriter {
        calls: usize,
    }

    impl Write for InterruptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Ok(2)
            } else if self.calls == 2 {
                Err(io::ErrorKind::Interrupted.into())
            } else {
                Ok(buf.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut writer = InterruptedWriter { calls: 0 };

    let (count, status) = fwrite(&[1_u16, 2], &mut writer);
    let error = status.unwrap_err();

    assert_eq!(count, 1);
    assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(writer.calls, 2);
}

#[test]
fn fwrite_accepts_dynamically_sized_writers() {
    let mut bytes = Vec::new();
    let writer: &mut dyn Write = &mut bytes;

    let (count, status) = fwrite(&[1_u8, 2, 3], writer);

    assert_eq!(count, 3);
    status.unwrap();
    assert_eq!(bytes, [1, 2, 3]);
}
