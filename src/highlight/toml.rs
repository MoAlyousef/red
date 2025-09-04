use super::colors::*;
use super::HighlightData;
use tree_sitter_highlight::HighlightConfiguration;

use tree_sitter_toml_ng as ts;

pub const STYLES: &[(&str, u32)] = &[
    ("DEFAULT", RED),
    ("property", RED),
    ("comment", GREY),
    ("string", GREEN),
    ("number", GREEN),
    ("operator", LIGHTGREY),
    ("punctuation", DARKYELLOW),
    ("constant.builtin", DARKYELLOW),
];

pub fn lang_data() -> HighlightData {
    let (names, styles) = super::resolve_styles(STYLES);
    let mut config =
        HighlightConfiguration::new(ts::LANGUAGE.into(), "toml", ts::HIGHLIGHTS_QUERY, "", "")
            .unwrap();
    config.configure(&names);
    HighlightData::new(styles, config, None)
}
