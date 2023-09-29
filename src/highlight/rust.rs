use fltk::{
    enums::*,
    text::{StyleTableEntry, TextBuffer},
};
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;

use tree_sitter_rust as ts;

const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "function",
    "keyword",
    "operator",
    "property",
    "punctuation",
    "string",
    "type",
    "variable",
];

pub fn styles() -> Vec<StyleTableEntry> {
    vec![
        StyleTableEntry {
            // attr
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
            // const
            color: Color::DarkYellow,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // fn
            color: Color::Blue.lighter(),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // kwd
            color: Color::Cyan.lighter(),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // op
            color: Color::Foreground,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // prop
            color: Color::DarkYellow,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // punct
            color: Color::Foreground,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // str
            color: Color::Green.lighter(),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // ty
            color: Color::DarkYellow,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            // var
            color: Color::Yellow,
            font: Font::Courier,
            size: 14,
        },
    ]
}

pub fn apply(s: &str, sbuf: &mut TextBuffer) {
    sbuf.set_text(&crate::apply_!(s));
}
