use std::str::FromStr;

use crate::utf8_width::*;
use crate::whitespaces::*;
use crate::ScannerError;

/// A simple text scanner which can in-memory-ly parse primitive types and strings using UTF-8 from a string slice.
#[derive(Debug)]
pub struct ScannerStr<'a> {
    text: &'a str,
    text_length: usize,
    position: usize,
}

impl<'a> ScannerStr<'a> {
    /// Create a scanner from a string.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use std::io;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456");
    /// ```
    #[inline]
    pub fn new<S: ?Sized + AsRef<str>>(text: &S) -> ScannerStr {
        let text = text.as_ref();

        ScannerStr {
            text,
            text_length: text.len(),
            position: 0,
        }
    }
}

impl<'a> ScannerStr<'a> {
    /// Read the next char. If the data is not a correct char, it will return a `Ok(Some(REPLACEMENT_CHARACTER))` which is �. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("5 c 中文");
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
        if self.position == self.text_length {
            return Ok(None);
        }

        let data = self.text.as_bytes();

        let e = data[self.position];

        let width = unsafe { get_width_assume_valid(e) };

        match width {
            1 => {
                self.position += 1;

                Ok(Some(e as char))
            }
            _ => {
                let char_str = &self.text[self.position..(self.position + width)];

                self.position += width;

                Ok(char_str.chars().next())
            }
        }
    }

    /// Read the next line but not include the tailing line character (or line chracters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some("123 456"), sc.next_line().unwrap());
    /// assert_eq!(Some("789 "), sc.next_line().unwrap());
    /// assert_eq!(Some(""), sc.next_line().unwrap());
    /// assert_eq!(Some(" 中文 "), sc.next_line().unwrap());
    /// ```
    pub fn next_line(&mut self) -> Result<Option<&'a str>, ScannerError> {
        if self.position == self.text_length {
            return Ok(None);
        }

        let data = self.text.as_bytes();

        let mut p = self.position;

        loop {
            let e = data[p];

            let width = unsafe { get_width_assume_valid(e) };

            match width {
                1 => {
                    match e {
                        b'\n' => {
                            let text = &self.text[self.position..p];

                            if p + 1 < self.text_length && data[p + 1] == b'\r' {
                                self.position = p + 2;
                            } else {
                                self.position = p + 1;
                            }

                            return Ok(Some(text));
                        }
                        b'\r' => {
                            let text = &self.text[self.position..p];

                            if p + 1 < self.text_length && data[p + 1] == b'\n' {
                                self.position = p + 2;
                            } else {
                                self.position = p + 1;
                            }

                            return Ok(Some(text));
                        }
                        _ => (),
                    }

                    p += 1;
                }
                _ => {
                    p += width;
                }
            }

            if p == self.text_length {
                break;
            }
        }

        let text = &self.text[self.position..p];

        self.position = p;

        Ok(Some(text))
    }
}

impl<'a> ScannerStr<'a> {
    /// Skip the next whitespaces (`javaWhitespace`). If there is nothing to read, it will return `Ok(false)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2   c");
    ///
    /// assert_eq!(Some('1'), sc.next_char().unwrap());
    /// assert_eq!(Some(' '), sc.next_char().unwrap());
    /// assert_eq!(Some('2'), sc.next_char().unwrap());
    /// assert_eq!(true, sc.skip_whitespaces().unwrap());
    /// assert_eq!(Some('c'), sc.next_char().unwrap());
    /// assert_eq!(false, sc.skip_whitespaces().unwrap());
    /// ```
    pub fn skip_whitespaces(&mut self) -> Result<bool, ScannerError> {
        if self.position == self.text_length {
            return Ok(false);
        }

        let data = self.text.as_bytes();

        loop {
            let e = data[self.position];

            let width = unsafe { get_width_assume_valid(e) };

            match width {
                1 => {
                    if !is_whitespace_1(e) {
                        break;
                    }

                    self.position += 1;
                }
                3 => {
                    if !is_whitespace_3(
                        data[self.position],
                        data[self.position + 1],
                        data[self.position + 2],
                    ) {
                        break;
                    }

                    self.position += 3;
                }
                _ => {
                    break;
                }
            }

            if self.position == self.text_length {
                break;
            }
        }

        Ok(true)
    }

    /// Read the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some("123"), sc.next().unwrap());
    /// assert_eq!(Some("456"), sc.next().unwrap());
    /// assert_eq!(Some("789"), sc.next().unwrap());
    /// assert_eq!(Some("中文"), sc.next().unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<&'a str>, ScannerError> {
        if !self.skip_whitespaces()? {
            return Ok(None);
        }

        if self.position == self.text_length {
            return Ok(None);
        }

        let data = self.text.as_bytes();

        let mut p = self.position;

        loop {
            let e = data[p];

            let width = unsafe { get_width_assume_valid(e) };

            match width {
                1 => {
                    if is_whitespace_1(e) {
                        let text = &self.text[self.position..p];

                        self.position = p;

                        return Ok(Some(text));
                    }

                    p += 1;
                }
                3 => {
                    if is_whitespace_3(
                        data[self.position],
                        data[self.position + 1],
                        data[self.position + 2],
                    ) {
                        let text = &self.text[self.position..p];

                        self.position = p;

                        return Ok(Some(text));
                    } else {
                        p += 3;
                    }
                }
                _ => {
                    p += width;
                }
            }

            if p == self.text_length {
                break;
            }
        }

