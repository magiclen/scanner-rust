/// Build the KMP "longest proper prefix which is also a suffix" table for `pattern`.
/// It lets a search fall back on a mismatch without re-reading earlier bytes.
pub(crate) fn compute_lps(pattern: &[u8]) -> Vec<usize> {
    let mut lps = vec![0; pattern.len()];
    let mut len = 0;
    let mut i = 1;

    while i < pattern.len() {
        if pattern[i] == pattern[len] {
            len += 1;
            lps[i] = len;
            i += 1;
        } else if len > 0 {
            len = lps[len - 1];
        } else {
            lps[i] = 0;
            i += 1;
        }
    }

    lps
}

/// Find the position of the first occurrence of `boundary` in `data`.
/// An empty boundary always returns `None`.
pub(crate) fn find_boundary(data: &[u8], boundary: &[u8]) -> Option<usize> {
    let first = *boundary.first()?;
    let offset = data.iter().position(|&byte| byte == first)?;
    let data = &data[offset..];

    if data.len() < boundary.len() {
        return None;
    }

    // Single-byte boundaries and a full match at the first candidate need no prefix table.
    if data.starts_with(boundary) {
        return Some(offset);
    }

    let lps = compute_lps(boundary);
    let mut b = 0;

    for (i, &byte) in data.iter().enumerate() {
        while b > 0 && byte != boundary[b] {
            b = lps[b - 1];
        }

        if byte == boundary[b] {
            b += 1;
        }

        if b == boundary.len() {
            return Some(offset + i + 1 - boundary.len());
        }
    }

    None
}
