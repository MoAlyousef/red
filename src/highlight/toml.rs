use tree_sitter::Language;

use tree_sitter_toml as ts;

pub const STYLES: &[(&str, &str)] = &[
    ("property", "#fa0000"),
    ("comment", "#808080"),
    ("string", "#629755"),
    ("number", "#629755"),
    ("operator", "#d8d8d8"),
    ("punctuation", "#d8d8d8"),
    ("constant.builtin", "#6897bb"),
];

pub fn lang_data() -> (Language, &'static str) {
    (ts::language(), ts::HIGHLIGHT_QUERY)
}