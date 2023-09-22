use crate::dialogs;
use crate::state::STATE;
use fltk::{enums::*, prelude::*, *};
use std::{env, path::PathBuf};

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
        menu::MenuFlag::MenuDivider,
        menu_cb,
    );
    m.add(
        "&Edit/Find\t",
        Shortcut::Ctrl | 'f',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/Replace\t",
        Shortcut::Ctrl | 'h',
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

fn close_app() {
    STATE.with(|s| {
        if !s.modified {
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

fn find() {
    let mut dlg = dialogs::FindDialog::new();
    dlg.show();
}

fn replace() {
    let mut dlg = dialogs::ReplaceDialog::new();
    dlg.show();
}

pub fn win_cb(_: &mut window::Window) {
    if app::event() == Event::Close {
        close_app();
    }
}

pub fn editor_cb(_e: &mut text::TextEditor) {
    STATE.with(|s| s.was_modified(true));
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
                            s.was_modified(false);
                        }
                    }
                });
            }
            "&File/Open...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseFile);
                if let Ok(text) = std::fs::read_to_string(&c) {
                    STATE.with(move |s| {
                        s.buf.set_text(&text);
                        s.was_modified(false);
                        s.current_file = c.clone();
                    });
                }
            }
            "&File/Save\t" => {
                STATE.with(|s| {
                    if s.modified && s.current_file.exists() {
                        std::fs::write(&s.current_file, s.buf.text()).ok();
                        s.was_modified(false);
                    }
                });
            }
            "&File/Save as...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile);
                if c.exists() {
                    STATE.with(move |s| {
                        std::fs::write(&c, s.buf.text()).expect("Failed to write to file!");
                        s.was_modified(false);
                    });
                }
            }
            "&File/Quit\t" => close_app(),
            "&Edit/Cut\t" => ed.cut(),
            "&Edit/Copy\t" => ed.copy(),
            "&Edit/Paste\t" => ed.paste(),
            "&Edit/Find\t" => find(),
            "&Edit/Replace\t" => replace(),
            "&Help/About\t" => {
                dialog::message_default("A minimal text editor written using fltk-rs!")
            }
            _ => unreachable!(),
        }
    }
}

pub fn fbr_cb(f: &mut browser::FileBrowser) {
    if let Some(path) = f.text(f.value()) {
        let path = PathBuf::from(path);
        if path.exists() {
            if path.is_dir() && app::event_clicks() {
                f.load(path.clone()).expect("Couldn't load directory!");
                let cwd = env::current_dir().unwrap();
                env::set_current_dir(cwd.join(path)).unwrap();
            } else if let Ok(text) = std::fs::read_to_string(&path) {
                STATE.with(move |s| {
                    if s.modified {
                        let c = dialog::choice2_default(
                            "Are you sure you want to exit without saving?",
                            "Yes",
                            "No",
                            "",
                        );
                        if c == Some(0) {
                            s.buf.set_text(&text);
                            s.current_file = path.clone();
                            s.was_modified(false);
                        }
                    } else {
                        s.buf.set_text(&text);
                        s.current_file = path.clone();
                        s.was_modified(false);
                    }
                });
            }
        }
    }
}

pub fn tab_close_cb(g: &mut impl GroupExt) {
    if app::callback_reason() == CallbackReason::Closed {
        let mut parent = g.parent().unwrap();
        parent.remove(g);
        app::redraw();
    }
}

#[cfg(feature = "portable-pty")]
pub fn init_term(term: &crate::term::AnsiTerm, current_path: PathBuf) {
    if current_path.exists() {
        let mut writer1 = term.writer1.lock().unwrap();
        if current_path.is_dir() {
            writer1
                .write_all(
                    format!(
                        "cd {}\nclear\n",
                        current_path.canonicalize().unwrap().display()
                    )
                    .as_bytes(),
                )
                .unwrap();
        } else {
            writer1
                .write_all(
                    format!(
                        "cd {}\nclear\n",
                        current_path
                            .canonicalize()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .display()
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    }
}
