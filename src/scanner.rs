use std::{
    char::REPLACEMENT_CHARACTER,
    fs::File,
    io::Read,
    path::Path,
    str::{FromStr, from_utf8},
};

use educe::Educe;
use utf8_width::*;

use crate::{
    ScannerError,
    kmp::compute_lps,
    reader::read,
    text::{into_string_lossy, numeric_text},
    whitespaces::*,
};

/// A simple text scanner which can parse primitive types and strings using UTF-8.
#[derive(Educe)]
#[educe(Debug)]
pub struct Scanner<R: Read, const N: usize = 256> {
    #[educe(Debug(ignore))]
    reader:       R,
    buf:          [u8; N],
    buf_length:   usize,
    buf_offset:   usize,
    passing_byte: Option<u8>,
}

impl<R: Read> Scanner<R> {
    /// Create a scanner from a reader.
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new(io::stdin());
    /// ```
    #[inline]
    pub fn new(reader: R) -> Scanner<R> {
        Self::new2(reader)
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    /// Create a scanner from a reader and set the buffer size via generics.
    ///
    /// The buffer size `N` must be at least 4 bytes.
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc: Scanner<_, 1024> = Scanner::new2(io::stdin());
    /// ```
    #[inline]
    pub fn new2(reader: R) -> Scanner<R, N> {
        // The buffer must be at least 4 bytes to hold a full UTF-8 character.
        const { assert!(N >= 4, "the buffer size N must be at least 4 bytes") };

        Scanner {
            reader,
            buf: [0u8; N],
            buf_length: 0,
            buf_offset: 0,
            passing_byte: None,
        }
    }
}

impl Scanner<File> {
    /// Create a scanner to read data from a file by its path.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::scan_path("Cargo.toml").unwrap();
    /// ```
    #[inline]
    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<Scanner<File>, ScannerError> {
        Self::scan_path2(path)
    }
}

impl<const N: usize> Scanner<File, N> {
    /// Create a scanner to read data from a file by its path and set the buffer size via generics.
    ///
    /// The buffer size `N` must be at least 4 bytes.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc: Scanner<_, 1024> = Scanner::scan_path2("Cargo.toml").unwrap();
    /// ```
    #[inline]
    pub fn scan_path2<P: AsRef<Path>>(path: P) -> Result<Scanner<File, N>, ScannerError> {
        let reader = File::open(path)?;

        Ok(Scanner::new2(reader))
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    #[inline]
    fn buf_align_to_front_end(&mut self) {
        self.buf.copy_within(self.buf_offset..(self.buf_offset + self.buf_length), 0);

        self.buf_offset = 0;
    }

    #[inline]
    fn buf_left_shift(&mut self, distance: usize) {
        debug_assert!(self.buf_length >= distance);

        self.buf_offset += distance;
        self.buf_length -= distance;

        if self.buf_offset >= N - 4 {
            self.buf_align_to_front_end();
        }
    }

    /// Left shift (if necessary) the buffer to remove bytes from the start of the buffer. Typically, you should use this after `peek`ing the buffer.
    ///
    /// # Safety
    ///
    /// `number_of_bytes` must not be greater than the length of the currently buffered data (the length of the slice returned by `peek`). A larger value underflows the internal buffer length and causes out-of-bounds access afterwards.
    #[inline]
    pub unsafe fn remove_heading_bytes_from_buffer(&mut self, number_of_bytes: usize) {
        self.buf_left_shift(number_of_bytes);
    }

    #[inline]
    fn buf_skip_line(&mut self, line_length: usize) {
        let ending = if self.buf[self.buf_offset + line_length] == b'\n' { b'\r' } else { b'\n' };
        let mut consumed = line_length + 1;

        if consumed == self.buf_length {
            // Defer the second byte when a line ending is split across reads.
            self.passing_byte = Some(ending);
        } else if self.buf[self.buf_offset + consumed] == ending {
            consumed += 1;
        }

        self.buf_left_shift(consumed);
    }

    #[inline]
    fn buf_fill_char(&mut self, width: usize) -> Result<usize, ScannerError> {
        while self.buf_length < width {
            let size =
                read(&mut self.reader, &mut self.buf[(self.buf_offset + self.buf_length)..])?;

            if size == 0 {
                // A sequence cut off by the end of data is handled one byte at a time.
                return Ok(1);
            }

            self.buf_length += size;
        }

        Ok(width)
    }

    #[inline]
    fn buf_raw_char_width(&mut self) -> Result<usize, ScannerError> {
        let width = get_width(self.buf[self.buf_offset]);

        if width <= 1 || self.buf_fill_char(width)? == 1 {
            return Ok(1);
        }

        // A broken sequence must not skip over a whitespace byte.
        if self.buf[(self.buf_offset + 1)..(self.buf_offset + width)]
            .iter()
            .all(|byte| byte & 0xC0 == 0x80)
        {
            Ok(width)
        } else {
            Ok(1)
        }
    }

    // Force inlining because this runs for every multi-byte character in hot loops.
    #[inline(always)]
    fn buf_push_char(&mut self, temp: &mut String, width: usize) {
        match from_utf8(&self.buf[self.buf_offset..(self.buf_offset + width)]) {
            Ok(char_str) => {
                temp.push_str(char_str);

                self.buf_left_shift(width);
            },
            Err(_) => {
                temp.push(REPLACEMENT_CHARACTER);

                self.buf_left_shift(1);
            },
        }
    }

    fn passing_read(&mut self) -> Result<bool, ScannerError> {
        if self.buf_length == 0 {
            let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(false);
            }

            self.buf_length += size;

            if let Some(passing_byte) = self.passing_byte.take()
                && self.buf[self.buf_offset] == passing_byte
            {
                self.buf_left_shift(1);

                return if size == 1 {
                    let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

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

            Ok(true)
        } else {
            Ok(true)
        }
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    /// Read the next char. If the data is not a correct char, it will return a `Ok(Some(REPLACEMENT_CHARACTER))` which is �. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("5 c 中文".as_bytes());
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
        if !self.passing_read()? {
            return Ok(None);
        }

        let e = self.buf[self.buf_offset];

        let width = get_width(e);

        match width {
            0 => {
                self.buf_left_shift(1);

                Ok(Some(REPLACEMENT_CHARACTER))
            },
            1 => {
                self.buf_left_shift(1);

                Ok(Some(e as char))
            },
            _ => {
                let width = self.buf_fill_char(width)?;
                let char_str_bytes = &self.buf[self.buf_offset..(self.buf_offset + width)];

                match from_utf8(char_str_bytes) {
                    Ok(char_str) => {
                        let c = char_str.chars().next();

                        self.buf_left_shift(width);

                        Ok(c)
                    },
                    Err(_) => {
                        self.buf_left_shift(1);

                        Ok(Some(REPLACEMENT_CHARACTER))
                    },
                }
            },
        }
    }

    /// Read the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123 456".into()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some("".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(" 中文 ".into()), sc.next_line().unwrap());
    /// ```
    pub fn next_line(&mut self) -> Result<Option<String>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut temp = String::new();

        loop {
            let e = self.buf[self.buf_offset];

            let width = get_width(e);

            match width {
                0 => {
                    self.buf_left_shift(1);

                    temp.push(REPLACEMENT_CHARACTER);
                },
                1 => {
                    if e == b'\n' || e == b'\r' {
                        self.buf_skip_line(0);

                        return Ok(Some(temp));
                    }

                    self.buf_left_shift(1);

                    temp.push(e as char);
                },
                _ => {
                    let width = self.buf_fill_char(width)?;

                    self.buf_push_char(&mut temp, width);
                },
            }

            if self.buf_length == 0 {
                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }
    }

    /// Read the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)) without fully validating UTF-8. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123 456".into()), sc.next_line_raw().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line_raw().unwrap());
    /// assert_eq!(Some("".into()), sc.next_line_raw().unwrap());
    /// assert_eq!(Some(" 中文 ".into()), sc.next_line_raw().unwrap());
    /// ```
    pub fn next_line_raw(&mut self) -> Result<Option<Vec<u8>>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut temp = Vec::new();

        loop {
            let data = &self.buf[self.buf_offset..(self.buf_offset + self.buf_length)];

            if let Some(line_length) = data.iter().position(|&byte| byte == b'\n' || byte == b'\r')
            {
                temp.extend_from_slice(&data[..line_length]);
                self.buf_skip_line(line_length);

                return Ok(Some(temp));
            }

            temp.extend_from_slice(data);
            self.buf_left_shift(self.buf_length);

            if !self.passing_read()? {
                return Ok(Some(temp));
            }
        }
    }

    /// Drop the next line but not include the trailing line character (or line characters like `CrLf`(`\r\n`)). If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped line.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_line().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_line().unwrap());
    /// assert_eq!(Some(" 中文 ".into()), sc.next_line().unwrap());
    /// assert_eq!(None, sc.drop_next_line().unwrap());
    /// ```
    pub fn drop_next_line(&mut self) -> Result<Option<usize>, ScannerError> {
        if !self.passing_read()? {
            return Ok(None);
        }

        let mut c = 0;

        loop {
            let data = &self.buf[self.buf_offset..(self.buf_offset + self.buf_length)];

            if let Some(line_length) = data.iter().position(|&byte| byte == b'\n' || byte == b'\r')
            {
                self.buf_skip_line(line_length);

                return Ok(Some(c + line_length));
            }

            c += self.buf_length;
            self.buf_left_shift(self.buf_length);

            if !self.passing_read()? {
                return Ok(Some(c));
            }
        }
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    /// Skip the next whitespaces (`javaWhitespace`). If there is nothing to read, it will return `Ok(false)`.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("1 2   c".as_bytes());
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
            let e = self.buf[self.buf_offset];

            let width = get_width(e);

            match width {
                0 => {
                    break;
                },
                1 => {
                    if !is_whitespace_1(e) {
                        break;
                    }

                    self.buf_left_shift(1);
                },
                3 if self.buf_fill_char(width)? == width
                    && is_whitespace_3(
                        self.buf[self.buf_offset],
                        self.buf[self.buf_offset + 1],
                        self.buf[self.buf_offset + 2],
                    ) =>
                {
                    self.buf_left_shift(3);
                },
                _ => {
                    break;
                },
            }

            if self.buf_length == 0 {
                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

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
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next().unwrap());
    /// assert_eq!(Some("456".into()), sc.next().unwrap());
    /// assert_eq!(Some("789".into()), sc.next().unwrap());
    /// assert_eq!(Some("中文".into()), sc.next().unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<String>, ScannerError> {
        if !self.skip_whitespaces()? || self.buf_length == 0 {
            return Ok(None);
        }

        let mut temp = String::new();

        loop {
            let e = self.buf[self.buf_offset];

            let width = get_width(e);

            match width {
                0 => {
                    self.buf_left_shift(1);

                    temp.push(REPLACEMENT_CHARACTER);
                },
                1 => {
                    if is_whitespace_1(e) {
                        return Ok(Some(temp));
                    }

                    self.buf_left_shift(1);

                    temp.push(e as char);
                },
                _ => {
                    let width = self.buf_fill_char(width)?;

                    if width == 3
                        && is_whitespace_3(
                            self.buf[self.buf_offset],
                            self.buf[self.buf_offset + 1],
                            self.buf[self.buf_offset + 2],
                        )
                    {
                        return Ok(Some(temp));
                    }

                    self.buf_push_char(&mut temp, width);
                },
            }

            if self.buf_length == 0 {
                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }
    }

    /// Read the next token separated by whitespaces without fully validating UTF-8. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some("456".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some("789".into()), sc.next_raw().unwrap());
    /// assert_eq!(Some("中文".into()), sc.next_raw().unwrap());
    /// assert_eq!(None, sc.next_raw().unwrap());
    /// ```
    pub fn next_raw(&mut self) -> Result<Option<Vec<u8>>, ScannerError> {
        if !self.skip_whitespaces()? || self.buf_length == 0 {
            return Ok(None);
        }

        let mut temp = Vec::new();

        loop {
            let width = self.buf_raw_char_width()?;

            if (width == 1 && is_whitespace_1(self.buf[self.buf_offset]))
                || (width == 3
                    && is_whitespace_3(
                        self.buf[self.buf_offset],
                        self.buf[self.buf_offset + 1],
                        self.buf[self.buf_offset + 2],
                    ))
            {
                return Ok(Some(temp));
            }

            temp.extend_from_slice(&self.buf[self.buf_offset..(self.buf_offset + width)]);
            self.buf_left_shift(width);

            if !self.passing_read()? {
                return Ok(Some(temp));
            }
        }
    }

    /// Drop the next token separated by whitespaces. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped token.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("456".into()), sc.next().unwrap());
    /// assert_eq!(Some(3), sc.drop_next().unwrap());
    /// assert_eq!(Some("中文".into()), sc.next().unwrap());
    /// assert_eq!(None, sc.drop_next().unwrap());
    /// ```
    pub fn drop_next(&mut self) -> Result<Option<usize>, ScannerError> {
        if !self.skip_whitespaces()? || self.buf_length == 0 {
            return Ok(None);
        }

        let mut c = 0;

        loop {
            let width = self.buf_raw_char_width()?;

            if (width == 1 && is_whitespace_1(self.buf[self.buf_offset]))
                || (width == 3
                    && is_whitespace_3(
                        self.buf[self.buf_offset],
                        self.buf[self.buf_offset + 1],
                        self.buf[self.buf_offset + 2],
                    ))
            {
                return Ok(Some(c));
            }

            c += width;
            self.buf_left_shift(width);

            if !self.passing_read()? {
                return Ok(Some(c));
            }
        }
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    /// Read the next bytes. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_bytes(3).unwrap());
    /// assert_eq!(Some(" 456".into()), sc.next_bytes(4).unwrap());
    /// assert_eq!(Some("\r\n789 ".into()), sc.next_bytes(6).unwrap());
    /// assert_eq!(Some("中文".into()), sc.next_raw().unwrap());
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
                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

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

    /// Drop the next N bytes. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length of the actually dropped bytes.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_bytes(7).unwrap());
    /// assert_eq!(Some("".into()), sc.next_line().unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(1), sc.drop_next_bytes(1).unwrap());
    /// assert_eq!(Some(" 中文 ".into()), sc.next_line().unwrap());
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
                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

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

impl<R: Read, const N: usize> Scanner<R, N> {
    /// Read the next text until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// An empty boundary reads all remaining data.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_until(" ").unwrap());
    /// assert_eq!(Some("456\r".into()), sc.next_until("\n").unwrap());
    /// assert_eq!(Some("78".into()), sc.next_until("9 ").unwrap());
    /// assert_eq!(Some("\n\n 中文 ".into()), sc.next_until("kk").unwrap());
    /// assert_eq!(None, sc.next().unwrap());
    /// ```
    pub fn next_until<S: AsRef<str>>(
        &mut self,
        boundary: S,
    ) -> Result<Option<String>, ScannerError> {
        let boundary = boundary.as_ref();

        Ok(self.next_until_raw(boundary.as_bytes())?.map(into_string_lossy))
    }

    /// Read the next data until it reaches a specific boundary without fully validating UTF-8. If there is nothing to read, it will return `Ok(None)`.
    ///
    /// An empty boundary reads all remaining data.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some("123".into()), sc.next_until_raw(" ").unwrap());
    /// assert_eq!(Some("456\r".into()), sc.next_until_raw("\n").unwrap());
    /// assert_eq!(Some("78".into()), sc.next_until_raw("9 ").unwrap());
    /// assert_eq!(Some("\n\n 中文 ".into()), sc.next_until_raw("kk").unwrap());
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

        // An empty boundary never matches, so keep reading until the end.
        if boundary_length == 0 {
            loop {
                temp.extend_from_slice(
                    &self.buf[self.buf_offset..(self.buf_offset + self.buf_length)],
                );

                self.buf_left_shift(self.buf_length);

                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(temp));
                }

                self.buf_length += size;
            }
        }

        // Match the boundary across buffer refills with KMP so a mismatch never re-reads bytes.
        let lps = compute_lps(boundary);
        let mut b = 0;

        loop {
            let mut i = 0;

            while i < self.buf_length {
                let e = self.buf[self.buf_offset + i];

                while b > 0 && e != boundary[b] {
                    b = lps[b - 1];
                }

                if e == boundary[b] {
                    b += 1;
                }

                i += 1;

                if b == boundary_length {
                    // The matched boundary is the last `boundary_length` bytes seen so far.
                    temp.extend_from_slice(&self.buf[self.buf_offset..(self.buf_offset + i)]);
                    temp.truncate(temp.len() - boundary_length);

                    self.buf_left_shift(i);

                    return Ok(Some(temp));
                }
            }

            temp.extend_from_slice(&self.buf[self.buf_offset..(self.buf_offset + self.buf_length)]);

            self.buf_left_shift(self.buf_length);

            let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(Some(temp));
            }

            self.buf_length += size;
        }
    }

