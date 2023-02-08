use crate::state::STATE;
use fltk::{enums::*, prelude::*, *};
use std::path::PathBuf;

pub fn init_menu(m: &mut menu::SysMenuBar) {
    m.add(
        "&File/New...\t",
        Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Open...\t",
        Shortcut::Ctrl | 'o',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Save\t",
        Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Save as...\t",
        Shortcut::Ctrl | 'w',
        menu::MenuFlag::MenuDivider,
        menu_cb,
    );
    let idx = m.add(
        "&File/Quit\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.at(idx).unwrap().set_label_color(Color::Red);
    m.add(
        "&Edit/Cut\t",
        Shortcut::Ctrl | 'x',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/Copy\t",
        Shortcut::Ctrl | 'c',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/Paste\t",
        Shortcut::Ctrl | 'v',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Help/About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        menu_cb,
    );
}

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

pub fn fbr_cb(f: &mut browser::FileBrowser) {
    if let Some(path) = f.text(f.value()) {
        let path = PathBuf::from(path);
        if path.exists() && !path.is_dir() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                STATE.with(move |s| {
                    s.buf.set_text(&text);
                    s.saved = false;
                    s.current_file = path.clone();
                });
            }
        }
    }
}

pub fn editor_cb(_e: &mut text::TextEditor) {
    STATE.with(|s| s.saved = false);
}

pub fn menu_cb(m: &mut menu::SysMenuBar) {
    if let Ok(mpath) = m.item_pathname(None) {
        let ed: text::TextEditor = app::widget_from_id("ed").unwrap();
        match mpath.as_str() {
            "&File/New\t" => {
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
            "&File/Open...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseFile);
                if let Ok(text) = std::fs::read_to_string(&c) {
                    STATE.with(move |s| {
                        s.buf.set_text(&text);
                        s.saved = false;
                        s.current_file = c.clone();
                    });
                }
            }
            "&File/Save\t" => {
                STATE.with(|s| {
                    if !s.saved && s.current_file.exists() {
                        std::fs::write(&s.current_file, &s.buf.text()).ok();
                    }
                });
            }
            "&File/Save as...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile);
                STATE.with(move |s| {
                    std::fs::write(&c, &s.buf.text()).ok();
                    s.saved = true;
                    s.current_file = c.clone();
                });
            }
            "&File/Quit\t" => {
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
            "&Edit/Cut\t" => ed.cut(),
            "&Edit/Copy\t" => ed.copy(),
            "&Edit/Paste\t" => ed.paste(),
            "&Help/About\t" => {
                dialog::message_default("A minimal text editor written using fltk-rs!")
            }
            _ => unreachable!(),
        }
    }
}
