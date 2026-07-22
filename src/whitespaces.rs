#[inline]
pub(crate) fn is_whitespace_1(c: u8) -> bool {
    // Matches Java's `Character.isWhitespace` for ASCII: HT/LF/VT/FF/CR, FS/GS/RS/US, and SPACE.
    (9..=13).contains(&c) || (28..=32).contains(&c)
}

#[inline]
pub(crate) fn is_whitespace_3(b1: u8, b2: u8, b3: u8) -> bool {
    // Matches Java's `Character.isWhitespace` for the 3-byte UTF-8 code points, decomposed into bytes.
    match b1 {
        225 => match b2 {
            // U+1680 OGHAM SPACE MARK
            154 => b3 == 128,
            _ => false,
        },
        226 => match b2 {
            // U+2000..=U+2006, U+2008..=U+200A (U+2007 FIGURE SPACE is non-breaking, excluded), U+2028, U+2029
            128 => (128..=134).contains(&b3) || (136..=138).contains(&b3) || b3 == 168 || b3 == 169,
            // U+205F MEDIUM MATHEMATICAL SPACE
            129 => b3 == 159,
            _ => false,
        },
        227 => match b2 {
            // U+3000 IDEOGRAPHIC SPACE
            128 => b3 == 128,
            _ => false,
        },
        _ => false,
    }
}
