use scanner_rust::{ScannerStr, ScannerU8Slice, ScannerU8SliceAscii};

#[test]
fn next_until_boundaries() {
    for (data, boundary, expected) in [
        ("aaba_tailaba_end", "aba", &["a", "_tail", "_end"][..]),
        ("ababababacrest", "ababac", &["abab", "rest"][..]),
        ("café界中文界é", "界", &["café", "中文", "é"][..]),
        ("abc", "", &["abc"][..]),
        ("abc", "long", &["abc"][..]),
    ] {
        let mut text = ScannerStr::new(data);
        let mut bytes = ScannerU8Slice::new(data.as_bytes());
        let mut ascii = ScannerU8SliceAscii::new(data.as_bytes());

        for expected in expected {
            assert_eq!(Some(*expected), text.next_until(boundary).unwrap());
            assert_eq!(Some(expected.as_bytes()), bytes.next_until(boundary).unwrap());
            assert_eq!(Some(expected.as_bytes()), ascii.next_until(boundary).unwrap());
        }

        assert_eq!(None, text.next_until(boundary).unwrap());
        assert_eq!(None, bytes.next_until(boundary).unwrap());
        assert_eq!(None, ascii.next_until(boundary).unwrap());
    }
}
