use super::HighlightData;

use tree_sitter_md as ts;

const RED: u32 = 0xe06c75;
const GREEN: u32 = 0x98c379;
const DARKYELLOW: u32 = 0xc69a66;

pub const STYLES: &[(&str, u32)] = &[
    ("DEFAULT", 0xfafafa),
    ("text.title", RED),
    ("text.reference", 0x808080),
    ("punctuation.special", GREEN),
    ("text.literal", 0xffffff),
    ("punctuation.delimiter", 0xc69a66),
    ("text.uri", DARKYELLOW),
];

pub fn lang_data() -> HighlightData {
    let (names, styles) = super::resolve_styles(STYLES);
    HighlightData::new(
        names,
        styles,
        ts::language(),
        ts::HIGHLIGHT_QUERY_BLOCK,
        None,
    )
}
