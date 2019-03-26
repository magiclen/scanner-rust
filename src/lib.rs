#![cfg_attr(feature = "nightly", feature(str_internals))]

mod utf8;
mod whitespaces;

use std::io::{self, Read, Cursor};
use std::path::Path;
use std::fs::File;
use std::ptr::copy;
use std::num::ParseIntError;
use std::num::ParseFloatError;

use self::utf8::*;
use self::whitespaces::*;

const DEFAULT_BUFFER_SIZE: usize = 64; // must be equal to or bigger than 4

#[derive(Debug)]
pub enum ScannerError {
    IOError(io::Error),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
}

pub struct Scanner<R: Read> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
    last_cr: bool,
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
            last_cr: false,
        }
    }

    /// Create a scanner.
    pub fn new(reader: R) -> Scanner<R> {
        Self::with_capacity(reader, DEFAULT_BUFFER_SIZE)
    }
}

impl<R: Read> Scanner<R> {
    pub fn scan_stream(reader: R) -> Scanner<R> {
        Self::new(reader)
    }
}

impl Scanner<File> {
    pub fn scan_file(file: File) -> Result<Scanner<File>, ScannerError> {
        let metadata = file.metadata().map_err(|err| ScannerError::IOError(err))?;

        let size = metadata.len();

        let buffer_size = (size as usize).min(DEFAULT_BUFFER_SIZE).max(4);

        Ok(Self::with_capacity(file, buffer_size))
    }

    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<Scanner<File>, ScannerError> {
        let file = File::open(path).map_err(|err| ScannerError::IOError(err))?;

        Self::scan_file(file)
    }
}

impl Scanner<Cursor<String>> {
    pub fn scan_string<S: Into<String>>(s: S) -> Scanner<Cursor<String>> {
        let s = s.into();

        let size = s.len();

        let buffer_size = size.min(DEFAULT_BUFFER_SIZE).max(4);

        Scanner::with_capacity(Cursor::new(s), buffer_size)
    }
}

impl Scanner<&[u8]> {
    pub fn scan_slice<B: AsRef<[u8]> + ?Sized>(b: &B) -> Scanner<&[u8]> {
        let b = b.as_ref();

        let size = b.len();

        let buffer_size = size.min(DEFAULT_BUFFER_SIZE).max(4);

        Scanner::with_capacity(b, buffer_size)
    }
}

