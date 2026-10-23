use std::{
    cell::Cell,
    io::{self, ErrorKind, Read},
};

use scanner_rust::{Scanner, ScannerAscii, ScannerError};

struct InterruptingReader<'a> {
    data:      &'a [u8],
    interrupt: bool,
}

impl Read for InterruptingReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.interrupt = !self.interrupt;

        if self.interrupt {
            return Err(io::Error::from(ErrorKind::Interrupted));
        }

        let size = buffer.len().min(2);

        self.data.read(&mut buffer[..size])
    }
}

struct OversizedReader;

impl Read for OversizedReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        Ok(buffer.len() + 1)
    }
}

struct CountingReader<'a> {
    data:  &'a [u8],
    calls: &'a Cell<usize>,
}

impl Read for CountingReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        assert!(!buffer.is_empty());

        self.calls.set(self.calls.get() + 1);

        let size = buffer.len().min(2);

        self.data.read(&mut buffer[..size])
    }
}

#[test]
fn read_retries_interrupted() {
    let data: &[u8] = b"123\r\n456\nlast";
    let mut sc: Scanner<_, 4> = Scanner::new2(InterruptingReader {
        data,
        interrupt: false,
    });

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("456".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("last".into()), sc.next_until("|").unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let mut sc: ScannerAscii<_, 4> = ScannerAscii::new2(InterruptingReader {
        data,
        interrupt: false,
    });

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("456".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("last".into()), sc.next_until("|").unwrap());
    assert_eq!(None, sc.next_line().unwrap());
}

#[test]
fn read_rejects_oversized_length() {
    let mut sc = Scanner::new(OversizedReader);

    match sc.peek(true).unwrap_err() {
        ScannerError::IOError(err) => assert_eq!(ErrorKind::InvalidData, err.kind()),
        err => panic!("unexpected error: {err}"),
    }

    let mut sc = ScannerAscii::new(OversizedReader);

    match sc.peek(true).unwrap_err() {
        ScannerError::IOError(err) => assert_eq!(ErrorKind::InvalidData, err.kind()),
        err => panic!("unexpected error: {err}"),
    }
}

#[test]
fn read_full_buffers() {
    let data: Vec<u8> = (0..=255).cycle().take(768).collect();
    let mut sc = Scanner::new(data.as_slice());

    assert_eq!(Some(data[..512].to_vec()), sc.next_bytes(512).unwrap());
    assert_eq!(Some(data[512..].to_vec()), sc.next_bytes(512).unwrap());
    assert_eq!(None, sc.next_bytes(1).unwrap());

    let mut sc = ScannerAscii::new(data.as_slice());

    assert_eq!(Some(data[..512].to_vec()), sc.next_bytes(512).unwrap());
    assert_eq!(Some(data[512..].to_vec()), sc.next_bytes(512).unwrap());
    assert_eq!(None, sc.next_bytes(1).unwrap());
}

#[test]
fn peek_fills_available_space() {
    let calls = Cell::new(0);
    let mut sc: Scanner<_, 8> = Scanner::new2(CountingReader {
        data:  b"abcdefghij",
        calls: &calls,
    });

    assert_eq!(b"abcdefgh", sc.peek(false).unwrap());
    assert_eq!(Some(b"a".to_vec()), sc.next_bytes(1).unwrap());
    assert_eq!(b"bcdefgh", sc.peek(false).unwrap());
    assert_eq!(b"bcdefghi", sc.peek(true).unwrap());
    assert_eq!(Some(b"bcdefghi".to_vec()), sc.next_bytes(8).unwrap());
    assert_eq!(b"j", sc.peek(false).unwrap());

    let mut sc: ScannerAscii<_, 8> = ScannerAscii::new2(CountingReader {
        data:  b"abcdefghij",
        calls: &calls,
    });

    assert_eq!(b"abcdefgh", sc.peek(false).unwrap());
    assert_eq!(Some(b"a".to_vec()), sc.next_bytes(1).unwrap());
    assert_eq!(b"bcdefgh", sc.peek(false).unwrap());
    assert_eq!(b"bcdefghi", sc.peek(true).unwrap());
    assert_eq!(Some(b"bcdefghi".to_vec()), sc.next_bytes(8).unwrap());
    assert_eq!(b"j", sc.peek(false).unwrap());
}

#[test]
fn peek_stops_at_eof() {
    let calls = Cell::new(0);
    let mut sc = Scanner::new(CountingReader {
        data: b"", calls: &calls
    });

    assert_eq!(b"", sc.peek(true).unwrap());
    assert_eq!(1, calls.get());

    calls.set(0);

    let mut sc = ScannerAscii::new(CountingReader {
        data: b"", calls: &calls
    });

    assert_eq!(b"", sc.peek(true).unwrap());
    assert_eq!(1, calls.get());
}

#[test]
fn invalid_leading_byte_does_not_refill() {
    let calls = Cell::new(0);
    let mut sc = Scanner::new(CountingReader {
        data: &[0x80], calls: &calls
    });

    assert_eq!(Some('\u{FFFD}'), sc.next_char().unwrap());
    assert_eq!(1, calls.get());
}
