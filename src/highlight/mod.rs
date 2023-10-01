use fltk::{
    app,
    enums::{Color, Font},
    prelude::DisplayExt,
    text::{StyleTableEntry, TextBuffer, TextEditor},
};
use std::path::Path;

mod rust;
mod toml;

fn translate_style(idx: usize) -> char {
    char::from_u32(65 + idx as u32 + 1).unwrap()
}

fn resolve_styles(v: Vec<(&'static str, Color)>) -> (Vec<&'static str>, Vec<StyleTableEntry>) {
    let mut names = Vec::new();
    let mut styles = Vec::new();
    for elem in v {
        names.push(elem.0);
        styles.push(StyleTableEntry {
            color: elem.1,
            font: Font::Courier,
            size: app::font_size(),
        });
    }
    names.remove(0);
    (names, styles)
}

struct HighlightData {
    styles: Vec<(&'static str, Color)>,
    func: fn(s: &str, sbuf: &mut TextBuffer, names: &[&str]),
}

impl HighlightData {
    pub fn new(
        s: Vec<(&'static str, Color)>,
        f: fn(s: &str, sbuf: &mut TextBuffer, names: &[&str]),
    ) -> Self {
        Self { styles: s, func: f }
    }
}

fn get_highlight(p: &Path) -> Option<HighlightData> {
    if let Some(ext) = p.extension() {
        match ext.to_str().unwrap() {
            "rs" => Some(HighlightData::new(rust::styles(), rust::apply)),
            "toml" => Some(HighlightData::new(toml::styles(), toml::apply)),
            _ => None,
        }
    } else {
        None
    }
}

pub fn highlight(p: &Path, ed: &mut TextEditor, buf: &mut TextBuffer) {
    if let Some(HighlightData { styles, func }) = get_highlight(p) {
        let (names, styles) = resolve_styles(styles);
        let mut sbuf = TextBuffer::default();
        ed.set_highlight_data(sbuf.clone(), styles);
        func(&buf.text(), &mut sbuf, &names);
        buf.add_modify_callback({
            let buf = buf.clone();
            move |_, _, _, _, _| {
                func(&buf.text(), &mut sbuf, &names);
            }
        });
    }
}

#[macro_export]
macro_rules! apply_ {
    ($s:tt, $names: tt) => {{
        let mut highlighter = Highlighter::new();

        let lang = ts::language();

        let mut config = HighlightConfiguration::new(lang, ts::HIGHLIGHT_QUERY, "", "").unwrap();
        config.configure($names);
        let highlights = highlighter
            .highlight(&config, $s.as_bytes(), None, |_| None)
            .unwrap();

        let mut local_buf = "A".repeat($s.len());
        let mut c = 'A';
        for event in highlights {
            match event.unwrap() {
                HighlightEvent::HighlightStart(s) => {
                    c = super::translate_style(s.0);
                }
                HighlightEvent::Source { start, end } => {
                    local_buf.replace_range(start..end, &c.to_string().repeat(end - start));
                }
                HighlightEvent::HighlightEnd => (),
            }
        }
        local_buf
    }};
}
