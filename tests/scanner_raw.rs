extern crate scanner_rust;

use scanner_rust::Scanner;

#[test]
fn next_line() {
    let data = "Hello, world.";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("Hello, world.".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());

    let data = "123 中文好難。寝る";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123 中文好難。寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());

    let data = "Hello, world.\n";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("Hello, world.".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());

    let data = "123 中文好難。寝る\n";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123 中文好難。寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());

    let data = "123 中文好難。寝る\n123 中文好難。寝る\n\n123 中文好難。\r\r寝る\r \n123 中文好難。寝る\rHello, \nworld.";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123 中文好難。寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("123 中文好難。寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("123 中文好難。".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some(" ".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("123 中文好難。寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("Hello, ".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("world.".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
}

#[test]
fn next_line_crlf() {
    let data = "123\r\n中文";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());

    let data = "123\n\r中文";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());

    let data = "123\r\n中文\r123\n456\n\r789\r";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("中文".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("123".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("456".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some("789".into()), sc.next_line_raw().unwrap());
    assert_eq!(None, sc.next_line_raw().unwrap());
}

#[test]
fn next_line_chars() {
    let data = "Hello, 123\n中文好難。寝る\n\n";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some('H'), sc.next_char().unwrap());
    assert_eq!(Some("ello, 123".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some('中'), sc.next_char().unwrap());
    assert_eq!(Some('文'), sc.next_char().unwrap());
    assert_eq!(Some("好難。寝る".into()), sc.next_line_raw().unwrap());
    assert_eq!(Some('\n'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
}

#[test]
fn next() {
    let data = "123 456  789 \n \t \r 中文好難\n";

    let mut sc = Scanner::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_raw().unwrap());
    assert_eq!(Some("456".into()), sc.next_raw().unwrap());
    assert_eq!(Some("789".into()), sc.next_raw().unwrap());
    assert_eq!(Some("中文好難".into()), sc.next_raw().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_raw().unwrap());
    assert_eq!(None, sc.next_raw().unwrap());
}
