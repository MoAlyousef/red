use super::HighlightData;
use tree_sitter_highlight::HighlightConfiguration;

use tree_sitter_md as ts;

const WHITE: u32 = 0xabb2bf;
const RED: u32 = 0xe06c75;
const GREEN: u32 = 0x98c379;
const DARKYELLOW: u32 = 0xc69a66;

pub const STYLES: &[(&str, u32)] = &[
    ("DEFAULT", WHITE),
    ("text.title", RED),
    ("text.reference", 0x808080),
    ("punctuation.special", RED),
    ("text.literal", GREEN),
    ("punctuation.delimiter", 0xc69a66),
    ("text.uri", DARKYELLOW),
];

pub fn lang_data() -> HighlightData {
    let (names, styles) = super::resolve_styles(STYLES);
    let mut config =
        HighlightConfiguration::new(ts::language(), ts::HIGHLIGHT_QUERY_BLOCK, "", "").unwrap();
    config.configure(&names);
    HighlightData::new(styles, config, None)
}
