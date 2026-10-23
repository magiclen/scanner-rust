use std::io::{self, ErrorKind, Read};

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
