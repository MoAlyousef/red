use super::HighlightData;

use tree_sitter_toml as ts;

const RED: u32 = 0xe06c75;
const GREEN: u32 = 0x98c379;
const DARKYELLOW: u32 = 0xc69a66;

pub const STYLES: &[(&str, u32)] = &[
    ("DEFAULT", RED),
    ("property", RED),
    ("comment", 0x808080),
    ("string", GREEN),
    ("number", GREEN),
    ("operator", 0xd8d8d8),
    ("punctuation", 0xc69a66),
    ("constant.builtin", DARKYELLOW),
];

pub fn lang_data() -> HighlightData {
    let (names, styles) = super::resolve_styles(STYLES);
    HighlightData::new(names, styles, ts::language(), ts::HIGHLIGHT_QUERY, None)
}
