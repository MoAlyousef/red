use tree_sitter::Language;

use tree_sitter_rust as ts;

const GREEN: u32 = 0x98c379;
const RED: u32 = 0xe06c75;
const YELLOW: u32 = 0xe5c07b;
const DARKYELLOW: u32 = 0xc69a66;
const WHITE: u32 = 0xabb2bf;
const BLUE: u32 = 0x61afef;
const PURPLE: u32 = 0xc678dd;
const GREY: u32 = 0x808080;

pub const STYLES: &[(&str, u32)] = &[
    ("attribute", RED),
    ("constructor", DARKYELLOW),
    ("comment", GREY),
    ("constant", DARKYELLOW),
    ("constant.builtin", DARKYELLOW),
    ("function", BLUE),
    ("function.method", BLUE),
    ("keyword", PURPLE),
    ("operator", WHITE),
    ("property", RED),
    ("punctuation.bracket", DARKYELLOW),
    ("punctuation.delimiter", WHITE),
    ("string", GREEN),
    ("type", YELLOW),
    ("type.builtin", YELLOW),
    ("variable", RED),
    ("variable.builtin", RED),
    ("variable.parameter", WHITE),
    ("label", WHITE),
];

pub fn lang_data() -> (Language, &'static str) {
    (ts::language(), ts::HIGHLIGHT_QUERY)
}
