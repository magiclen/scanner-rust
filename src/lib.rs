#![cfg_attr(feature = "nightly", feature(str_internals))]

extern crate regex;

mod utf8;

use std::io::{self, Read, Cursor};
use std::path::Path;
use std::fs::File;
use std::ptr::copy;

use self::utf8::*;

const DEFAULT_RADIX: u8 = 10;
const DEFAULT_BUFFER_SIZE: usize = 1024; // must be equal to or bigger than 4

pub struct Scanner<R: Read> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
    radix: u8,
}

impl<R: Read> Scanner<R> {
    /// Create a scanner with a specific capacity.
    pub fn with_capacity(reader: R, capacity: usize) -> Scanner<R> {
        assert!(capacity >= 4);

        let mut buffer = Vec::with_capacity(capacity);

        unsafe {
            buffer.set_len(capacity);
        }

        Scanner {
            reader,
            buffer,
            position: 0,
            radix: DEFAULT_RADIX,
        }
    }

    /// Create a scanner.
    pub fn new(reader: R) -> Scanner<R> {
        Self::with_capacity(reader, DEFAULT_BUFFER_SIZE)
    }

    pub fn scan_stream(reader: R) -> Scanner<R> {
        Self::new(reader)
    }
}

impl Scanner<File> {
    pub fn scan_file(file: File) -> Result<Scanner<File>, io::Error> {
        let metadata = file.metadata()?;

        let size = metadata.len();

        let buffer_size = (size as usize).min(DEFAULT_BUFFER_SIZE).min(4);

        Ok(Self::with_capacity(file, buffer_size))
    }

    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<Scanner<File>, io::Error> {
        let file = File::open(path)?;

        Self::scan_file(file)
    }
}

impl Scanner<Cursor<String>> {
    pub fn scan_string<S: Into<String>>(s: S) -> Scanner<Cursor<String>> {
        let s = s.into();

        let size = s.len();

        let buffer_size = size.min(DEFAULT_BUFFER_SIZE).min(4);

        Scanner::with_capacity(Cursor::new(s), buffer_size)
    }
}

impl Scanner<&[u8]> {
    pub fn scan_slice<B: AsRef<[u8]> + ?Sized>(b: &B) -> Scanner<&[u8]> {
        let b = b.as_ref();

        let size = b.len();

        let buffer_size = size.min(DEFAULT_BUFFER_SIZE).min(4);

        Scanner::with_capacity(b, buffer_size)
    }
}

impl Scanner<Cursor<Vec<u8>>> {
    pub fn scan_vec(v: Vec<u8>) -> Scanner<Cursor<Vec<u8>>> {
        let size = v.len();

        let buffer_size = size.min(DEFAULT_BUFFER_SIZE).min(4);

        Scanner::with_capacity(Cursor::new(v), buffer_size)
    }
}

impl<R: Read> Scanner<R> {
    fn pull(&mut self, length: usize) {
        if length < self.position {
            unsafe {
                copy(self.buffer.as_ptr().add(length), self.buffer.as_mut_ptr(), self.position - length);
            }

            self.position -= length;
        } else {
            self.position = 0;
        }
    }

    pub fn get_remains(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    fn fetch_next_line(&mut self) -> Result<(Vec<u8>, Option<usize>), io::Error> {
        let len = self.buffer.len();

        let mut temp = Vec::new();

        let size = {
            let buffer = &mut self.buffer[self.position..len];

            self.reader.read(buffer)?
        };

        if size == 0 {
            return Ok((temp, None));
        }

        self.position += size;

        let mut p = 0;

        loop {
            let width = utf8_char_width(self.buffer[p]);

            match width {
                0 => {
                    p += 1;
                }
                1 => {
                    if self.buffer[p] as char == '\n' {
                        return Ok((temp, Some(p)));
                    }

                    p += 1;
                }
                _ => {
                    let mut wp = width + p;

                    if wp > len {
                        temp.extend_from_slice(&self.buffer);

                        self.position = 0;

                        wp = width;
                    }

                    while self.position < wp {
                        let size = {
                            let buffer = &mut self.buffer[self.position..len];

                            self.reader.read(buffer)?
                        };

                        if size == 0 {
                            break;
                        }

                        self.position += size;
                    }

                    if self.position < wp {
                        return Ok((temp, Some(self.position)));
                    } else {
                        p = wp;
                    }
                }
            }

            if p == self.position {
                if p == len {
                    temp.extend_from_slice(&self.buffer);

                    self.position = 0;

                    p = 0;

                    let size = {
                        let buffer = &mut self.buffer[self.position..len];

                        self.reader.read(buffer)?
                    };

                    if size == 0 {
                        return Ok((temp, None));
                    }

                    self.position += size;
                } else {
                    let size = {
                        let buffer = &mut self.buffer[self.position..len];

                        self.reader.read(buffer)?
                    };

                    if size == 0 {
                        if self.buffer[p - 1] as char == '\n' {
                            return Ok((temp, Some(p - 1)));
                        } else {
                            return Ok((temp, Some(p)));
                        }
                    }

                    self.position += size;
                }
            }
        }
    }
}

impl<R: Read> Scanner<R> {
    pub fn next_char(&mut self) -> Result<Option<char>, io::Error> {
        let len = self.buffer.len();

        if self.position == 0 {
            let size = {
                let buffer = &mut self.buffer[..len];

                self.reader.read(buffer)?
            };

            if size == 0 {
                return Ok(None);
            }

            self.position += size;
        }

        let width = utf8_char_width(self.buffer[0]);

        match width {
            0 => {
                self.pull(1);

                Ok(None)
            }
            1 => {
                let c = self.buffer[0] as char;

                self.pull(1);

                Ok(Some(c))
            }
            _ => {
                while self.position < width {
                    let size = {
                        let buffer = &mut self.buffer[self.position..len];

                        self.reader.read(buffer)?
                    };

                    if size == 0 {
                        break;
                    }

                    self.position += size;
                }

                if self.position < width {
                    self.pull(1);

                    Ok(None)
                } else {
                    let s = match core::str::from_utf8(&self.buffer[..width]) {
                        Ok(s) => {
                            s.chars().next()
                        }
                        Err(_) => {
                            self.pull(1);

                            return Ok(None);
                        }
                    };

                    self.pull(width);

                    Ok(s)
                }
            }
        }
    }

    pub fn next_line(&mut self) -> Result<Option<String>, io::Error> {
        let result = self.fetch_next_line()?;

        let mut v = result.0;

        match result.1 {
            Some(t) => {
                v.extend_from_slice(&self.buffer[..t]);

                self.pull(t + 1);

                return Ok(Some(String::from_utf8_lossy(&v).to_string()));
            }
            None => {
                if v.is_empty() {
                    return Ok(None);
                } else {
                    return Ok(Some(String::from_utf8_lossy(&v).to_string()));
                }
            }
        }
    }
}