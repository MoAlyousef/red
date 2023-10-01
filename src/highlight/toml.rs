use fltk::{enums::*, text::TextBuffer};
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;

use tree_sitter_toml as ts;

pub fn styles() -> Vec<(&'static str, Color)> {
    vec![
        ("", Color::Foreground),
        ("property", Color::Red),
        ("comment", Color::Foreground.darker()),
        ("string", Color::Green.darker()),
        ("number", Color::Green.darker()),
        ("operator", Color::White),
        ("punctuation", Color::White),
        ("constant.builtin", Color::Yellow),
    ]
}

pub fn apply(s: &str, sbuf: &mut TextBuffer, names: &[&str]) {
    sbuf.set_text(&crate::apply_!(s, names));
}
