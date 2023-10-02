use tree_sitter::Language;

use tree_sitter_rust as ts;

pub const STYLES: &[(&str, u32)] = &[
    ("attribute", 0xfa0000),
    ("constructor", 0x00ffff),
    ("comment", 0x808080),
    ("constant", 0x00ff00),
    ("constant.builtin", 0x6897bb),
    ("function", 0x00ffff),
    ("function.method", 0x00ffff),
    ("keyword", 0x9876aa),
    ("operator", 0xd8d8d8),
    ("property", 0xcc7832),
    ("punctuation.bracket", 0xcc7832),
    ("punctuation.delimiter", 0xcc7832),
    ("string", 0x629755),
    ("type", 0xcc7832),
    ("type.builtin", 0x32cd32),
    ("variable", 0xeedd82),
    ("variable.builtin", 0xeedd82),
    ("variable.parameter", 0x32cd32),
    ("label", 0xffffff),
];

pub fn lang_data() -> (Language, &'static str) {
    (ts::language(), ts::HIGHLIGHT_QUERY)
}
