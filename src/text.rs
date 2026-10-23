use std::str::{from_utf8, from_utf8_unchecked};

/// Get a `&str` view of a token so that it can be parsed into a number.
#[inline]
pub(crate) fn numeric_text(bytes: &[u8]) -> &str {
    if bytes.is_ascii() {
        // SAFETY: ASCII bytes are valid UTF-8.
        unsafe { from_utf8_unchecked(bytes) }
    } else {
        // Every supported number type rejects the replacement character.
        from_utf8(bytes).unwrap_or("\u{FFFD}")
    }
}

/// Convert bytes into a `String`, and only copy them when they are not valid UTF-8.
#[inline]
pub(crate) fn into_string_lossy(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
    }
}