        let text = &self.text[self.position..p];

        self.position = p;

        Ok(Some(text))
    }
}

impl<'a> ScannerStr<'a> {
    /// Read the next text (as a string slice) with a specific max number of characters. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some("123"), sc.next_str(3).unwrap());
    /// assert_eq!(Some(" 456"), sc.next_str(4).unwrap());
    /// assert_eq!(Some("\r\n789 "), sc.next_str(6).unwrap());
    /// assert_eq!(Some("\n\n 中"), sc.next_str(4).unwrap());
    /// assert_eq!(Some("文"), sc.next().unwrap());
    /// assert_eq!(Some(" "), sc.next_str(2).unwrap());
    /// assert_eq!(None, sc.next_str(2).unwrap());
    /// ```
    pub fn next_str(
        &mut self,
        max_number_of_characters: usize,
    ) -> Result<Option<&str>, ScannerError> {
        if self.position == self.text_length {
            return Ok(None);
        }

        let data = self.text.as_bytes();

        let mut p = self.position;
        let mut c = 0;

        while c < max_number_of_characters {
            let width = unsafe { get_width_assume_valid(data[p]) };

            p += width;

            c += 1;

            if p == self.text_length {
                break;
            }
        }

        let text = &self.text[self.position..p];

        self.position = p;

        Ok(Some(text))
    }
}

impl<'a> ScannerStr<'a> {
    /// Read the next text until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some("123"), sc.next_until(" ").unwrap());
    /// assert_eq!(Some("456\r"), sc.next_until("\n").unwrap());
    /// assert_eq!(Some("78"), sc.next_until("9 ").unwrap());
    /// assert_eq!(Some("\n\n 中文 "), sc.next_until("kk").unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    pub fn next_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<&'a str>, ScannerError> {
        if self.position == self.text_length {
            return Ok(None);
        }

        let boundary = boundary.as_ref().as_bytes();
        let boundary_length = boundary.len();

        if boundary_length == 0 || boundary_length >= self.text_length - self.position {
            let text = &self.text[self.position..];

            self.position = self.text_length;

            return Ok(Some(text));
        }

        let data = self.text.as_bytes();

        for i in self.position..(self.text_length - boundary_length) {
            let e = i + boundary_length;

            if &data[i..e] == boundary {
                let text = &self.text[self.position..i];

                self.position = e;

                return Ok(Some(text));
            }
        }

        let text = &self.text[self.position..];

        self.position = self.text_length;

        Ok(Some(text))
    }
}

impl<'a> ScannerStr<'a> {
    #[inline]
    fn next_parse<T: FromStr>(&mut self) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next()?;

        match result {
            Some(s) => Ok(Some(s.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next token separated by whitespaces and parse it to a `u8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2.5");
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
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2.5");
    ///
    /// assert_eq!(Some(1.0), sc.next_f64().unwrap());
    /// assert_eq!(Some(2.5), sc.next_f64().unwrap());
    /// ```
    #[inline]
    pub fn next_f64(&mut self) -> Result<Option<f64>, ScannerError> {
        self.next_parse()
    }
}

impl<'a> ScannerStr<'a> {
    #[inline]
    fn next_raw_parse<T: FromStr, S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_until(boundary)?;

        match result {
            Some(s) => Ok(Some(s.parse()?)),
            None => Ok(None),
        }
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_u8_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u8_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u8_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<u8>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_u16_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u16_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u16_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<u16>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_u32_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u32_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<u32>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_u64_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u64_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<u64>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `u128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_u128_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_u128_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_u128_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<u128>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `usize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_usize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_usize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_usize_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<usize>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i8` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_i8_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i8_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i8_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<i8>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i16` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_i16_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i16_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i16_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<i16>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_i32_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i32_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<i32>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_i64_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i64_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<i64>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `i128` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_i128_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_i128_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_i128_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<i128>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `isize` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2");
    ///
    /// assert_eq!(Some(1), sc.next_isize_until(" ").unwrap());
    /// assert_eq!(Some(2), sc.next_isize_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_isize_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<isize>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `f32` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2.5");
    ///
    /// assert_eq!(Some(1.0), sc.next_f32_until(" ").unwrap());
    /// assert_eq!(Some(2.5), sc.next_f32_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_f32_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<f32>, ScannerError> {
        self.next_raw_parse(boundary)
    }

    /// Read the next text until it reaches a specific boundary and parse it to a `f64` value. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// extern crate scanner_rust;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("1 2.5");
    ///
    /// assert_eq!(Some(1.0), sc.next_f64_until(" ").unwrap());
    /// assert_eq!(Some(2.5), sc.next_f64_until(" ").unwrap());
    /// ```
    #[inline]
    pub fn next_f64_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<f64>, ScannerError> {
        self.next_raw_parse(boundary)
    }
}

impl<'a> Iterator for ScannerStr<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next().unwrap_or(None)
    }
}
