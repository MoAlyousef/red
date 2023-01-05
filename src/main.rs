use fltk::{enums::*, prelude::*, *};
use std::path::PathBuf;

struct State {
    saved: bool,
    buf: text::TextBuffer,
    current_file: PathBuf,
}

fn menu_cb(m: &mut menu::MenuBar) {
    if let Ok(mpath) = m.item_pathname(None) {
        let state = app::GlobalState::<State>::get();
        let ed: text::TextEditor = app::widget_from_id("ed").unwrap();
        match mpath.as_str() {
            "File/New" => {
                state.with(|s| {
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
                let mut nfc =
                    dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
                nfc.show();
                let c = nfc.filename();
                let text = std::fs::read_to_string(&c).unwrap();
                state.with(move |s| {
                    s.buf.set_text(&text);
                    s.saved = false;
                    s.current_file = c.clone();
                });
            }
            "File/Save..." => {
                state.with(|s| {
                    if !s.saved && s.current_file.exists() {
                        std::fs::write(&s.current_file, &s.buf.text()).unwrap();
                    }
                });
            }
            "File/Save As..." => {
                let mut nfc =
                    dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseSaveFile);
                nfc.show();
                let c = nfc.filename();
                state.with(move |s| {
                    std::fs::write(&c, &s.buf.text()).unwrap();
                    s.saved = true;
                    s.current_file = c.clone();
                });
            }
            "File/Quit" => {
                state.with(|s| {
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

fn editor_cb(_e: &mut text::TextEditor) {
    let state = app::GlobalState::<State>::get();
    state.with(|s| s.saved = false);
}

fn win_cb(w: &mut window::Window) {
    if app::event() == Event::Close {
        w.hide();
    }
}

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Oxy);
    let buf = text::TextBuffer::default();
    let state = State {
        saved: false,
        buf: buf.clone(),
        current_file: PathBuf::new(),
    };
    app::GlobalState::new(state);
    let mut w = window::Window::default()
        .with_size(800, 600)
        .with_label("Ted");
    let mut m = menu::MenuBar::new(0, 0, 800, 30, None);
    m.add_choice("File/New|File/Open...|File/Save...|File/Save As...|File/Quit|Edit/Cut|Edit/Copy|Edit/Paste|Help/About");
    m.set_callback(menu_cb);
    let mut ed = text::TextEditor::new(0, 30, 800, 600 - 30, None);
    ed.set_linenumber_width(50);
    ed.set_id("ed");
    ed.set_buffer(buf);
    ed.set_trigger(CallbackTrigger::Changed);
    ed.set_callback(editor_cb);
    w.end();
    w.resizable(&ed);
    w.show();
    w.set_callback(win_cb);
    a.run().unwrap();
}
