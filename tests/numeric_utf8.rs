use std::num::IntErrorKind;

use scanner_rust::{Scanner, ScannerAscii, ScannerError, ScannerU8Slice, ScannerU8SliceAscii};

#[test]
fn invalid_utf8() {
    for data in [&[0xFF][..], &[b'1', 0xE2, 0x82][..]] {
        for result in [
            Scanner::new(data).next_i64(),
            ScannerAscii::new(data).next_i64(),
            ScannerU8Slice::new(data).next_i64(),
            ScannerU8SliceAscii::new(data).next_i64(),
            Scanner::new(data).next_i64_until("|"),
            ScannerAscii::new(data).next_i64_until("|"),
            ScannerU8Slice::new(data).next_i64_until("|"),
            ScannerU8SliceAscii::new(data).next_i64_until("|"),
        ] {
            assert!(matches!(result, Err(ScannerError::ParseIntError(_))));
        }

        for result in [
            Scanner::new(data).next_f64(),
            ScannerAscii::new(data).next_f64(),
            ScannerU8Slice::new(data).next_f64(),
            ScannerU8SliceAscii::new(data).next_f64(),
            Scanner::new(data).next_f64_until("|"),
            ScannerAscii::new(data).next_f64_until("|"),
            ScannerU8Slice::new(data).next_f64_until("|"),
            ScannerU8SliceAscii::new(data).next_f64_until("|"),
        ] {
            assert!(matches!(result, Err(ScannerError::ParseFloatError(_))));
        }
    }
}

#[test]
fn unicode_overflow_priority() {
    let data = "9223372036854775808中".as_bytes();

    for result in [
        Scanner::new(data).next_i64(),
        ScannerAscii::new(data).next_i64(),
        ScannerU8Slice::new(data).next_i64(),
        ScannerU8SliceAscii::new(data).next_i64(),
        Scanner::new(data).next_i64_until("|"),
        ScannerAscii::new(data).next_i64_until("|"),
        ScannerU8Slice::new(data).next_i64_until("|"),
        ScannerU8SliceAscii::new(data).next_i64_until("|"),
    ] {
        match result.unwrap_err() {
            ScannerError::ParseIntError(error) => {
                assert_eq!(&IntErrorKind::PosOverflow, error.kind());
            },
            error => panic!("unexpected error: {error}"),
        }
    }
}
