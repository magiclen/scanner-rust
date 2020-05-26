use std::char::REPLACEMENT_CHARACTER;
use std::fmt::{self, Formatter};
use std::fs::File;
use std::intrinsics::copy;
use std::io::Read;
use std::path::Path;
use std::str::from_utf8_unchecked;

use crate::whitespaces::*;
use crate::ScannerError;
use crate::BUFFER_SIZE;

/// A simple text scanner which can parse primitive types and strings using ASCII.
#[derive(Educe)]
#[educe(Debug)]
pub struct ScannerAscii<R: Read> {
    #[educe(Debug(ignore))]
    reader: R,
    #[educe(Debug(method = "fmt"))]
    buf: [u8; BUFFER_SIZE],
    buf_length: usize,
    buf_offset: usize,
}

impl<R: Read> ScannerAscii<R> {
    /// Create a scanner from a reader.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use std::io;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new(io::stdin());
    /// ```
    #[inline]
    pub fn new(reader: R) -> ScannerAscii<R> {
        ScannerAscii {
            reader,
            buf: [0; BUFFER_SIZE],
            buf_length: 0,
            buf_offset: 0,
        }
    }
}

impl ScannerAscii<File> {
    /// Create a scanner to read data from a file by its path.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::scan_path("Cargo.toml").unwrap();
    /// ```
    #[inline]
    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<ScannerAscii<File>, ScannerError> {
        let reader = File::open(path)?;

        Ok(ScannerAscii::new(reader))
    }
}

impl<R: Read> ScannerAscii<R> {
    fn buf_left_shift(&mut self, distance: usize) {
        debug_assert!(self.buf_length >= distance);

        self.buf_offset += distance;

        if BUFFER_SIZE - self.buf_offset < 32 {
            unsafe {
                copy(
                    self.buf.as_ptr().add(self.buf_offset),
                    self.buf.as_mut_ptr(),
                    self.buf_length,
                );
            }

            self.buf_offset = 0;
        }

        self.buf_length -= distance;
    }
}

impl<R: Read> ScannerAscii<R> {
    /// Read the next char. If the data is not a correct char, it will return a `Ok(Some(REPLACEMENT_CHARACTER))` which is �. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("5 c ab".as_bytes());
    ///
    /// assert_eq!(Some('5'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('c'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('a'), sc.next_char().unwrap());
    /// assert_eq!(Some('b'), sc.next_char().unwrap());
    /// assert_eq!(None, sc.next_char().unwrap());
    /// ```
    pub fn next_char(&mut self) -> Result<Option<char>, ScannerError> {
        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let e = self.buf[self.buf_offset];

        self.buf_left_shift(1);

        if e >= 128 {
            Ok(Some(REPLACEMENT_CHARACTER))
        } else {
            Ok(Some(e as char))
        }
    }

    /// Read the next line but not include the tailing line character (or line chracters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123 456".into()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some("".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(" ab ".into()), sc.next_line().unwrap());
    /// ```
    pub fn next_line(&mut self) -> Result<Option<String>, ScannerError> {
        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let mut temp = String::new();

        loop {
            let e = self.buf[self.buf_offset];

            if e == b'\n' {
                if self.buf_length == 1 {
                    let size =
                        self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

                    if size == 0 {
                        self.buf_left_shift(1);

                        return Ok(Some(temp));
                    }

                    self.buf_length += size;
                }

                if self.buf[self.buf_offset + 1] == b'\r' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(temp));
            } else if e == b'\r' {
                if self.buf_length == 1 {
                    let size =
                        self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

                    if size == 0 {
                        self.buf_left_shift(1);

                        return Ok(Some(temp));
                    }

                    self.buf_length += size;
                }

                if self.buf[self.buf_offset + 1] == b'\n' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(temp));
            }

            self.buf_left_shift(1);

