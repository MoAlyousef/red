use fltk::{
    app,
    enums::{Color, Font},
    prelude::DisplayExt,
    text::{StyleTableEntryExt, TextAttr, TextBuffer, TextEditor},
};
use std::path::Path;
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;
#[cfg(feature = "lsp")]
use crate::diagnostics;

mod colors;
mod md;
mod rust;
mod toml;

fn translate_style(idx: usize) -> char {
    char::from_u32(65 + idx as u32).unwrap()
}

fn resolve_styles(v: &[(&'static str, u32)]) -> (Vec<&'static str>, Vec<StyleTableEntryExt>) {
    let mut names = Vec::new();
    let mut styles = Vec::new();
    for elem in v {
        names.push(elem.0);
        styles.push(StyleTableEntryExt {
            color: Color::from_hex(elem.1),
            font: Font::Courier,
            size: app::font_size(),
            attr: TextAttr::None,
            bgcolor: Color::Background,
        });
    }
    // Duplicate styles with underline attribute to overlay diagnostics without losing color
    let base_len = styles.len();
    for i in 0..base_len {
        let mut s = styles[i];
        s.attr = TextAttr::Underline;
        styles.push(s);
    }
    (names, styles)
}

pub struct HighlightData {
    styles: Vec<StyleTableEntryExt>,
    config: HighlightConfiguration,
    exception_fn: Option<fn(usize, &str) -> char>,
}

impl HighlightData {
    pub fn new(
        styles: Vec<StyleTableEntryExt>,
        config: HighlightConfiguration,
        exception_fn: Option<fn(usize, &str) -> char>,
    ) -> Self {
        Self {
            styles,
            config,
            exception_fn,
        }
    }
}

fn get_highlight(p: &Path) -> Option<HighlightData> {
    if let Some(ext) = p.extension() {
        match ext.to_str().unwrap() {
            "rs" => Some(rust::lang_data()),
            "toml" => Some(toml::lang_data()),
            "md" => Some(md::lang_data()),
            _ => None,
        }
    } else {
        None
    }
}

pub fn highlight(p: &Path, ed: &mut TextEditor, buf: &mut TextBuffer) {
    if let Some(HighlightData {
        styles,
        config,
        exception_fn,
    }) = get_highlight(p)
    {
        let mut highlighter = Highlighter::new();
        let mut sbuf = TextBuffer::default();
        let base_styles = styles.len() / 2; // we doubled styles to include underline variants
        ed.set_highlight_data_ext(sbuf.clone(), styles);
        // Register style buffer for diagnostics overlay
        #[cfg(feature = "lsp")]
        diagnostics::register_style_buf(p, &sbuf, base_styles);
        apply(
            &mut highlighter,
            &config,
            &buf.text(),
            &mut sbuf,
            &exception_fn,
        );
        buf.add_modify_callback({
            let buf = buf.clone();
            move |_, _, _, _, _| {
                apply(
                    &mut highlighter,
                    &config,
                    &buf.text(),
                    &mut sbuf,
                    &exception_fn,
                );
            }
        });
    }
}

fn apply(
    highlighter: &mut Highlighter,
    config: &HighlightConfiguration,
    s: &str,
    sbuf: &mut TextBuffer,
    exception_fn: &Option<fn(usize, &str) -> char>,
) {
    let highlights = highlighter
        .highlight(config, s.as_bytes(), None, |_| None)
        .unwrap();

    let mut local_buf = "A".repeat(s.len());
    let mut curr = 0;
    for event in highlights {
        match event.unwrap() {
            HighlightEvent::HighlightStart(s) => {
                curr = s.0;
            }
            HighlightEvent::Source { start, end } => {
                let c = if let Some(f) = exception_fn {
                    f(curr, &s[start..end])
                } else {
                    translate_style(curr)
                };
                local_buf.replace_range(start..end, &c.to_string().repeat(end - start));
            }
            HighlightEvent::HighlightEnd => curr = 0,
        }
    }
    // Set base syntax highlight
    sbuf.set_text(&local_buf);
}
