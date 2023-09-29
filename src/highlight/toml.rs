use fltk::{
    enums::*,
    text::{StyleTableEntry, TextBuffer},
};
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;

use tree_sitter_toml as ts;

const HIGHLIGHT_NAMES: &[&str] = &[
    "property",
    "comment",
    "string",
    "number",
    "operator",
    "punctuation",
];

pub fn styles() -> Vec<StyleTableEntry> {
    vec![
        StyleTableEntry {
            // prop
            color: Color::Red,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // comment
            color: Color::Foreground.darker(),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // str
            color: Color::Green.darker(),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // num
            color: Color::Green.darker(),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // punct
            color: Color::White,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // =
            color: Color::White,
            font: Font::Courier,
            size: 14,
        },
    ]
}

pub fn apply(s: &str, sbuf: &mut TextBuffer) {
    sbuf.set_text(&crate::apply_!(s));
}