    /// Drop the next data until it reaches a specific boundary. If there is nothing to read, it will return `Ok(None)`. If there is something to read, it will return `Ok(Some(i))`. The `i` is the length (in bytes) of the dropped data, excluding the boundary.
    ///
    /// An empty boundary drops all remaining data.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!(Some(7), sc.drop_next_until("\r\n").unwrap());
    /// assert_eq!(Some("789 ".into()), sc.next_line().unwrap());
    /// assert_eq!(Some(0), sc.drop_next_until("\n").unwrap());
    /// assert_eq!(Some(" 中文 ".into()), sc.next_line().unwrap());
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

        // An empty boundary never matches, so keep reading until the end.
        if boundary_length == 0 {
            loop {
                c += self.buf_length;

                self.buf_left_shift(self.buf_length);

                let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

                if size == 0 {
                    return Ok(Some(c));
                }

                self.buf_length += size;
            }
        }

        // Match the boundary across buffer refills with KMP so a mismatch never re-reads bytes.
        let lps = compute_lps(boundary);
        let mut b = 0;

        loop {
            let mut i = 0;

            while i < self.buf_length {
                let e = self.buf[self.buf_offset + i];

                while b > 0 && e != boundary[b] {
                    b = lps[b - 1];
                }

                if e == boundary[b] {
                    b += 1;
                }

                i += 1;

                if b == boundary_length {
                    self.buf_left_shift(i);

                    return Ok(Some(c + i - boundary_length));
                }
            }

            c += self.buf_length;

            self.buf_left_shift(self.buf_length);

            let size = read(&mut self.reader, &mut self.buf[self.buf_offset..])?;

            if size == 0 {
                return Ok(Some(c));
            }

            self.buf_length += size;
        }
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    /// Fill up the buffer as much as possible and return an immutable slice of all the currently buffered (unread) data.
    /// Reading stops at the end of the stream or when the buffer is full. If `shift` is `true`, the buffered data is first moved to the front of the buffer so that up to the whole buffer size can be filled; if `false`, only the space after the current read position is filled.
    ///
    /// ```rust
    /// use scanner_rust::Scanner;
    ///
    /// let mut sc = Scanner::new("123 456\r\n789 \n\n 中文 ".as_bytes());
    ///
    /// assert_eq!("123 456\r\n789 \n\n 中文 ".as_bytes(), sc.peek(false).unwrap());
    /// ```
    #[inline]
    pub fn peek(&mut self, shift: bool) -> Result<&[u8], ScannerError> {
        // Consume any line-terminator byte deferred by a previous read so it is not peeked again.
        if !self.passing_read()? {
            return Ok(&[]);
        }

        if shift {
            self.buf_align_to_front_end();
        }

        while self.buf_offset + self.buf_length < N {
            let size =
                read(&mut self.reader, &mut self.buf[(self.buf_offset + self.buf_length)..])?;

            if size == 0 {
                break;
            }

            self.buf_length += size;
        }

        Ok(&self.buf[self.buf_offset..(self.buf_offset + self.buf_length)])
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    #[inline]
    fn next_raw_parse<T: FromStr>(&mut self) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_raw()?;

        match result {
            Some(s) => Ok(Some(numeric_text(&s).parse()?)),
            None => Ok(None),
        }
    }
}

impl<R: Read, const N: usize> Scanner<R, N> {
    #[inline]
    fn next_until_raw_parse<T: FromStr, D: ?Sized + AsRef<[u8]>>(
        &mut self,
        boundary: &D,
    ) -> Result<Option<T>, ScannerError>
    where
        ScannerError: From<<T as FromStr>::Err>, {
        let result = self.next_until_raw(boundary)?;

        match result {
            Some(s) => Ok(Some(numeric_text(&s).parse()?)),
            None => Ok(None),
        }
    }
}

macro_rules! scanner_number_methods {
    ($(($t:ty, $next:ident, $next_until:ident, $sample:literal, $v1:literal, $v2:literal)),+ $(,)?) => {
        impl<R: Read, const N: usize> Scanner<R, N> {
            $(
                #[doc = concat!(
                    "Read the next token separated by whitespaces and parse it to a `", stringify!($t), "` value. If there is nothing to read, it will return `Ok(None)`.\n\n```rust\nuse scanner_rust::Scanner;\n\nlet mut sc = Scanner::new(", stringify!($sample), ".as_bytes());\n\nassert_eq!(Some(", stringify!($v1), "), sc.", stringify!($next), "().unwrap());\nassert_eq!(Some(", stringify!($v2), "), sc.", stringify!($next), "().unwrap());\n```"
                )]
                #[inline]
                pub fn $next(&mut self) -> Result<Option<$t>, ScannerError> {
                    self.next_raw_parse()
                }
            )+
        }

        impl<R: Read, const N: usize> Scanner<R, N> {
            $(
                #[doc = concat!(
                    "Read the next text until it reaches a specific boundary and parse it to a `", stringify!($t), "` value. If there is nothing to read, it will return `Ok(None)`. An empty boundary reads all remaining data before parsing.\n\n```rust\nuse scanner_rust::Scanner;\n\nlet mut sc = Scanner::new(", stringify!($sample), ".as_bytes());\n\nassert_eq!(Some(", stringify!($v1), "), sc.", stringify!($next_until), "(\" \").unwrap());\nassert_eq!(Some(", stringify!($v2), "), sc.", stringify!($next_until), "(\" \").unwrap());\n```"
                )]
                #[inline]
                pub fn $next_until<D: ?Sized + AsRef<[u8]>>(
                    &mut self,
                    boundary: &D,
                ) -> Result<Option<$t>, ScannerError> {
                    self.next_until_raw_parse(boundary)
                }
            )+
        }
    };
}

scanner_number_methods! {
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
