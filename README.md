Scanner
====================

[![Build Status](https://travis-ci.org/magiclen/scanner-rust.svg?branch=master)](https://travis-ci.org/magiclen/scanner-rust)
[![Build status](https://ci.appveyor.com/api/projects/status/2iwlj6lyv6p26g81/branch/master?svg=true)](https://ci.appveyor.com/project/magiclen/scanner-rust/branch/master)

This crate provides a Java-like Scanner which can parse primitive types and strings using UTF-8.

## Example

```rust
extern crate scanner_rust;

use scanner_rust::Scanner;

let mut sc = Scanner::scan_slice(" 123   456.7    \t\r\n\n c中文字\n\tHello world!");

assert_eq!(Some(123), sc.next_u8().unwrap());
assert_eq!(Some(456.7), sc.next_f64().unwrap());
assert_eq!(Some(' '), sc.next_char().unwrap());
assert_eq!(Some(' '), sc.next_char().unwrap());
assert_eq!(true, sc.skip_whitespaces().unwrap());
assert_eq!(Some('c'), sc.next_char().unwrap());
assert_eq!(Some("中文字".into()), sc.next_line().unwrap());
assert_eq!(Some("\tHello world!".into()), sc.next_line().unwrap());
assert_eq!(None, sc.next_line().unwrap());
```

## Crates.io

https://crates.io/crates/scanner-rust

## Documentation

https://docs.rs/scanner-rust

## License

[MIT](LICENSE)