use std::{
    char::REPLACEMENT_CHARACTER,
    str::{FromStr, from_utf8_unchecked},
};

use crate::{ScannerError, whitespaces::*};

/// A simple text scanner which can in-memory-ly parse primitive types and strings using ASCII from a byte slice.
#[derive(Debug)]
pub struct ScannerU8SliceAscii<'a> {
    data:        &'a [u8],
    data_length: usize,
    position:    usize,
}

impl<'a> ScannerU8SliceAscii<'a> {
    /// Create a scanner from in-memory bytes.
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new(b"123 456");
    /// ```
    #[inline]
    pub fn new<D: ?Sized + AsRef<[u8]>>(data: &D) -> ScannerU8SliceAscii<'_> {
        let data = data.as_ref();

        ScannerU8SliceAscii {
            data,
            data_length: data.len(),
            position: 0,
        }
    }
}

impl<'a> ScannerU8SliceAscii<'a> {
    /// Read the next char. If the data is not a correct char, it will return a `Ok(Some(REPLACEMENT_CHARACTER))` which is �. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("5 c ab".as_bytes());
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
        if self.position == self.data_length {
            return Ok(None);
        }

        let e = self.data[self.position];

        self.position += 1;

        if e >= 128 { Ok(Some(REPLACEMENT_CHARACTER)) } else { Ok(Some(e as char)) }
    }

    /// Read the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123 456".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some(" ab ".as_bytes()), sc.next_line().unwrap());
    /// ```
    pub fn next_line(&mut self) -> Result<Option<&'a [u8]>, ScannerError> {
        if self.position == self.data_length {
            return Ok(None);
        }

        let mut p = self.position;

        loop {
            let e = self.data[p];

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

            if p == self.data_length {
                break;
            }
        }

        let data = &self.data[self.position..p];

        self.position = p;

        Ok(Some(data))
    }
}

impl<'a> ScannerU8SliceAscii<'a> {
    /// Skip the next whitespaces (`javaWhitespace`). If there is nothing to read, it will return `Ok(false)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("1 2   c".as_bytes());
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
            if !is_whitespace_1(self.data[self.position]) {
                break;
            }

            self.position += 1;

            if self.position == self.data_length {
                break;
            }
        }

        Ok(true)
    }

    /// Read the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some("456".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some("789".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some("ab".as_bytes()), sc.next().unwrap());
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
            if is_whitespace_1(self.data[p]) {
                let data = &self.data[self.position..p];

                self.position = p;

                return Ok(Some(data));
            }

            p += 1;

            if p == self.data_length {
                break;
            }
        }

        let data = &self.data[self.position..p];

        self.position = p;

        Ok(Some(data))
    }
}

impl<'a> ScannerU8SliceAscii<'a> {
    /// Read the next bytes. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".as_bytes()), sc.next_bytes(3).unwrap());
    /// assert_eq!(Some(" 456".as_bytes()), sc.next_bytes(4).unwrap());
    /// assert_eq!(Some("\r\n789 ".as_bytes()), sc.next_bytes(6).unwrap());
    /// assert_eq!(Some("ab".as_bytes()), sc.next().unwrap());
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

    /// Drop the next N bytes. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length of the actually dropped bytes.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_bytes(7).unwrap());
    /// assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some(1), sc.drop_next_bytes(1).unwrap());
    /// assert_eq!(Some(" ab ".as_bytes()), sc.next_line().unwrap());
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

impl<'a> ScannerU8SliceAscii<'a> {
    /// Read the next data until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some("123".as_bytes()), sc.next_until(" ").unwrap());
    /// assert_eq!(Some("456\r".as_bytes()), sc.next_until("\n").unwrap());
    /// assert_eq!(Some("78".as_bytes()), sc.next_until("9 ").unwrap());
    /// assert_eq!(Some("\n\n ab ".as_bytes()), sc.next_until("kk").unwrap());
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

        if boundary_length == 0 || boundary_length > self.data_length - self.position {
            let data = &self.data[self.position..];

            self.position = self.data_length;

            return Ok(Some(data));
        }

        for i in self.position..=(self.data_length - boundary_length) {
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

impl<'a> ScannerU8SliceAscii<'a> {
    #[inline]
    fn next_parse<T: FromStr>(&mut self) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next()?;

        match result {
            // SAFETY: for malformed input `s` may not be valid UTF-8 (technically UB to treat as a `&str`), but it is only fed to a primitive `FromStr` that reads it as bytes, so an invalid token merely fails to parse; validation is skipped for speed.
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(s) }.parse()?)),
            None => Ok(None),
        }
    }
}

