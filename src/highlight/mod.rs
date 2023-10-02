use fltk::{
    app,
    enums::{Color, Font},
    prelude::DisplayExt,
    text::{StyleTableEntry, TextBuffer, TextEditor},
};
use std::path::Path;
use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;

mod rust;
mod toml;

fn translate_style(idx: usize) -> char {
    char::from_u32(65 + idx as u32 + 1).unwrap()
}

fn resolve_styles(v: &[(&'static str, &'static str,)]) -> (Vec<&'static str>, Vec<StyleTableEntry>) {
    let mut names = Vec::new();
    let mut styles = Vec::new();
    styles.push(StyleTableEntry {
        color: Color::Foreground,
        font: Font::Courier,
        size: app::font_size(),
    });
    for elem in v {
        names.push(elem.0);
        styles.push(StyleTableEntry {
            color: Color::from_hex_str(elem.1).unwrap(),
            font: Font::Courier,
            size: app::font_size(),
        });
    }
    (names, styles)
}

struct HighlightData {
    names: Vec<&'static str>,
    styles: Vec<StyleTableEntry>,
    lang: Language,
    hq: &'static str,
}

impl HighlightData {
    pub fn new(
        s: &[(&'static str, &'static str,)],
        lang_data: (Language, &'static str),
    ) -> Self {
        let (names, styles) = resolve_styles(s);
        Self { names, styles, lang: lang_data.0, hq: lang_data.1 }
    }
}

fn get_highlight(p: &Path) -> Option<HighlightData> {
    if let Some(ext) = p.extension() {
        match ext.to_str().unwrap() {
            "rs" => Some(HighlightData::new(rust::STYLES, rust::lang_data())),
            "toml" => Some(HighlightData::new(toml::STYLES, toml::lang_data())),
            _ => None,
        }
    } else {
        None
    }
}

pub fn highlight(p: &Path, ed: &mut TextEditor, buf: &mut TextBuffer) {
    if let Some(HighlightData { names, styles, lang, hq }) = get_highlight(p) {
        let mut highlighter = Highlighter::new();
        let mut config = HighlightConfiguration::new(lang, hq, "", "").unwrap();
        config.configure(&names);
        let mut sbuf = TextBuffer::default();
        ed.set_highlight_data(sbuf.clone(), styles);
        apply(&mut highlighter, &config, &buf.text(), &mut sbuf);
        buf.add_modify_callback({
            let buf = buf.clone();
            move |_, _, _, _, _| {
                apply(&mut highlighter, &config, &buf.text(), &mut sbuf);
            }
        });
    }
}

fn apply(highlighter: &mut Highlighter, config: &HighlightConfiguration, s: &str, sbuf: &mut TextBuffer) {
    let highlights = highlighter
        .highlight(config, s.as_bytes(), None, |_| None)
        .unwrap();

    let mut local_buf = "A".repeat(s.len());
    let mut c = 'A';
    for event in highlights {
        match event.unwrap() {
            HighlightEvent::HighlightStart(s) => {
                c = translate_style(s.0);
            }
            HighlightEvent::Source { start, end } => {
                local_buf.replace_range(start..end, &c.to_string().repeat(end - start));
            }
            HighlightEvent::HighlightEnd => (),
        }
    }
    sbuf.set_text(&local_buf);
}
