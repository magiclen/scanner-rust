use std::{
    char::REPLACEMENT_CHARACTER,
    str::{from_utf8, from_utf8_unchecked, FromStr},
};

use utf8_width::*;

use crate::{whitespaces::*, ScannerError};

/// A simple text scanner which can in-memory-ly parse primitive types and strings using UTF-8 from a byte slice.
#[derive(Debug)]
pub struct ScannerU8Slice<'a> {
    data:        &'a [u8],
    data_length: usize,
    position:    usize,
}

impl<'a> ScannerU8Slice<'a> {
    /// Create a scanner from in-memory bytes.
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new(b"123 456");
    /// ```
    #[inline]
    pub fn new<D: ?Sized + AsRef<[u8]>>(data: &D) -> ScannerU8Slice {
        let data = data.as_ref();

        ScannerU8Slice {
            data,
            data_length: data.len(),
            position: 0,
        }
    }
}

impl<'a> ScannerU8Slice<'a> {
    /// Read the next char. If the data is not a correct char, it will return a `Ok(Some(REPLACEMENT_CHARACTER))` which is �. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("5 c 中文".as_bytes());
    ///
    /// assert_eq!(Some('5'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('c'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('中'), sc.next_char().unwrap());
    /// assert_eq!(Some('文'), sc.next_char().unwrap());
    /// assert_eq!(None, sc.next_char().unwrap());
    /// ```
    pub fn next_char(&mut self) -> Result<Option<char>, ScannerError> {
        if self.position == self.data_length {
            return Ok(None);
        }

        let e = self.data[self.position];

        let width = get_width(e);

        match width {
            0 => {
                self.position += 1;

                Ok(Some(REPLACEMENT_CHARACTER))
            },
            1 => {
                self.position += 1;

                Ok(Some(e as char))
            },
            _ => {
                if self.position + width > self.data_length {
                    self.position += 1;

                    Ok(Some(REPLACEMENT_CHARACTER))
                } else {
                    let char_str_bytes = &self.data[self.position..(self.position + width)];

                    match from_utf8(char_str_bytes) {
                        Ok(char_str) => {
                            self.position += width;

                            Ok(char_str.chars().next())
                        },
                        Err(_) => {
                            self.position += 1;

                            Ok(Some(REPLACEMENT_CHARACTER))
                        },
                    }
                }
            },
        }
    }

    /// Read the next line but not include the tailing line character (or line chracters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123 456".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some(" 中文 ".as_bytes()), sc.next_line().unwrap());
    /// ```
    pub fn next_line(&mut self) -> Result<Option<&'a [u8]>, ScannerError> {
        if self.position == self.data_length {
            return Ok(None);
        }

        let mut p = self.position;

        loop {
            let e = self.data[p];

            let width = get_width(e);

            match width {
                0 => {
                    p += 1;
                },
                1 => {
                    match e {
                        b'\n' => {
                            let data = &self.data[self.position..p];

                            if p + 1 < self.data_length && self.data[p + 1] == b'\r' {
                                self.position = p + 2;
                            } else {
                                self.position = p + 1;
                            }

                            return Ok(Some(data));
                        },
                        b'\r' => {
                            let data = &self.data[self.position..p];

                            if p + 1 < self.data_length && self.data[p + 1] == b'\n' {
                                self.position = p + 2;
                            } else {
                                self.position = p + 1;
                            }

                            return Ok(Some(data));
                        },
                        _ => (),
                    }

                    p += 1;
                },
                _ => {
                    if p + width >= self.data_length {
                        let data = &self.data[self.position..];

                        self.position = self.data_length;

                        return Ok(Some(data));
                    } else {
                        p += width;
                    }
                },
            }

            if p == self.data_length {
                break;
            }
        }

        let data = &self.data[self.position..p];

        self.position = p;

        Ok(Some(data))
    }
}

impl<'a> ScannerU8Slice<'a> {
    /// Skip the next whitespaces (`javaWhitespace`). If there is nothing to read, it will return `Ok(false)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2   c".as_bytes());
    ///
    /// assert_eq!(Some('1'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('2'), sc.next_char().unwrap());
    /// assert_eq!(true, sc.skip_whitespaces().unwrap());
    /// assert_eq!(Some('c'), sc.next_char().unwrap());
    /// assert_eq!(false, sc.skip_whitespaces().unwrap());
    /// ```
    pub fn skip_whitespaces(&mut self) -> Result<bool, ScannerError> {
        if self.position == self.data_length {
            return Ok(false);
        }

        loop {
            let e = self.data[self.position];

            let width = get_width(e);

            match width {
                0 => {
                    break;
                },
                1 => {
                    if !is_whitespace_1(e) {
                        break;
                    }

                    self.position += 1;
                },
                3 => {
                    if self.position + width <= self.data_length
                        && is_whitespace_3(
                            self.data[self.position],
                            self.data[self.position + 1],
                            self.data[self.position + 2],
                        )
                    {
                        self.position += 3;
                    } else {
                        break;
                    }
                },
                _ => {
                    break;
                },
            }

            if self.position == self.data_length {
                break;
            }
        }

        Ok(true)
    }