impl Scanner<Cursor<Vec<u8>>> {
    pub fn scan_vec(v: Vec<u8>) -> Scanner<Cursor<Vec<u8>>> {
        let size = v.len();

        let buffer_size = size.min(DEFAULT_BUFFER_SIZE).max(4);

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

    fn fetch_next_line(&mut self) -> Result<(Vec<u8>, Option<usize>, bool), ScannerError> {
        let len = self.buffer.len();

        let mut temp = Vec::new();

        if self.position == 0 {
            let size = {
                let buffer = &mut self.buffer[self.position..];

                self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
            };

            if size == 0 {
                return Ok((temp, None, false));
            }

            self.position += size;
        }

        let mut p = 0;

        loop {
            let width = utf8_char_width(self.buffer[p]);

            match width {
                0 => {
                    p += 1;
                }
                1 => {
                    if self.buffer[p] == b'\n' {
                        return Ok((temp, Some(p), false));
                    } else if self.buffer[p] == b'\r' {
                        return Ok((temp, Some(p), true));
                    }

                    p += 1;
                }
                _ => {
                    let mut wp = width + p;

                    if wp > len {
                        temp.extend_from_slice(&self.buffer[..self.position]);

                        self.position = 0;

                        wp = width - 1;
                    }

                    while self.position < wp {
                        let size = {
                            let buffer = &mut self.buffer[self.position..];

                            self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                        };

                        if size == 0 {
                            break;
                        }

                        self.position += size;
                    }

                    if self.position < wp {
                        return Ok((temp, Some(self.position), false));
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
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                    };

                    if size == 0 {
                        return Ok((temp, None, false));
                    }

                    self.position += size;
                } else {
                    let size = {
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                    };

                    if size == 0 {
                        return Ok((temp, Some(p), false));
                    }

                    self.position += size;
                }
            }
        }
    }

    fn fetch_next_non_whitespace(&mut self) -> Result<Option<usize>, ScannerError> {
        let len = self.buffer.len();

        if self.position == 0 {
            let size = {
                let buffer = &mut self.buffer[self.position..];

                self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
            };

            if size == 0 {
                return Ok(None);
            }

            self.position += size;
        }

        let mut p = 0;

        loop {
            let width = utf8_char_width(self.buffer[p]);

            match width {
                0 => {
                    return Ok(Some(p));
                }
                1 => {
                    if !is_whitespace_1(self.buffer[p]) {
                        return Ok(Some(p));
                    }

                    p += 1;
                }
                _ => {
                    let mut wp = width + p;

                    if wp > len {
                        self.buffer[0] = self.buffer[p];

                        self.position = 1;

                        p = 0;

                        wp = width;
                    }

                    while self.position < wp {
                        let size = {
                            let buffer = &mut self.buffer[self.position..];

                            self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                        };

                        if size == 0 {
                            break;
                        }

                        self.position += size;
                    }

                    if self.position < wp {
                        return Ok(Some(p));
                    } else {
                        match width {
                            2 => {}
                            3 => {
                                if !is_whitespace_3(self.buffer[p], self.buffer[p + 1], self.buffer[p + 2]) {
                                    return Ok(Some(p));
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                        p = wp;
                    }
                }
            }

            if p == self.position {
                if p == len {
                    self.position = 0;

                    p = 0;

                    let size = {
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                    };

                    if size == 0 {
                        return Ok(None);
                    }

                    self.position += size;
                } else {
                    let size = {
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                    };

                    if size == 0 {
                        return Ok(None);
                    }

                    self.position += size;
                }
            }
        }
    }

    fn fetch_next_whitespace(&mut self) -> Result<(Vec<u8>, Option<usize>), ScannerError> {
        let len = self.buffer.len();

        let mut temp = Vec::new();

        if self.position == 0 {
            let size = {
                let buffer = &mut self.buffer[self.position..];

                self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
            };

            if size == 0 {
                return Ok((temp, None));
            }

            self.position += size;
        }

        let mut p = 0;

        loop {
            let width = utf8_char_width(self.buffer[p]);

            match width {
                0 => {
                    p += 1;
                }
                1 => {
                    if is_whitespace_1(self.buffer[p]) {
                        return Ok((temp, Some(p)));
                    }

                    p += 1;
                }
                _ => {
                    let mut wp = width + p;

                    if wp > len {
                        temp.extend_from_slice(&self.buffer[..self.position]);

                        self.position = 0;

                        wp = width - 1;
                    }

                    while self.position < wp {
                        let size = {
                            let buffer = &mut self.buffer[self.position..];

                            self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                        };

                        if size == 0 {
                            break;
                        }

                        self.position += size;
                    }

                    if self.position < wp {
                        return Ok((temp, Some(self.position)));
                    } else {
                        match width {
                            2 => {}
                            3 => {
                                if is_whitespace_3(self.buffer[p], self.buffer[p + 1], self.buffer[p + 2]) {
                                    return Ok((temp, Some(p)));
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
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
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                    };

                    if size == 0 {
                        return Ok((temp, None));
                    }

                    self.position += size;
                } else {
                    let size = {
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
                    };

                    if size == 0 {
                        return Ok((temp, Some(p)));
                    }

                    self.position += size;
                }
            }
        }
    }
}

impl<R: Read> Scanner<R> {
    pub fn next_char(&mut self) -> Result<Option<char>, ScannerError> {
        self.last_cr = false;

        if self.position == 0 {
            let size = {
                let buffer = &mut self.buffer[..];

                self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
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
                        let buffer = &mut self.buffer[self.position..];

                        self.reader.read(buffer).map_err(|err| ScannerError::IOError(err))?
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

    pub fn next_line(&mut self) -> Result<Option<String>, ScannerError> {
        let result = self.fetch_next_line()?;

        let mut v = result.0;

        match result.1 {
            Some(t) => {
                v.extend_from_slice(&self.buffer[..t]);

                self.pull(t + 1);

                if v.is_empty() && !result.2 {
                    if self.last_cr {
                        self.last_cr = false;

                        return self.next_line();
                    }
                }

                self.last_cr = result.2;

                Ok(Some(String::from_utf8_lossy(&v).to_string()))
            }
            None => {
                if v.is_empty() {
                    Ok(None)
                } else {
                    self.last_cr = result.2;

                    Ok(Some(String::from_utf8_lossy(&v).to_string()))
                }
            }
        }
    }
}

impl<R: Read> Scanner<R> {
    pub fn skip_whitespaces(&mut self) -> Result<bool, ScannerError> {
        self.last_cr = false;

        let result = self.fetch_next_non_whitespace()?;

        match result {
            Some(t) => {
                self.pull(t);

                return Ok(true);
            }
            None => {
                Ok(false)
            }
        }
    }

    pub fn next(&mut self) -> Result<Option<String>, ScannerError> {
        let result = self.skip_whitespaces()?;

        if result {
            let result = self.fetch_next_whitespace()?;

            let mut v = result.0;

            match result.1 {
                Some(t) => {
                    v.extend_from_slice(&self.buffer[..t]);

                    self.pull(t);

                    Ok(Some(String::from_utf8_lossy(&v).to_string()))
                }
                None => {
                    if v.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(String::from_utf8_lossy(&v).to_string()))
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn next_u8(&mut self) -> Result<Option<u8>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_u16(&mut self) -> Result<Option<u16>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_u32(&mut self) -> Result<Option<u32>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_u64(&mut self) -> Result<Option<u64>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_u128(&mut self) -> Result<Option<u128>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_i8(&mut self) -> Result<Option<i8>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_i16(&mut self) -> Result<Option<i16>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_i32(&mut self) -> Result<Option<i32>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_i64(&mut self) -> Result<Option<i64>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_i128(&mut self) -> Result<Option<i128>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseIntError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_f32(&mut self) -> Result<Option<f32>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseFloatError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }

    pub fn next_f64(&mut self) -> Result<Option<f64>, ScannerError> {
        let result = self.next()?;

        match result {
            Some(s) => {
                Ok(Some(s.parse().map_err(|err| ScannerError::ParseFloatError(err))?))
            }
            None => {
                Ok(None)
            }
        }
    }
}