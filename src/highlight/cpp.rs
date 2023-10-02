use super::HighlightData;

use tree_sitter_cpp as ts;

const GREEN: u32 = 0x98c379;
const RED: u32 = 0xe06c75;
const YELLOW: u32 = 0xe5c07b;
const DARKYELLOW: u32 = 0xc69a66;
const BLUE: u32 = 0x61afef;
const PURPLE: u32 = 0xc678dd;

pub const STYLES: &[(&str, u32)] = &[
    ("constant", DARKYELLOW),
    ("function", BLUE),
    ("keyword", PURPLE),
    ("string", GREEN),
    ("type", YELLOW),
    ("variable.builtin", RED),
];

pub fn lang_data() -> HighlightData {
    let (names, styles) = super::resolve_styles(STYLES);
    HighlightData::new(names, styles, ts::language(), ts::HIGHLIGHT_QUERY, None)
}