            if e >= 128 {
                temp.push(REPLACEMENT_CHARACTER);
            } else {
                temp.push(e as char);
            }

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }
    }

    /// Read the next line include the tailing line character (or line chracters like `CrLf`(`\r\n`)) without validating ASCII. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123 456".into()), sc.next_line_raw().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line_raw().unwrap());
    /// assert_eq!(Some("".into()), sc.next_line_raw().unwrap());
    /// assert_eq!(Some(" ab ".into()), sc.next_line_raw().unwrap());
    /// ```
    pub fn next_line_raw(&mut self) -> Result<Option<Vec<u8>>, ScannerError> {
        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let mut temp = Vec::new();

        loop {
            let e = self.buf[self.buf_offset];

            if e == b'\n' {
                if self.buf_length == 1 {
                    let size =
                        self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

                    if size == 0 {
                        self.buf_left_shift(1);

                        return Ok(Some(temp));
                    }

                    self.buf_length += size;
                }

                if self.buf[self.buf_offset + 1] == b'\r' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(temp));
            } else if e == b'\r' {
                if self.buf_length == 1 {
                    let size =
                        self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

                    if size == 0 {
                        self.buf_left_shift(1);

                        return Ok(Some(temp));
                    }

                    self.buf_length += size;
                }

                if self.buf[self.buf_offset + 1] == b'\n' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(temp));
            }

            self.buf_left_shift(1);

            temp.push(e);

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }
    }

    /// Drop the next line but not include the tailing line character (or line chracters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`. If there are something to read, it will return `Ok(Some(i))`. The `i` is the length of the dropped line.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_line().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_line().unwrap());
    /// assert_eq!(Some(" ab ".into()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_line().unwrap());
    /// ```
    pub fn drop_next_line(&mut self) -> Result<Option<usize>, ScannerError> {
        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let mut c = 0;

        loop {
            let e = self.buf[self.buf_offset];

            if e == b'\n' {
                if self.buf_length == 1 {
                    let size =
                        self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

                    if size == 0 {
                        self.buf_left_shift(1);

                        return Ok(Some(c));
                    }

                    self.buf_length += size;
                }

                if self.buf[self.buf_offset + 1] == b'\r' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(c));
            } else if e == b'\r' {
                if self.buf_length == 1 {
                    let size =
                        self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

                    if size == 0 {
                        self.buf_left_shift(1);

                        return Ok(Some(c));
                    }

                    self.buf_length += size;
                }

                if self.buf[self.buf_offset + 1] == b'\n' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(c));
            }

            self.buf_left_shift(1);

            c += 1;

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(c));
                }

                self.buf_length += size;
            }
        }
    }
}

impl<R: Read> ScannerAscii<R> {
    /// Skip the next whitespaces (`javaWhitespace`). If there is nothing to read, it will return `Ok(false)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2   c".as_bytes());
    ///
    /// assert_eq!(Some('1'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('2'), sc.next_char().unwrap());
    /// assert_eq!(true, sc.skip_whitespaces().unwrap());
    /// assert_eq!(Some('c'), sc.next_char().unwrap());
    /// assert_eq!(false, sc.skip_whitespaces().unwrap());
    /// ```
    pub fn skip_whitespaces(&mut self) -> Result<bool, ScannerError> {
        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(false);
            }

            self.buf_length += size;
        }

        loop {
            if !is_whitespace_1(self.buf[self.buf_offset]) {
                break;
            }

            self.buf_left_shift(1);

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(true);
                }

                self.buf_length += size;
            }
        }

        Ok(true)
    }

    /// Read the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next().unwrap());
    /// assert_eq!(Some("456".into()), sc.next().unwrap());
    /// assert_eq!(Some("789".into()), sc.next().unwrap());
    /// assert_eq!(Some("ab".into()), sc.next().unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<String>, ScannerError> {
        if !self.skip_whitespaces()? {
            return Ok(None);
        }

        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let mut temp = String::new();

        loop {
            let e = self.buf[self.buf_offset];

            if is_whitespace_1(e) {
                self.buf_left_shift(1);

                return Ok(Some(temp));
            }

            self.buf_left_shift(1);

            if e >= 128 {
                temp.push(REPLACEMENT_CHARACTER);
            } else {
                temp.push(e as char);
            }

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }
    }

    /// Read the next token separated by whitespaces without validating ASCII. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some("456".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some("789".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some("ab".into()), sc.next_raw().unwrap());
    /// assert_eq!(None, sc.next_raw().unwrap());
    /// ```
    pub fn next_raw(&mut self) -> Result<Option<Vec<u8>>, ScannerError> {
        if !self.skip_whitespaces()? {
            return Ok(None);
        }

        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let mut temp = Vec::new();

        loop {
            let e = self.buf[self.buf_offset];

            if is_whitespace_1(e) {
                self.buf_left_shift(1);

                return Ok(Some(temp));
            }

            self.buf_left_shift(1);

            temp.push(e);

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }
    }

    /// Drop the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`. If there are something to read, it will return `Ok(Some(i))`. The `i` is the length of the dropped line.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("456".into()), sc.next().unwrap());
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("ab".into()), sc.next().unwrap());
    /// assert_eq!(None, sc.drop_next().unwrap());
    /// ```
    pub fn drop_next(&mut self) -> Result<Option<usize>, ScannerError> {
        if !self.skip_whitespaces()? {
            return Ok(None);
        }

        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(None);
            }

            self.buf_length += size;
        }

        let mut c = 0;

        loop {
            if is_whitespace_1(self.buf[self.buf_offset]) {
                self.buf_left_shift(1);

                return Ok(Some(c));
            }

            self.buf_left_shift(1);

            c += 1;

            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(c));
                }

                self.buf_length += size;
            }
        }
    }
}

