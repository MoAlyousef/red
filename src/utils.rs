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
    let idx = m.add(
        "&View/File browser\t",
        Shortcut::Ctrl | 'f',
        menu::MenuFlag::Toggle,
        menu_cb,
    );
    m.at(idx).unwrap().set();
    #[cfg(feature = "portable-pty")]
    {
        let idx = m.add(
            "&View/Terminal\t",
            Shortcut::Ctrl | 'h',
            menu::MenuFlag::Toggle,
            menu_cb,
        );
        m.at(idx).unwrap().set();
    }
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
    STATE.with(|_| {
        app::quit();
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
            "&File/New...\t" => {
                STATE.with(|s| {
                    s.append(None);
                });
            }
            "&File/Open...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseFile);
                STATE.with(move |s| {
                    s.append(Some(c.clone()));
                });
            }
            "&File/Save\t" => {
                STATE.with(|s| {
                    if s.modified() && s.current_file().unwrap().exists() {
                        std::fs::write(s.current_file().unwrap(), s.buf().text()).ok();
                        s.was_modified(false);
                    }
                });
            }
            "&File/Save as...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile);
                if c.exists() {
                    STATE.with(move |s| {
                        std::fs::write(&c, s.buf().text()).expect("Failed to write to file!");
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
            "&View/File browser\t" => {
                let mut item = m.at(m.value()).unwrap();
                let fbr: browser::FileBrowser = app::widget_from_id("fbr").unwrap();
                let mut parent: group::Flex = unsafe { fbr.parent().unwrap().into_widget() };
                if !item.value() {
                    parent.fixed(&fbr, 1);
                    item.clear();
                } else {
                    parent.fixed(&fbr, 180);
                    item.set();
                }
                app::redraw();
            }
            "&View/Terminal\t" => {
                let mut item = m.at(m.value()).unwrap();
                let term: text::SimpleTerminal = app::widget_from_id("term").unwrap();
                let mut parent: group::Flex = unsafe { term.parent().unwrap().into_widget() };
                if !item.value() {
                    parent.fixed(&term, 1);
                    item.clear();
                } else {
                    parent.fixed(&term, 160);
                    item.set();
                }
                app::redraw();
            }
            "&Help/About\t" => {
                dialog::message_default("A minimal text editor written using fltk-rs!")
            }
            _ => (),
        }
    }
}

pub fn fbr_cb(f: &mut browser::FileBrowser) {
    if let Some(path) = f.text(f.value()) {
        let path = PathBuf::from(path);
        if path.exists() {
            if path.is_dir() {
                f.load(path.clone()).expect("Couldn't load directory!");
                let cwd = env::current_dir().unwrap();
                env::set_current_dir(cwd.join(path)).unwrap();
            } else {
                STATE.with(move |s| {
                    s.append(Some(path.clone()));
                });
            }
        }
    }
}

pub fn tab_close_cb(g: &mut impl GroupExt) {
    if app::callback_reason() == CallbackReason::Closed {
        let ed: text::TextEditor = unsafe { g.child(0).unwrap().into_widget() };
        let edid = ed.as_widget_ptr() as usize;
        let buf = ed.buffer().unwrap();
        let mut parent = g.parent().unwrap();
        parent.remove(g);
        unsafe {
            text::TextBuffer::delete(buf);
        }
        STATE.with(move |s| s.map.remove(&edid));
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
