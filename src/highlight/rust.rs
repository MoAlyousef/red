use fltk::{enums::*, text::TextBuffer};
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;

use tree_sitter_rust as ts;

pub fn styles() -> Vec<(&'static str, Color)> {
    vec![
        ("", Color::Foreground),
        ("attribute", Color::Red),
        ("comment", Color::Foreground.darker()),
        ("constant", Color::Green),
        ("function", Color::Blue.lighter()),
        ("keyword", Color::Magenta.lighter()),
        ("operator", Color::Foreground),
        ("property", Color::DarkYellow),
        ("punctuation", Color::Foreground),
        ("string", Color::Green.lighter()),
        ("type", Color::DarkYellow),
        ("variable", Color::Yellow),
        ("label", Color::White),
    ]
}

pub fn apply(s: &str, sbuf: &mut TextBuffer, names: &[&str]) {
    sbuf.set_text(&crate::apply_!(s, names));
}
