use tree_sitter::Language;

use tree_sitter_toml as ts;

pub const STYLES: &[(&str, u32)] = &[
    ("property", 0xfa0000),
    ("comment", 0x808080),
    ("string", 0x629755),
    ("number", 0x629755),
    ("operator", 0xd8d8d8),
    ("punctuation", 0xd8d8d8),
    ("constant.builtin", 0x6897bb),
];

pub fn lang_data() -> (Language, &'static str) {
    (ts::language(), ts::HIGHLIGHT_QUERY)
}
