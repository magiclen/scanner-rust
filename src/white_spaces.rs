const WHITE_SPACES_1: [u8; 9] = [
    '\u{0009}' as u8,
    '\u{000A}' as u8,
    '\u{000B}' as u8,
    '\u{000C}' as u8,
    '\u{000D}' as u8,
    '\u{001C}' as u8,
    '\u{001D}' as u8,
    '\u{001E}' as u8,
    '\u{001F}' as u8,
];

const WHITE_SPACES_2: [(u8, u8); 1] = [
    (194, 160)        // \u00A0
];

const WHITE_SPACES_3: [(u8, u8, u8); 2] = [
    (226, 128, 135), // \u2007
    (226, 128, 175)  // \u2007
];