    /// Read the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some("456".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some("789".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some("中文".as_bytes()), sc.next().unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<&'a [u8]>, ScannerError> {
        if !self.skip_whitespaces()? {
            return Ok(None);
        }

        if self.position == self.data_length {
            return Ok(None);
        }

        let mut p = self.position;

        loop {
            let e = self.data[p];

            let width = get_width(e);

            match width {
                0 => {
                    p += 1;
                },
                1 => {
                    if is_whitespace_1(e) {
                        let data = &self.data[self.position..p];

                        self.position = p;

                        return Ok(Some(data));
                    }

                    p += 1;
                },
                3 => {
                    if self.position + width > self.data_length {
                        let data = &self.data[self.position..];

                        self.position = self.data_length;

                        return Ok(Some(data));
                    } else if is_whitespace_3(
                        self.data[self.position],
                        self.data[self.position + 1],
                        self.data[self.position + 2],
                    ) {
                        let data = &self.data[self.position..p];

                        self.position = p;

                        return Ok(Some(data));
                    } else {
                        p += 3;
                    }
                },
                _ => {
                    if self.position + width >= self.data_length {
                        let data = &self.data[self.position..];

                        self.position = self.data_length;

                        return Ok(Some(data));
                    } else {
                        p += width;
                    }
                },
            }

            if p == self.data_length {
                break;
            }
        }

        let data = &self.data[self.position..p];

        self.position = p;

        Ok(Some(data))
    }
}

impl<'a> ScannerU8Slice<'a> {
    /// Read the next bytes. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".as_bytes()), sc.next_bytes(3).unwrap());
    /// assert_eq!(Some(" 456".as_bytes()), sc.next_bytes(4).unwrap());
    /// assert_eq!(Some("\r\n789 ".as_bytes()), sc.next_bytes(6).unwrap());
    /// assert_eq!(Some("中文".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some(" ".as_bytes()), sc.next_bytes(2).unwrap());
    /// assert_eq!(None, sc.next_bytes(2).unwrap());
    /// ```
    pub fn next_bytes(
        &mut self,
        max_number_of_bytes: usize,
    ) -> Result<Option<&'a [u8]>, ScannerError> {
        if self.position == self.data_length {
            return Ok(None);
        }

        let dropping_bytes = max_number_of_bytes.min(self.data_length - self.position);

        let data = &self.data[self.position..(self.position + dropping_bytes)];

        self.position += dropping_bytes;

        Ok(Some(data))
    }

    /// Drop the next N bytes. If there is nothing to read, it will return `Ok(None)`. If there are something to read, it will return `Ok(Some(i))`. The `i` is the length of the actually dropped bytes.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_bytes(7).unwrap());
    /// assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some(1), sc.drop_next_bytes(1).unwrap());
    /// assert_eq!(Some(" 中文 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_bytes(1).unwrap());
    /// ```
    pub fn drop_next_bytes(
        &mut self,
        max_number_of_bytes: usize,
    ) -> Result<Option<usize>, ScannerError> {
        if self.position == self.data_length {
            return Ok(None);
        }

        let dropping_bytes = max_number_of_bytes.min(self.data_length - self.position);

        self.position += dropping_bytes;

        Ok(Some(dropping_bytes))
    }
}

impl<'a> ScannerU8Slice<'a> {
    /// Read the next data until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".as_bytes()), sc.next_until(" ").unwrap());
    /// assert_eq!(Some("456\r".as_bytes()), sc.next_until("\n").unwrap());
    /// assert_eq!(Some("78".as_bytes()), sc.next_until("9 ").unwrap());
    /// assert_eq!(Some("\n\n 中文 ".as_bytes()), sc.next_until("kk").unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    pub fn next_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<&'a [u8]>, ScannerError> {
        if self.position == self.data_length {
            return Ok(None);
        }

        let boundary = boundary.as_ref();
        let boundary_length = boundary.len();

        if boundary_length == 0 || boundary_length >= self.data_length - self.position {
            let data = &self.data[self.position..];

            self.position = self.data_length;

            return Ok(Some(data));
        }

        for i in self.position..(self.data_length - boundary_length) {
            let e = i + boundary_length;

            if &self.data[i..e] == boundary {
                let data = &self.data[self.position..i];

                self.position = e;

                return Ok(Some(data));
            }
        }

