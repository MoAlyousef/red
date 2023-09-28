use fltk::{
    prelude::DisplayExt,
    text::{StyleTableEntry, TextBuffer, TextEditor},
};
use std::path::Path;

pub mod json;

struct HighlightData {
    styles: Vec<StyleTableEntry>,
    func: fn(s: &str, sbuf: &mut TextBuffer),
}

impl HighlightData {
    pub fn new(s: Vec<StyleTableEntry>, f: fn(s: &str, sbuf: &mut TextBuffer)) -> Self {
        Self { styles: s, func: f }
    }
}

fn get_highlight(p: &Path) -> Option<HighlightData> {
    if let Some(ext) = p.extension() {
        match ext.to_str().unwrap() {
            "json" => Some(HighlightData::new(json::styles(), json::apply)),
            _ => None,
        }
    } else {
        None
    }
}

pub fn highlight(p: &Path, ed: &mut TextEditor, buf: &mut TextBuffer) {
    if let Some(HighlightData { styles, func }) = get_highlight(p) {
        let mut sbuf = TextBuffer::default();
        ed.set_highlight_data(sbuf.clone(), styles);
        func(&buf.text(), &mut sbuf);
        buf.add_modify_callback({
            let buf = buf.clone();
            move |_, _, _, _, _| {
                func(&buf.text(), &mut sbuf);
            }
        });
    }
}
