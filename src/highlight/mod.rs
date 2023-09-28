use fltk::{
    prelude::DisplayExt,
    text::{TextBuffer, TextEditor},
};
use std::path::Path;

pub mod json;

pub fn highlight(p: &Path, ed: &mut TextEditor, buf: &mut TextBuffer) {
    if let Some(ext) = p.extension() {
        let (styles, func) = match ext.to_str().unwrap() {
            "json" => (Some(json::styles()), Some(json::apply)),
            _ => (None, None),
        };
        if let Some(func) = func {
            let mut sbuf = TextBuffer::default();
            ed.set_highlight_data(sbuf.clone(), styles.unwrap());
            func(&buf.text(), &mut sbuf);
            buf.add_modify_callback({
                let buf = buf.clone();
                move |_, _, _, _, _| {
                    func(&buf.text(), &mut sbuf);
                }
            });
        }
    }
}