        let data = &self.data[self.position..];

        self.position = self.data_length;

        Ok(Some(data))
    }
}

impl<'a> ScannerU8Slice<'a> {
    #[inline]
    fn next_parse<T: FromStr>(&mut self) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next()?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `u8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u8().unwrap());
    /// assert_eq!(Some(2), sc.next_u8().unwrap());
    /// ```
    #[inline]
    pub fn next_u8(&mut self) -> Result<Option<u8>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `u16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u16().unwrap());
    /// assert_eq!(Some(2), sc.next_u16().unwrap());
    /// ```
    #[inline]
    pub fn next_u16(&mut self) -> Result<Option<u16>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `u32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u32().unwrap());
    /// assert_eq!(Some(2), sc.next_u32().unwrap());
    /// ```
    #[inline]
    pub fn next_u32(&mut self) -> Result<Option<u32>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `u64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u64().unwrap());
    /// assert_eq!(Some(2), sc.next_u64().unwrap());
    /// ```
    #[inline]
    pub fn next_u64(&mut self) -> Result<Option<u64>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `u128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u128().unwrap());
    /// assert_eq!(Some(2), sc.next_u128().unwrap());
    /// ```
    #[inline]
    pub fn next_u128(&mut self) -> Result<Option<u128>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `usize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_usize().unwrap());
    /// assert_eq!(Some(2), sc.next_usize().unwrap());
    /// ```
    #[inline]
    pub fn next_usize(&mut self) -> Result<Option<usize>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `i8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i8().unwrap());
    /// assert_eq!(Some(2), sc.next_i8().unwrap());
    /// ```
    #[inline]
    pub fn next_i8(&mut self) -> Result<Option<i8>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `i16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i16().unwrap());
    /// assert_eq!(Some(2), sc.next_i16().unwrap());
    /// ```
    #[inline]
    pub fn next_i16(&mut self) -> Result<Option<i16>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `i32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i32().unwrap());
    /// assert_eq!(Some(2), sc.next_i32().unwrap());
    /// ```
    #[inline]
    pub fn next_i32(&mut self) -> Result<Option<i32>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `i64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i64().unwrap());
    /// assert_eq!(Some(2), sc.next_i64().unwrap());
    /// ```
    #[inline]
    pub fn next_i64(&mut self) -> Result<Option<i64>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `i128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i128().unwrap());
    /// assert_eq!(Some(2), sc.next_i128().unwrap());
    /// ```
    #[inline]
    pub fn next_i128(&mut self) -> Result<Option<i128>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `isize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_isize().unwrap());
    /// assert_eq!(Some(2), sc.next_isize().unwrap());
    /// ```
    #[inline]
    pub fn next_isize(&mut self) -> Result<Option<isize>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `f32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f32().unwrap());
    /// assert_eq!(Some(2.5), sc.next_f32().unwrap());
    /// ```
    #[inline]
    pub fn next_f32(&mut self) -> Result<Option<f32>, ScannerError> {
        self.next_parse()
    }

    /// Read the next token separated by whitespaces and parse it to a `f64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f64().unwrap());
    /// assert_eq!(Some(2.5), sc.next_f64().unwrap());
    /// ```
    #[inline]
    pub fn next_f64(&mut self) -> Result<Option<f64>, ScannerError> {
        self.next_parse()
    }
}

impl<'a> ScannerU8Slice<'a> {
    #[inline]
    fn next_until_parse<T: FromStr, D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_until(boundary)?;

        match result {
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(s) }.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u8_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u8_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u8_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u8>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u16_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u16_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u16_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u16>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u32_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u32_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u32>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u64_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u64_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u64>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_u128_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u128_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u128_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<u128>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `usize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_usize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_usize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_usize_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<usize>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i8_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i8_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i8_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i8>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i16_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i16_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i16_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i16>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i32_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i32_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i32>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i64_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i64_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i64>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_i128_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i128_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i128_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<i128>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `isize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2".as_bytes());
    ///
    /// assert_eq!(Some(1), sc.next_isize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_isize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_isize_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<isize>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `f32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f32_until(" ").unwrap());
    /// assert_eq!(Some(2.5), sc.next_f32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_f32_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<f32>, ScannerError> {
        self.next_until_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `f64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8Slice;
    ///
    /// let mut sc = ScannerU8Slice::new("1 2.5".as_bytes());
    ///
    /// assert_eq!(Some(1.0), sc.next_f64_until(" ").unwrap());
    /// assert_eq!(Some(2.5), sc.next_f64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_f64_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<f64>, ScannerError> {
        self.next_until_parse(boundary)
    }
}

impl<'a> Iterator for ScannerU8Slice<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next().unwrap_or(None)
    }
}
