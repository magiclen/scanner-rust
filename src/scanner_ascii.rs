use std::char::REPLACEMENT_CHARACTER;
use std::cmp::Ordering;
use std::fs::File;
use std::intrinsics::copy;
use std::io::Read;
use std::path::Path;
use std::str::{from_utf8_unchecked, FromStr};

use crate::generic_array::typenum::{IsGreaterOrEqual, True, U256, U4};
use crate::generic_array::{ArrayLength, GenericArray};

use crate::whitespaces::*;
use crate::ScannerError;

/// A simple text scanner which can parse primitive types and strings using ASCII.
#[derive(Educe)]
#[educe(Debug)]
pub struct ScannerAscii<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True> = U256> {
    #[educe(Debug(ignore))]
    reader: R,
    buf: GenericArray<u8, N>,
    buf_length: usize,
    buf_offset: usize,
    passing_byte: Option<u8>,
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
        Self::new2(reader)
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    /// Create a scanner from a reader and set the buffer size via generics.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use std::io;
    ///
    /// use scanner_rust::generic_array::typenum::U1024;
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc: ScannerAscii<_, U1024> = ScannerAscii::new2(io::stdin());
    /// ```
    #[inline]
    pub fn new2(reader: R) -> ScannerAscii<R, N> {
        ScannerAscii {
            reader,
            buf: GenericArray::default(),
            buf_length: 0,
            buf_offset: 0,
            passing_byte: None,
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
        Self::scan_path2(path)
    }
}

impl<N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<File, N> {
    /// Create a scanner to read data from a file by its path and set the buffer size via generics.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::generic_array::typenum::U1024;
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc: ScannerAscii<_, U1024> = ScannerAscii::scan_path2("Cargo.toml").unwrap();
    /// ```
    #[inline]
    pub fn scan_path2<P: AsRef<Path>>(path: P) -> Result<ScannerAscii<File, N>, ScannerError> {
        let reader = File::open(path)?;

        Ok(ScannerAscii::new2(reader))
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    #[inline]
    fn buf_align_to_frond_end(&mut self) {
        unsafe {
            copy(self.buf.as_ptr().add(self.buf_offset), self.buf.as_mut_ptr(), self.buf_length);
        }

        self.buf_offset = 0;
    }

    #[inline]
    fn buf_left_shift(&mut self, distance: usize) {
        debug_assert!(self.buf_length >= distance);

        self.buf_offset += distance;

        if self.buf_offset >= N::USIZE - 4 {
            self.buf_align_to_frond_end();
        }

        self.buf_length -= distance;
    }

    /// Left shift (if necessary) the buffer to remove bytes from the start of the buffer. Typically, you should use this after `peek`ing the buffer.
    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn remove_heading_bytes_from_buffer(&mut self, number_of_bytes: usize) {
        self.buf_left_shift(number_of_bytes);
    }

    fn passing_read(&mut self) -> Result<bool, ScannerError> {
        if self.buf_length == 0 {
            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(false);
            }

            self.buf_length += size;

            if let Some(passing_byte) = self.passing_byte {
                self.passing_byte = None;

                if self.buf[self.buf_offset] == passing_byte {
                    self.buf_left_shift(1);

                    return if size == 1 {
                        let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                        if size == 0 {
                            Ok(false)
                        } else {
                            self.buf_length += size;

                            Ok(true)
                        }
                    } else {
                        Ok(true)
                    };
                }
            }

            Ok(true)
        } else {
            Ok(true)
        }
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
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
        if !self.passing_read()? {
            return Ok(None);
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
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut temp = String::new();

        loop {
            let e = self.buf[self.buf_offset];

            println!("{}", e);

            if e == b'\n' {
                if self.buf_length == 1 {
                    self.passing_byte = Some(b'\r');
                    self.buf_left_shift(1);
                } else if self.buf[self.buf_offset + 1] == b'\r' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(temp));
            } else if e == b'\r' {
                if self.buf_length == 1 {
                    self.passing_byte = Some(b'\n');
                    self.buf_left_shift(1);
                } else if self.buf[self.buf_offset + 1] == b'\n' {
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
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut temp = Vec::new();

        loop {
            let e = self.buf[self.buf_offset];

            if e == b'\n' {
                if self.buf_length == 1 {
                    self.passing_byte = Some(b'\r');
                    self.buf_left_shift(1);
                } else if self.buf[self.buf_offset + 1] == b'\r' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(temp));
            } else if e == b'\r' {
                if self.buf_length == 1 {
                    self.passing_byte = Some(b'\n');
                    self.buf_left_shift(1);
                } else if self.buf[self.buf_offset + 1] == b'\n' {
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
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut c = 0;

        loop {
            let e = self.buf[self.buf_offset];

            if e == b'\n' {
                if self.buf_length == 1 {
                    self.passing_byte = Some(b'\r');
                    self.buf_left_shift(1);
                } else if self.buf[self.buf_offset + 1] == b'\r' {
                    self.buf_left_shift(2);
                } else {
                    self.buf_left_shift(1);
                }

                return Ok(Some(c));
            } else if e == b'\r' {
                if self.buf_length == 1 {
                    self.passing_byte = Some(b'\n');
                    self.buf_left_shift(1);
                } else if self.buf[self.buf_offset + 1] == b'\n' {
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

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
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
        if !self.passing_read()? {
            return Ok(false);
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

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    /// Read the next bytes. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_bytes(3).unwrap());
    /// assert_eq!(Some(" 456".into()), sc.next_bytes(4).unwrap());
    /// assert_eq!(Some("\r\n789 ".into()), sc.next_bytes(6).unwrap());
    /// assert_eq!(Some("ab".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some(" ".into()), sc.next_bytes(2).unwrap());
    /// assert_eq!(None, sc.next_bytes(2).unwrap());
    /// ```
    pub fn next_bytes(
        &mut self,
        max_number_of_bytes: usize,
    ) -> Result<Option<Vec<u8>>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut temp = Vec::new();
        let mut c = 0;

        while c < max_number_of_bytes {
            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }

            let dropping_bytes = self.buf_length.min(max_number_of_bytes - c);

            temp.extend_from_slice(&self.buf[self.buf_offset..(self.buf_offset + dropping_bytes)]);

            self.buf_left_shift(dropping_bytes);

            c += dropping_bytes;
        }

        Ok(Some(temp))
    }

    /// Drop the next N bytes. If there is nothing to read, it will return `Ok(None)`. If there are something to read, it will return `Ok(Some(i))`. The `i` is the length of the actually dropped bytes.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_bytes(7).unwrap());
    /// assert_eq!(Some("".into()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(1), sc.drop_next_bytes(1).unwrap());
    /// assert_eq!(Some(" ab ".into()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_bytes(1).unwrap());
    /// ```
    pub fn drop_next_bytes(
        &mut self,
        max_number_of_bytes: usize,
    ) -> Result<Option<usize>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut c = 0;

        while c < max_number_of_bytes {
            if self.buf_length == 0 {
                let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(c));
                }

                self.buf_length += size;
            }

            let dropping_bytes = self.buf_length.min(max_number_of_bytes - c);

            self.buf_left_shift(dropping_bytes);

            c += dropping_bytes;
        }

        Ok(Some(c))
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    /// Read the next text until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_until(" ").unwrap());
    /// assert_eq!(Some("456\r".into()), sc.next_until("\n").unwrap());
    /// assert_eq!(Some("78".into()), sc.next_until("9 ").unwrap());
    /// assert_eq!(Some("\n\n ab ".into()), sc.next_until("kk").unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    pub fn next_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<String>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let boundary = boundary.as_ref().as_bytes();
        let boundary_length = boundary.len();
        let mut temp = String::new();

        let mut b = 0;

        loop {
            let mut p = 0;

            while p < self.buf_length {
                if self.buf[self.buf_offset + p] == boundary[b] {
                    b += 1;
                    p += 1;

                    if b == boundary_length {
                        match p.cmp(&boundary_length) {
                            Ordering::Equal => (),
                            Ordering::Greater => {
                                temp.push_str(
                                    String::from_utf8_lossy(
                                        &self.buf[self.buf_offset
                                            ..(self.buf_offset + p - boundary_length)],
                                    )
                                    .as_ref(),
                                );
                            }
                            Ordering::Less => {
                                let adjusted_temp_length = temp.len() - (boundary_length - p);

                                unsafe {
                                    temp.as_mut_vec().set_len(adjusted_temp_length);
                                }
                            }
                        }

                        self.buf_left_shift(p);

                        return Ok(Some(temp));
                    }
                } else {
                    b = 0;
                    p += 1;
                }
            }

            temp.push_str(
                String::from_utf8_lossy(
                    &self.buf[self.buf_offset..(self.buf_offset + self.buf_length)],
                )
                .as_ref(),
            );

            self.buf_left_shift(self.buf_length);

            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(Some(temp));
            }

            self.buf_length += size;
        }
    }

    /// Read the next data until it reaches a specific boundary without fully validating UTF-8. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_until_raw(" ").unwrap());
    /// assert_eq!(Some("456\r".into()), sc.next_until_raw("\n").unwrap());
    /// assert_eq!(Some("78".into()), sc.next_until_raw("9 ").unwrap());
    /// assert_eq!(Some("\n\n ab ".into()), sc.next_until_raw("kk").unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    pub fn next_until_raw<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<Vec<u8>>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let boundary = boundary.as_ref();
        let boundary_length = boundary.len();
        let mut temp = Vec::new();

        let mut b = 0;

        loop {
            let mut p = 0;

            while p < self.buf_length {
                if self.buf[self.buf_offset + p] == boundary[b] {
                    b += 1;
                    p += 1;

                    if b == boundary_length {
                        match p.cmp(&boundary_length) {
                            Ordering::Equal => (),
                            Ordering::Greater => {
                                temp.extend_from_slice(
                                    &self.buf
                                        [self.buf_offset..(self.buf_offset + p - boundary_length)],
                                );
                            }
                            Ordering::Less => {
                                let adjusted_temp_length = temp.len() - (boundary_length - p);

                                unsafe {
                                    temp.set_len(adjusted_temp_length);
                                }
                            }
                        }

                        self.buf_left_shift(p);

                        return Ok(Some(temp));
                    }
                } else {
                    b = 0;
                    p += 1;
                }
            }

            temp.extend_from_slice(&self.buf[self.buf_offset..(self.buf_offset + self.buf_length)]);

            self.buf_left_shift(self.buf_length);

            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(Some(temp));
            }

            self.buf_length += size;
        }
    }

    /// Drop the next data until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_until("\r\n").unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_until("\n").unwrap());
    /// assert_eq!(Some(" ab ".into()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_until("").unwrap());
    /// ```
    pub fn drop_next_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<usize>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let boundary = boundary.as_ref();
        let boundary_length = boundary.len();
        let mut c = 0;

        let mut b = 0;

        loop {
            let mut p = 0;

            while p < self.buf_length {
                if self.buf[self.buf_offset + p] == boundary[b] {
                    b += 1;
                    p += 1;

                    if b == boundary_length {
                        match p.cmp(&boundary_length) {
                            Ordering::Equal => (),
                            Ordering::Greater => {
                                c += p - boundary_length;
                            }
                            Ordering::Less => {
                                c -= boundary_length - p;
                            }
                        }

                        self.buf_left_shift(p);

                        return Ok(Some(c));
                    }
                } else {
                    b = 0;
                    p += 1;
                }
            }

            c += self.buf_length;

            self.buf_left_shift(self.buf_length);

            let size = self.reader.read(&mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(Some(c));
            }

            self.buf_length += size;
        }
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    /// Try to fill up the buffer and return the immutable byte slice of the valid buffered data.
    /// If the `shift` parameter is set to `false`, the guaranteed minimum data length of the result is **32** (if the unread data is long enough), otherwise it is `BUFFER_SIZE`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!("123 456\r\n789 \n\n ab ".as_bytes(), sc.peek(false).unwrap());
    /// ```
    #[inline]
    pub fn peek(&mut self, shift: bool) -> Result<&[u8], ScannerError> {
        if shift {
            self.buf_align_to_frond_end();
        }

        loop {
            let size = self.reader.read(&mut self.buf[(self.buf_offset + self.buf_length)..])?;

            if size == 0 {
                break;
            }

            self.buf_length += size;
        }

        Ok(&self.buf[self.buf_offset..(self.buf_offset + self.buf_length)])
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    #[inline]
    fn next_raw_parse<T: FromStr>(&mut self) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
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
        self.next_raw_parse()
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ScannerAscii<R, N> {
    #[inline]
    fn next_until_raw_parse<T: FromStr, D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_until_raw(boundary)?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(&s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u8_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u8_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u8_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u8>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u16_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u16_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u16_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u16>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u32_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u32_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u32>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u64_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u64_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u64>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u128_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u128_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u128_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u128>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `usize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_usize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_usize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_usize_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<usize>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_usize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_usize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i8_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i8>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i16_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i16_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i16_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i16>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i32_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i32_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i32>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i64_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i64_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i64>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i128_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i128_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i128_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i128>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `isize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_isize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_isize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_isize_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<isize>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `f32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f32_until(" ").unwrap());
    /// assert_eq!(Some(2.5), sc.next_f32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_f32_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<f32>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `f64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerAscii;
    ///
    /// let mut sc = ScannerAscii::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f64_until(" ").unwrap());
    /// assert_eq!(Some(2.5), sc.next_f64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_f64_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<f64>, ScannerError> {
        self.next_until_raw_parse(boundary)
    }
}