impl<R: Read> ScannerAscii<R> {
    /// Read the next token separated by whitespaces and parse it to a `u8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u8().unwrap());
    /// assert_eq!(Some(2), sc.next_u8().unwrap());
    /// ```
    #[inline]
    pub fn next_u8(&mut self) -> Result<Option<u8>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `u16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u16().unwrap());
    /// assert_eq!(Some(2), sc.next_u16().unwrap());
    /// ```
    #[inline]
    pub fn next_u16(&mut self) -> Result<Option<u16>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `u32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u32().unwrap());
    /// assert_eq!(Some(2), sc.next_u32().unwrap());
    /// ```
    #[inline]
    pub fn next_u32(&mut self) -> Result<Option<u32>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `u64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u64().unwrap());
    /// assert_eq!(Some(2), sc.next_u64().unwrap());
    /// ```
    #[inline]
    pub fn next_u64(&mut self) -> Result<Option<u64>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `u128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u128().unwrap());
    /// assert_eq!(Some(2), sc.next_u128().unwrap());
    /// ```
    #[inline]
    pub fn next_u128(&mut self) -> Result<Option<u128>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `usize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_usize().unwrap());
    /// assert_eq!(Some(2), sc.next_usize().unwrap());
    /// ```
    #[inline]
    pub fn next_usize(&mut self) -> Result<Option<usize>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `i8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i8().unwrap());
    /// assert_eq!(Some(2), sc.next_i8().unwrap());
    /// ```
    #[inline]
    pub fn next_i8(&mut self) -> Result<Option<i8>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `i16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i16().unwrap());
    /// assert_eq!(Some(2), sc.next_i16().unwrap());
    /// ```
    #[inline]
    pub fn next_i16(&mut self) -> Result<Option<i16>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `i32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i32().unwrap());
    /// assert_eq!(Some(2), sc.next_i32().unwrap());
    /// ```
    #[inline]
    pub fn next_i32(&mut self) -> Result<Option<i32>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `i64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i64().unwrap());
    /// assert_eq!(Some(2), sc.next_i64().unwrap());
    /// ```
    #[inline]
    pub fn next_i64(&mut self) -> Result<Option<i64>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `i128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i128().unwrap());
    /// assert_eq!(Some(2), sc.next_i128().unwrap());
    /// ```
    #[inline]
    pub fn next_i128(&mut self) -> Result<Option<i128>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `isize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_isize().unwrap());
    /// assert_eq!(Some(2), sc.next_isize().unwrap());
    /// ```
    #[inline]
    pub fn next_isize(&mut self) -> Result<Option<isize>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `f32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f32().unwrap());
    /// assert_eq!(Some(2.5), sc.next_f32().unwrap());
    /// ```
    #[inline]
    pub fn next_f32(&mut self) -> Result<Option<f32>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `f64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f64().unwrap());
    /// assert_eq!(Some(2.5), sc.next_f64().unwrap());
    /// ```
    #[inline]
    pub fn next_f64(&mut self) -> Result<Option<f64>, ScannerError> {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }
}

#[inline]
fn fmt(s: &[u8], f: &mut Formatter) -> fmt::Result {
    let mut list = f.debug_list();

    for n in s.iter() {
        list.entry(n);
    }

    Ok(())
}
