use crate::STATE;
use fltk::{enums::*, prelude::*, *};
use std::path::PathBuf;

fn nfc_get_file(mode: dialog::NativeFileChooserType) -> PathBuf {
    let mut nfc = dialog::NativeFileChooser::new(mode);
    nfc.show();
    nfc.filename()
}

pub fn win_cb(w: &mut window::Window) {
    if app::event() == Event::Close {
        w.hide();
    }
}

pub fn editor_cb(_e: &mut text::TextEditor) {
    STATE.with(|s| s.saved = false);
}

pub fn menu_cb(m: &mut menu::MenuBar) {
    if let Ok(mpath) = m.item_pathname(None) {
        let ed: text::TextEditor = app::widget_from_id("ed").unwrap();
        match mpath.as_str() {
            "File/New" => {
                STATE.with(|s| {
                    if !s.buf.text().is_empty() {
                        let c = dialog::choice2_default(
                            "Are you sure you want to clear the buffer?",
                            "Yes",
                            "No",
                            "",
                        );
                        if c == Some(0) {
                            s.buf.set_text("");
                            s.saved = false;
                        }
                    }
                });
            }
            "File/Open..." => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseFile);
                let text = std::fs::read_to_string(&c).unwrap();
                STATE.with(move |s| {
                    s.buf.set_text(&text);
                    s.saved = false;
                    s.current_file = c.clone();
                });
            }
            "File/Save..." => {
                STATE.with(|s| {
                    if !s.saved && s.current_file.exists() {
                        std::fs::write(&s.current_file, &s.buf.text()).unwrap();
                    }
                });
            }
            "File/Save As..." => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile);
                STATE.with(move |s| {
                    std::fs::write(&c, &s.buf.text()).unwrap();
                    s.saved = true;
                    s.current_file = c.clone();
                });
            }
            "File/Quit" => {
                STATE.with(|s| {
                    if s.saved {
                        app::quit();
                    } else {
                        let c = dialog::choice2_default(
                            "Are you sure you want to exit without saving?",
                            "Yes",
                            "No",
                            "",
                        );
                        if c == Some(0) {
                            app::quit();
                        }
                    }
                });
            }
            "Edit/Cut" => ed.cut(),
            "Edit/Copy" => ed.copy(),
            "Edit/Paste" => ed.paste(),
            "Help/About" => dialog::message_default("A minimal text editor written using fltk-rs!"),
            _ => unreachable!(),
        }
    }
}
