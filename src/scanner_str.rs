use std::str::FromStr;

use utf8_width::*;

use crate::{ScannerError, whitespaces::*};

/// A simple text scanner which can in-memory-ly parse primitive types and strings using UTF-8 from a string slice.
#[derive(Debug)]
pub struct ScannerStr<'a> {
    text:        &'a str,
    text_length: usize,
    position:    usize,
}

impl<'a> ScannerStr<'a> {
    /// Create a scanner from a string.
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456");
    /// ```
    #[inline]
    pub fn new<S: ?Sized + AsRef<str>>(text: &S) -> ScannerStr<'_> {
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
            },
            _ => {
                let char_str = &self.text[self.position..(self.position + width)];

                self.position += width;

                Ok(char_str.chars().next())
            },
        }
    }

    /// Read the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
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
                        },
                        b'\r' => {
                            let text = &self.text[self.position..p];

                            if p + 1 < self.text_length && data[p + 1] == b'\n' {
                                self.position = p + 2;
                            } else {
                                self.position = p + 1;
                            }

                            return Ok(Some(text));
                        },
                        _ => (),
                    }

                    p += 1;
                },
                _ => {
                    p += width;
                },
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
                },
                3 => {
                    if !is_whitespace_3(
                        data[self.position],
                        data[self.position + 1],
                        data[self.position + 2],
                    ) {
                        break;
                    }

                    self.position += 3;
                },
                _ => {
                    break;
                },
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
                },
                3 => {
                    if is_whitespace_3(data[p], data[p + 1], data[p + 2]) {
                        let text = &self.text[self.position..p];

                        self.position = p;

                        return Ok(Some(text));
                    } else {
                        p += 3;
                    }
                },
                _ => {
                    p += width;
                },
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
    ) -> Result<Option<&'a str>, ScannerError> {
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

        if boundary_length == 0 || boundary_length > self.text_length - self.position {
            let text = &self.text[self.position..];

            self.position = self.text_length;

            return Ok(Some(text));
        }

        let data = self.text.as_bytes();

        for i in self.position..=(self.text_length - boundary_length) {
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
}

macro_rules! scanner_str_number_methods {
    ($(($t:ty, $next:ident, $next_until:ident, $sample:literal, $v1:literal, $v2:literal)),+ $(,)?) => {
        impl<'a> ScannerStr<'a> {
            $(
                #[doc = concat!(
                    "Read the next token separated by whitespaces and parse it to a `", stringify!($t), "` value. If there is nothing to read, it will return `Ok(None)`.\n\n```rust\nuse scanner_rust::ScannerStr;\n\nlet mut sc = ScannerStr::new(", stringify!($sample), ");\n\nassert_eq!(Some(", stringify!($v1), "), sc.", stringify!($next), "().unwrap());\nassert_eq!(Some(", stringify!($v2), "), sc.", stringify!($next), "().unwrap());\n```"
                )]
                #[inline]
                pub fn $next(&mut self) -> Result<Option<$t>, ScannerError> {
                    self.next_parse()
                }
            )+
        }

        impl<'a> ScannerStr<'a> {
            $(
                #[doc = concat!(
                    "Read the next text until it reaches a specific boundary and parse it to a `", stringify!($t), "` value. If there is nothing to read, it will return `Ok(None)`.\n\n```rust\nuse scanner_rust::ScannerStr;\n\nlet mut sc = ScannerStr::new(", stringify!($sample), ");\n\nassert_eq!(Some(", stringify!($v1), "), sc.", stringify!($next_until), "(\" \").unwrap());\nassert_eq!(Some(", stringify!($v2), "), sc.", stringify!($next_until), "(\" \").unwrap());\n```"
                )]
                #[inline]
                pub fn $next_until<S: AsRef<str>>(
                    &mut self,
                    boundary: S,
                ) -> Result<Option<$t>, ScannerError> {
                    self.next_raw_parse(boundary)
                }
            )+
        }
    };
}

scanner_str_number_methods! {
    (u8, next_u8, next_u8_until, "1 2", 1, 2),
    (u16, next_u16, next_u16_until, "1 2", 1, 2),
    (u32, next_u32, next_u32_until, "1 2", 1, 2),
    (u64, next_u64, next_u64_until, "1 2", 1, 2),
    (u128, next_u128, next_u128_until, "1 2", 1, 2),
    (usize, next_usize, next_usize_until, "1 2", 1, 2),
    (i8, next_i8, next_i8_until, "1 2", 1, 2),
    (i16, next_i16, next_i16_until, "1 2", 1, 2),
    (i32, next_i32, next_i32_until, "1 2", 1, 2),
    (i64, next_i64, next_i64_until, "1 2", 1, 2),
    (i128, next_i128, next_i128_until, "1 2", 1, 2),
    (isize, next_isize, next_isize_until, "1 2", 1, 2),
    (f32, next_f32, next_f32_until, "1 2.5", 1.0, 2.5),
    (f64, next_f64, next_f64_until, "1 2.5", 1.0, 2.5),
}

impl<'a> ScannerStr<'a> {
    /// Drop the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped line.
    ///
    /// ```rust
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some(7), sc.drop_next_line().unwrap());
    /// assert_eq!(Some("789 "), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_line().unwrap());
    /// assert_eq!(Some(" 中文 "), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_line().unwrap());
    /// ```
    #[inline]
    pub fn drop_next_line(&mut self) -> Result<Option<usize>, ScannerError> {
        Ok(self.next_line()?.map(str::len))
    }

    /// Drop the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped token.
    ///
    /// ```rust
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("456"), sc.next().unwrap());
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("中文"), sc.next().unwrap());
    /// assert_eq!(None, sc.drop_next().unwrap());
    /// ```
    #[inline]
    pub fn drop_next(&mut self) -> Result<Option<usize>, ScannerError> {
        Ok(self.next()?.map(str::len))
    }

    /// Drop the next text until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped text, excluding the boundary.
    ///
    /// ```rust
    /// use scanner_rust::ScannerStr;
    ///
    /// let mut sc = ScannerStr::new("123 456\r\n789 \n\n 中文 ");
    ///
    /// assert_eq!(Some(7), sc.drop_next_until("\r\n").unwrap());
    /// assert_eq!(Some("789 "), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_until("\n").unwrap());
    /// assert_eq!(Some(" 中文 "), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_until("").unwrap());
    /// ```
    #[inline]
    pub fn drop_next_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<usize>, ScannerError> {
        Ok(self.next_until(boundary)?.map(str::len))
    }
}

impl<'a> Iterator for ScannerStr<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next().unwrap_or(None)
    }
}
