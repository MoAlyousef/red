use crate::state::STATE;
use fltk::{enums::*, prelude::*, *};
use std::{env, path::PathBuf};

pub fn init_menu(m: &mut (impl MenuExt + 'static)) {
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
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Save All\t",
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
    m.at(idx).unwrap().set_label_color(Color::Selection);
    m.add(
        "&Edit/Undo\t",
        Shortcut::Ctrl | 'z',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/Redo\t",
        Shortcut::Ctrl | 'y',
        menu::MenuFlag::MenuDivider,
        menu_cb,
    );
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
    let idx = m.add(
        "&View/Terminal\t",
        Shortcut::Ctrl | 'h',
        menu::MenuFlag::Toggle,
        menu_cb,
    );
    m.at(idx).unwrap().set();
    m.add(
        "&Help/About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        menu_cb,
    );
}

pub fn init_editor(ed: &mut text::TextEditor) {
    ed.set_linenumber_width(40);
    ed.set_linenumber_size(12);
    // ed.set_linenumber_fgcolor(Color::Yellow);
    // ed.set_linenumber_bgcolor(Color::Background);
    ed.set_text_font(Font::Courier);
    ed.set_trigger(CallbackTrigger::Changed);
    ed.set_callback(editor_cb);
}

fn nfc_get_file(mode: dialog::NativeFileChooserType) -> PathBuf {
    let mut nfc = dialog::NativeFileChooser::new(mode);
    nfc.show();
    nfc.filename()
}

fn close_app() {
    app::quit();
}

fn find() {
    let mut dlg: window::Window = app::widget_from_id("find").unwrap();
    let main_win = app::first_window().unwrap();
    dlg.resize(main_win.x() + main_win.w() - 300, dlg.y() + 30, 300, 50);
    dlg.show();
}

fn replace() {
    let mut dlg: window::Window = app::widget_from_id("replace").unwrap();
    let main_win = app::first_window().unwrap();
    dlg.resize(main_win.x() + main_win.w() - 300, dlg.y() + 30, 300, 80);
    dlg.show();
}

pub fn win_cb(_: &mut window::Window) {
    if app::event() == Event::Close {
        close_app();
    }
}

pub fn editor_cb(_e: &mut text::TextEditor) {
    app::add_timeout3(0.01, |_| STATE.with(|s| s.was_modified(true)));
}

pub fn menu_cb(m: &mut impl MenuExt) {
    if let Ok(mpath) = m.item_pathname(None) {
        match mpath.as_str() {
            "&File/New...\t" => {
                STATE.with(|s| {
                    s.append(None);
                });
            }
            "&File/Open...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseFile);
                if c.exists() {
                    STATE.with(move |s| {
                        s.append(Some(c.canonicalize().unwrap()));
                    });
                }
            }
            "&File/Save\t" => {
                STATE.with(|s| {
                    if let Some(id) = s.current_id() {
                        let e = s.map.get(&id).unwrap();
                        let modified = e.modified;
                        if let Some(current_file) = e.current_file.as_ref() {
                            if modified && current_file.exists() {
                                std::fs::write(current_file, e.buf.text()).ok();
                                s.was_modified(false);
                            }
                        }
                    }
                });
            }
            "&File/Save as...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile);
                if c.exists() {
                    STATE.with(move |s| {
                        if let Some(buf) = s.buf().as_ref() {
                            std::fs::write(&c, buf.text()).expect("Failed to write to file!");
                            s.was_modified(false);
                        }
                    });
                }
            }
            "&File/Save All\t" => {
                STATE.with(|s| {
                    for v in s.map.values_mut() {
                        if v.modified && v.current_file.as_ref().unwrap().exists() {
                            std::fs::write(v.current_file.as_ref().unwrap(), v.buf.text()).ok();
                            v.modified = true;
                        }
                    }
                });
            }
            "&File/Quit\t" => close_app(),
            "&Edit/Undo\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.undo()
                }
            }),
            "&Edit/Redo\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.redo()
                }
            }),
            "&Edit/Cut\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.cut()
                }
            }),
            "&Edit/Copy\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.copy()
                }
            }),
            "&Edit/Paste\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.paste()
                }
            }),
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
                dialog::message_title("About");
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
            if path.is_dir() {
                f.load(path.clone()).expect("Couldn't load directory!");
                let cwd = env::current_dir().unwrap();
                env::set_current_dir(cwd.join(path)).unwrap();
                let mut info: frame::Frame = app::widget_from_id("info").unwrap();
                info.set_label(&format!(
                    "Directory: {}",
                    env::current_dir().unwrap().display()
                ));
                f.set_damage(true);
            } else {
                let mut is_image = false;
                if let Some(ext) = path.extension() {
                    match ext.to_str().unwrap() {
                        "jpg" | "gif" | "png" | "bmp" => is_image = true,
                        _ => (),
                    }
                }
                if is_image {
                    let img = image::SharedImage::load(path.clone()).unwrap();
                    let mut win: window::Window = app::widget_from_id("image_dialog").unwrap();
                    win.resize(win.x(), win.y(), img.w(), img.h());
                    win.child(0).unwrap().set_image(Some(img));
                    win.show();
                } else {
                    STATE.with(move |s| {
                        s.append(Some(path.canonicalize().unwrap()));
                    });
                }
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
        parent.set_damage(true);
    }
}

pub fn tab_splitter_cb(f: &mut frame::Frame, ev: enums::Event) -> bool {
    let mut parent: group::Flex = unsafe { f.parent().unwrap().into_widget() };
    let term: text::SimpleTerminal = app::widget_from_id("term").unwrap();
    match ev {
        enums::Event::Push => true,
        enums::Event::Drag => {
            parent.fixed(&term, parent.h() + parent.y() - app::event_y());
            app::redraw();
            true
        }
        _ => false,
    }
}

pub fn fbr_splitter_cb(f: &mut frame::Frame, ev: enums::Event) -> bool {
    let mut parent: group::Flex = unsafe { f.parent().unwrap().into_widget() };
    let fbr: browser::FileBrowser = app::widget_from_id("fbr").unwrap();
    match ev {
        enums::Event::Push => true,
        enums::Event::Drag => {
            parent.fixed(&fbr, app::event_x());
            app::redraw();
            true
        }
        _ => false,
    }
}