impl<'a> ScannerU8SliceAscii<'a> {
    #[inline]
    fn next_until_parse<T: FromStr, D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_until(boundary)?;

        match result {
            // SAFETY: for malformed input `s` may not be valid UTF-8 (technically UB to treat as a `&str`), but it is only fed to a primitive `FromStr` that reads it as bytes, so an invalid token merely fails to parse; validation is skipped for speed.
            Some(s) => Ok(Some(unsafe { from_utf8_unchecked(s) }.parse()?)),
            None => Ok(None),
        }
    }
}

macro_rules! scanner_u8_slice_ascii_number_methods {
    ($(($t:ty, $next:ident, $next_until:ident, $sample:literal, $v1:literal, $v2:literal)),+ $(,)?) => {
        impl<'a> ScannerU8SliceAscii<'a> {
            $(
                #[doc = concat!(
                    "Read the next token separated by whitespaces and parse it to a `", stringify!($t), "` value. If there is nothing to read, it will return `Ok(None)`.\n\n```rust\nuse scanner_rust::ScannerU8SliceAscii;\n\nlet mut sc = ScannerU8SliceAscii::new(", stringify!($sample), ".as_bytes());\n\nassert_eq!(Some(", stringify!($v1), "), sc.", stringify!($next), "().unwrap());\nassert_eq!(Some(", stringify!($v2), "), sc.", stringify!($next), "().unwrap());\n```"
                )]
                #[inline]
                pub fn $next(&mut self) -> Result<Option<$t>, ScannerError> {
                    self.next_parse()
                }
            )+
        }

        impl<'a> ScannerU8SliceAscii<'a> {
            $(
                #[doc = concat!(
                    "Read the next text until it reaches a specific boundary and parse it to a `", stringify!($t), "` value. If there is nothing to read, it will return `Ok(None)`.\n\n```rust\nuse scanner_rust::ScannerU8SliceAscii;\n\nlet mut sc = ScannerU8SliceAscii::new(", stringify!($sample), ".as_bytes());\n\nassert_eq!(Some(", stringify!($v1), "), sc.", stringify!($next_until), "(\" \").unwrap());\nassert_eq!(Some(", stringify!($v2), "), sc.", stringify!($next_until), "(\" \").unwrap());\n```"
                )]
                #[inline]
                pub fn $next_until<D: ?Sized + AsRef<[u8]>>(
                    &mut self,
                    boundary: &D,
                ) -> Result<Option<$t>, ScannerError> {
                    self.next_until_parse(boundary)
                }
            )+
        }
    };
}

scanner_u8_slice_ascii_number_methods! {
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

impl<'a> ScannerU8SliceAscii<'a> {
    /// Drop the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped line.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_line().unwrap());
    /// assert_eq!(Some("789 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_line().unwrap());
    /// assert_eq!(Some(" ab ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_line().unwrap());
    /// ```
    #[inline]
    pub fn drop_next_line(&mut self) -> Result<Option<usize>, ScannerError> {
        Ok(self.next_line()?.map(<[u8]>::len))
    }

    /// Drop the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped token.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("456".as_bytes()), sc.next().unwrap());
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("ab".as_bytes()), sc.next().unwrap());
    /// assert_eq!(None, sc.drop_next().unwrap());
    /// ```
    #[inline]
    pub fn drop_next(&mut self) -> Result<Option<usize>, ScannerError> {
        Ok(self.next()?.map(<[u8]>::len))
    }

    /// Drop the next data until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped data, excluding the boundary.
    ///
    /// ```rust
    /// use scanner_rust::ScannerU8SliceAscii;
    ///
    /// let mut sc = ScannerU8SliceAscii::new("123 456\r\n789 \n\n ab ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_until("\r\n").unwrap());
    /// assert_eq!(Some("789 ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_until("\n").unwrap());
    /// assert_eq!(Some(" ab ".as_bytes()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_until("").unwrap());
    /// ```
    #[inline]
    pub fn drop_next_until<D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<usize>, ScannerError> {
        Ok(self.next_until(boundary)?.map(<[u8]>::len))
    }
}

impl<'a> Iterator for ScannerU8SliceAscii<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next().unwrap_or(None)
    }
}
