extern crate scanner_rust;

use scanner_rust::ScannerAscii;

#[test]
fn read_chars() {
    let data = "Hello, world.";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some('H'), sc.next_char().unwrap());
    assert_eq!(Some('e'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('o'), sc.next_char().unwrap());
    assert_eq!(Some(','), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(Some('w'), sc.next_char().unwrap());
    assert_eq!(Some('o'), sc.next_char().unwrap());
    assert_eq!(Some('r'), sc.next_char().unwrap());
    assert_eq!(Some('l'), sc.next_char().unwrap());
    assert_eq!(Some('d'), sc.next_char().unwrap());
    assert_eq!(Some('.'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());

    let data = "123 abcd.,";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some('1'), sc.next_char().unwrap());
    assert_eq!(Some('2'), sc.next_char().unwrap());
    assert_eq!(Some('3'), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(Some('a'), sc.next_char().unwrap());
    assert_eq!(Some('b'), sc.next_char().unwrap());
    assert_eq!(Some('c'), sc.next_char().unwrap());
    assert_eq!(Some('d'), sc.next_char().unwrap());
    assert_eq!(Some('.'), sc.next_char().unwrap());
    assert_eq!(Some(','), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
}

#[test]
fn next_lines_crlf() {
    let data = "123\r\nab";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("ab".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\n\rab";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("ab".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\r\nab\r123\n456\n\r789\r";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("ab".into()), sc.next_line().unwrap());
    assert_eq!(Some("123".into()), sc.next_line().unwrap());
    assert_eq!(Some("456".into()), sc.next_line().unwrap());
    assert_eq!(Some("789".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
}

#[test]
fn next_lines_chars() {
    let data = "Hello, 123\nabcd\n\n";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some('H'), sc.next_char().unwrap());
    assert_eq!(Some("ello, 123".into()), sc.next_line().unwrap());
    assert_eq!(Some('a'), sc.next_char().unwrap());
    assert_eq!(Some('b'), sc.next_char().unwrap());
    assert_eq!(Some("cd".into()), sc.next_line().unwrap());
    assert_eq!(Some('\n'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
}

#[test]
fn next() {
    let data = "123 456  789 \n \t \r abcd\n";

    let mut sc = ScannerAscii::new(data.as_bytes());

    assert_eq!(Some("123".into()), sc.next_raw().unwrap());
    assert_eq!(Some("456".into()), sc.next_raw().unwrap());
    assert_eq!(Some("789".into()), sc.next_raw().unwrap());
    assert_eq!(Some("abcd".into()), sc.next_raw().unwrap());
    assert_eq!(Some("".into()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_raw().unwrap());
    assert_eq!(None, sc.next_raw().unwrap());
}
