#[inline]
pub(crate) fn is_whitespace_1(c: u8) -> bool {
    (9..=13).contains(&c) || (28..=32).contains(&c)
}

#[inline]
pub(crate) fn is_whitespace_3(b1: u8, b2: u8, b3: u8) -> bool {
    match b1 {
        225 => {
            match b2 {
                154 => {
                    matches!(b3, 128)
                }
                160 => {
                    matches!(b3, 142)
                }
                _ => false,
            }
        }
        226 => {
            match b2 {
                128 => (128..=138).contains(&b3) || b3 == 168 || b3 == 169,
                129 => {
                    matches!(b3, 159)
                }
                _ => false,
            }
        }
        227 => {
            match b2 {
                128 => {
                    matches!(b3, 128)
                }
                _ => false,
            }
        }
        _ => false,
    }
}
