extern crate scanner_rust;

use scanner_rust::ScannerU8Slice;

#[test]
fn read_chars() {
    let data = "Hello, world.";

    let mut sc = ScannerU8Slice::new(data);

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

    let data = "123 中文好難。寝る";

    let mut sc = ScannerU8Slice::new(data);

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
    let data = "Hello, world.";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("Hello, world.".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123 中文好難。寝る";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123 中文好難。寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "Hello, world.\n";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("Hello, world.".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123 中文好難。寝る\n";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123 中文好難。寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123 中文好難。寝る\n123 中文好難。寝る\n\n123 中文好難。\r\r寝る\r \n123 中文好難。寝る\rHello, \nworld.";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123 中文好難。寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("123 中文好難。寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("123 中文好難。".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some(" ".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("123 中文好難。寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("Hello, ".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("world.".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
}

#[test]
fn next_lines_crlf() {
    let data = "123\r\n中文";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("中文".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\n\r中文";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("中文".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());

    let data = "123\r\n中文\r123\n456\n\r789\r";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("中文".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("123".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("456".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some("789".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next_line().unwrap());
}

#[test]
fn next_lines_chars() {
    let data = "Hello, 123\n中文好難。寝る\n\n";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some('H'), sc.next_char().unwrap());
    assert_eq!(Some("ello, 123".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some('中'), sc.next_char().unwrap());
    assert_eq!(Some('文'), sc.next_char().unwrap());
    assert_eq!(Some("好難。寝る".as_bytes()), sc.next_line().unwrap());
    assert_eq!(Some('\n'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
}

#[test]
fn skip_whitespaces() {
    let data = "123 456";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('1'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('2'), sc.next_char().unwrap());
    assert_eq!(Some('3'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('4'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('5'), sc.next_char().unwrap());
    assert_eq!(Some('6'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(false, sc.skip_whitespaces().unwrap());

    let data = "123     \t\n\r\n    456";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('1'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('2'), sc.next_char().unwrap());
    assert_eq!(Some('3'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('4'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('5'), sc.next_char().unwrap());
    assert_eq!(Some('6'), sc.next_char().unwrap());
    assert_eq!(None, sc.next_char().unwrap());
    assert_eq!(false, sc.skip_whitespaces().unwrap());

    let data = "123   中文字  \t\n\r\n    456\n";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('1'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('2'), sc.next_char().unwrap());
    assert_eq!(Some('3'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('中'), sc.next_char().unwrap());
    assert_eq!(Some('文'), sc.next_char().unwrap());
    assert_eq!(Some('字'), sc.next_char().unwrap());
    assert_eq!(Some(' '), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('4'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(Some('5'), sc.next_char().unwrap());
    assert_eq!(Some('6'), sc.next_char().unwrap());
    assert_eq!(true, sc.skip_whitespaces().unwrap());
    assert_eq!(false, sc.skip_whitespaces().unwrap());
}

#[test]
fn next() {
    let data = "123 456  789 \n \t \r 中文好難\n";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some("123".as_bytes()), sc.next().unwrap());
    assert_eq!(Some("456".as_bytes()), sc.next().unwrap());
    assert_eq!(Some("789".as_bytes()), sc.next().unwrap());
    assert_eq!(Some("中文好難".as_bytes()), sc.next().unwrap());
    assert_eq!(Some("".as_bytes()), sc.next_line().unwrap());
    assert_eq!(None, sc.next().unwrap());
    assert_eq!(None, sc.next().unwrap());
}

#[test]
fn next_u8() {
    let data = "64 128";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(64), sc.next_u8().unwrap());
    assert_eq!(Some(128), sc.next_u8().unwrap());
}

#[test]
fn next_u16() {
    let data = "16384 32768";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(16384), sc.next_u16().unwrap());
    assert_eq!(Some(32768), sc.next_u16().unwrap());
}

#[test]
fn next_u32() {
    let data = "1073741824 2147483648";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(1073741824), sc.next_u32().unwrap());
    assert_eq!(Some(2147483648), sc.next_u32().unwrap());
}

#[test]
fn next_u64() {
    let data = "4611686018427387904 9223372036854775808";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(4611686018427387904), sc.next_u64().unwrap());
    assert_eq!(Some(9223372036854775808), sc.next_u64().unwrap());
}

#[test]
fn next_u128() {
    let data = "85070591730234615865843651857942052864 170141183460469231731687303715884105728";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(85070591730234615865843651857942052864), sc.next_u128().unwrap());
    assert_eq!(Some(170141183460469231731687303715884105728), sc.next_u128().unwrap());
}

#[test]
fn next_usize() {
    let data = "1073741824 2147483648";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(1073741824), sc.next_usize().unwrap());
    assert_eq!(Some(2147483648), sc.next_usize().unwrap());
}

#[test]
fn next_i8() {
    let data = "64 -128";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(64), sc.next_i8().unwrap());
    assert_eq!(Some(-128), sc.next_i8().unwrap());
}

#[test]
fn next_i16() {
    let data = "16384 -32768";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(16384), sc.next_i16().unwrap());
    assert_eq!(Some(-32768), sc.next_i16().unwrap());
}

#[test]
fn next_i32() {
    let data = "1073741824 -2147483648";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(1073741824), sc.next_i32().unwrap());
    assert_eq!(Some(-2147483648), sc.next_i32().unwrap());
}

#[test]
fn next_i64() {
    let data = "4611686018427387904 -9223372036854775808";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(4611686018427387904), sc.next_i64().unwrap());
    assert_eq!(Some(-9223372036854775808), sc.next_i64().unwrap());
}

#[test]
fn next_i128() {
    let data = "85070591730234615865843651857942052864 -170141183460469231731687303715884105728";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(85070591730234615865843651857942052864), sc.next_i128().unwrap());
    assert_eq!(Some(-170141183460469231731687303715884105728), sc.next_i128().unwrap());
}

#[test]
fn next_isize() {
    let data = "1073741824 -2147483648";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(1073741824), sc.next_isize().unwrap());
    assert_eq!(Some(-2147483648), sc.next_isize().unwrap());
}

#[test]
fn next_f32() {
    let data = "1 -5.124";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(1.0), sc.next_f32().unwrap());
    assert_eq!(Some(-5.124), sc.next_f32().unwrap());
}

#[test]
fn next_f64() {
    let data = "2 -123456.987654";

    let mut sc = ScannerU8Slice::new(data);

    assert_eq!(Some(2.0), sc.next_f64().unwrap());
    assert_eq!(Some(-123456.987654), sc.next_f64().unwrap());
}
