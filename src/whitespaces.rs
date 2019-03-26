pub(crate) fn is_whitespace_1(c: u8) -> bool {
    (c >= 9 && c <= 13) || (c >= 28 && c <= 32)
}

pub(crate) fn is_whitespace_3(b1: u8, b2: u8, b3: u8) -> bool {
    match b1 {
        225 => {
            match b2 {
                154 => {
                    match b3 {
                        128 => {
                            true
                        }
                        _ => {
                            false
                        }
                    }
                }
                160 => {
                    match b3 {
                        142 => {
                            true
                        }
                        _ => {
                            false
                        }
                    }
                }
                _ => {
                    false
                }
            }
        }
        226 => {
            match b2 {
                128 => {
                    (b3 >= 128 && b3 <= 138) || b3 == 168 || b3 == 168
                }
                129 => {
                    match b3 {
                        159 => {
                            true
                        }
                        _ => {
                            false
                        }
                    }
                }
                _ => {
                    false
                }
            }
        }
        227 => {
            match b2 {
                128 => {
                    match b3 {
                        128 => {
                            true
                        }
                        _ => {
                            false
                        }
                    }
                }
                _ => {
                    false
                }
            }
        }
        _ => {
            false
        }
    }
}