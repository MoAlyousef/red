use tree_sitter::Language;

use tree_sitter_rust as ts;

pub const STYLES: &[(&str, &str)] = &[
    ("attribute", "#fa0000"),
    ("constructor", "#00ffff"),
    ("comment", "#808080"),
    ("constant", "#00ff00"),
    ("constant.builtin", "#6897bb"),
    ("function", "#00ffff"),
    ("function.method", "#00ffff"),
    ("keyword", "#9876aa"),
    ("operator", "#d8d8d8"),
    ("property", "#cc7832"),
    ("punctuation.bracket", "#cc7832"),
    ("punctuation.delimiter", "#cc7832"),
    ("string", "#629755"),
    ("type", "#cc7832"),
    ("type.builtin", "#32cd32"),
    ("variable", "#eedd82"),
    ("variable.builtin", "#eedd82"),
    ("variable.parameter", "#32cd32"),
    ("label", "#ffffff"),
];

pub fn lang_data() -> (Language, &'static str) {
    (ts::language(), ts::HIGHLIGHT_QUERY)
}