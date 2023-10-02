use tree_sitter::Language;

use tree_sitter_markdown as ts;

const RED: u32 = 0xe06c75;
const GREEN: u32 = 0x98c379;
const DARKYELLOW: u32 = 0xc69a66;

pub const STYLES: &[(&str, u32)] = &[
    ("property", RED),
    ("comment", 0x808080),
    ("string", GREEN),
    ("number", GREEN),
    ("operator", 0xd8d8d8),
    ("punctuation", 0xc69a66),
    ("constant.builtin", DARKYELLOW),
];

pub fn lang_data() -> (Language, &'static str) {
    (ts::language(), ts::HIGHLIGHT_QUERY)
}
