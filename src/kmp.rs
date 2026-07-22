/// Build the KMP "longest proper prefix which is also a suffix" table for `pattern`.
/// It lets a streaming search fall back on a mismatch without re-reading consumed bytes.
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
