extern crate scanner_rust;

use scanner_rust::Scanner;
use std::io::Cursor;

#[test]
fn read_chars() {
    let data = "Hello, 123 中文好難。寝る";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some('H'), sc.next_char().unwrap());
    assert_eq!(Some('e'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('o'), sc.next_char().unwrap());
    assert_eq!(Some(','), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(Some('1'), sc.next_char().unwrap());
    assert_eq!(Some('2'), sc.next_char().unwrap());
    assert_eq!(Some('3'), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(Some('中'), sc.next_char().unwrap());
    assert_eq!(Some('文'), sc.next_char().unwrap());
    assert_eq!(Some('好'), sc.next_char().unwrap());
    assert_eq!(Some('難'), sc.next_char().unwrap());
    assert_eq!(Some('。'), sc.next_char().unwrap());
    assert_eq!(Some('寝'), sc.next_char().unwrap());
    assert_eq!(Some('る'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());

    let mut sc = Scanner::with_capacity(Cursor::new(data), 13);

    assert_eq!(Some('H'), sc.next_char().unwrap());
    assert_eq!(Some('e'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('o'), sc.next_char().unwrap());
    assert_eq!(Some(','), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(Some('1'), sc.next_char().unwrap());
    assert_eq!(Some('2'), sc.next_char().unwrap());
    assert_eq!(Some('3'), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(Some('中'), sc.next_char().unwrap());
    assert_eq!(Some('文'), sc.next_char().unwrap());
    assert_eq!(Some('好'), sc.next_char().unwrap());
    assert_eq!(Some('難'), sc.next_char().unwrap());
    assert_eq!(Some('。'), sc.next_char().unwrap());
    assert_eq!(Some('寝'), sc.next_char().unwrap());
    assert_eq!(Some('る'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
}

#[test]
fn next_lines() {
    let data = "Hello, 123 中文好難。寝る";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("Hello, 123 中文好難。寝る".into()), sc.next_line().unwrap());

    let data = "Hello, 123 中文好難。寝る\n";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("Hello, 123 中文好難。寝る".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "Hello, 123 中文好難。寝る\nHello, 123 中文好難。寝る\n\nHello, 123 中文好難。寝る\n \nHello, 123 中文好難。寝る";

    let mut sc = Scanner::with_capacity(Cursor::new(data), 13);

    assert_eq!(Some("Hello, 123 中文好難。寝る".into()), sc.next_line().unwrap());
    assert_eq!(Some("Hello, 123 中文好難。寝る".into()), sc.next_line().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(Some("Hello, 123 中文好難。寝る".into()), sc.next_line().unwrap());
    assert_eq!(Some(" ".into()), sc.next_line().unwrap());
    assert_eq!(Some("Hello, 123 中文好難。寝る".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
}

#[test]
fn next_lines_crlf() {
    let data = "123\r中文";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\r\n中文";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\r\r中文";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\n\r中文";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\n\n中文";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\r\n中文\r123\n456\n\r789\r";

    let mut sc = Scanner::scan_slice(data);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("456".into()), sc.next_line().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(Some("789".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let mut sc = Scanner::with_capacity(Cursor::new(data), 13);

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line().unwrap());
    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("456".into()), sc.next_line().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(Some("789".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
}