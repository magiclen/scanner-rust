Scanner
====================

[![Build Status](https://travis-ci.org/magiclen/scanner-rust.svg?branch=master)](https://travis-ci.org/magiclen/scanner-rust)

This crate provides Java-like Scanners which can parse primitive types and strings using UTF-8 or ASCII.

### Scan a stream

`Scanner` or `ScannerAscii` can be used for reading strings or raw data from a stream.

```rust
extern crate scanner_rust;

use std::io::{self, Write};

use scanner_rust::ScannerAscii;

print!("Please input two integers, a and b: ");
io::stdout().flush().unwrap();

let mut sc = ScannerAscii::new(io::stdin());

let a = {
    loop {
        match sc.next_isize() {
            Ok(i) => break i.unwrap_or(0),
            Err(_) => {
                print!("Re-input a and b: ");
                io::stdout().flush().unwrap();
            }
        }
    }
};

let b = {
    loop {
        match sc.next_isize() {
            Ok(i) => break i.unwrap_or(0),
            Err(_) => {
                print!("Re-input b: ");
                io::stdout().flush().unwrap();
            }
        }
    }
};

println!("{} + {} = {}", a, b, a + b);
```

Besides, the `drop_next` and `drop_next_line` methods are useful when you want to skip some data.

The default buffer size is 256 bytes. If you want to change that, you can use the `new2` associated function or the `scan_path2` associated function and define a length explicitly to create an instance of the above structs.

For example, to change the buffer size to 64 bytes,

```rust
extern crate scanner_rust;

use scanner_rust::generic_array::typenum::U64;
use scanner_rust::Scanner;

let mut sc: Scanner<_, U64> = Scanner::scan_path2("Cargo.toml").unwrap();
```

### Scan a string slice (`&str`)

`ScannerStr` can be used for reading strings from a string slice.

```rust
extern crate scanner_rust;

use std::io::{self, Write};

use scanner_rust::ScannerStr;

let mut sc = ScannerStr::new(" 123   456.7    \t\r\n\n c中文字\n\tHello world!");

assert_eq!(Some(123), sc.next_u8().unwrap());
assert_eq!(Some(456.7), sc.next_f64().unwrap());
assert_eq!(Some(' '), sc.next_char().unwrap());
assert_eq!(Some(' '), sc.next_char().unwrap());
assert_eq!(true, sc.skip_whitespaces().unwrap());
assert_eq!(Some('c'), sc.next_char().unwrap());
assert_eq!(Some("中文字"), sc.next_line().unwrap());
assert_eq!(Some("\tHello world!".into()), sc.next_line().unwrap());
assert_eq!(None, sc.next_line().unwrap());
```

### Scan a u8 slice

`ScannerU8Slice` or `ScannerU8SliceAscii` can be used for reading raw data from a `u8` slice.

```rust
extern crate scanner_rust;

use std::io::{self, Write};

use scanner_rust::ScannerU8Slice;

let mut sc = ScannerU8Slice::new(" 123   456.7    \t\r\n\n c中文字\n\tHello world!".as_bytes());

assert_eq!(Some(123), sc.next_u8().unwrap());
assert_eq!(Some(456.7), sc.next_f64().unwrap());
assert_eq!(Some(' '), sc.next_char().unwrap());
assert_eq!(Some(' '), sc.next_char().unwrap());
assert_eq!(true, sc.skip_whitespaces().unwrap());
assert_eq!(Some('c'), sc.next_char().unwrap());
assert_eq!(Some("中文字".as_bytes()), sc.next_line().unwrap());
assert_eq!(Some("\tHello world!".as_bytes()), sc.next_line().unwrap());
assert_eq!(None, sc.next_line().unwrap());
```

## Crates.io

https://crates.io/crates/scanner-rust

## Documentation

https://docs.rs/scanner-rust

## License

[MIT](LICENSE)