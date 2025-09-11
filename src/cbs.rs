use crate::state::STATE;
use fltk::{enums::*, prelude::*, *};
use std::{fs, path::PathBuf};

fn nfc_get_file(mode: dialog::NativeFileChooserType) -> PathBuf {
    let mut nfc = dialog::NativeFileChooser::new(mode);
    nfc.show();
    nfc.filename()
}

fn find() {
    let mut dlg: window::Window = app::widget_from_id("find").unwrap();
    let main_win = app::first_window().unwrap();
    dlg.resize(
        main_win.x() + main_win.w() - 400,
        main_win.y() + 30,
        400,
        50,
    );
    dlg.hide();
    dlg.show();
}

fn replace() {
    let mut dlg: window::Window = app::widget_from_id("replace").unwrap();
    let main_win = app::first_window().unwrap();
    dlg.resize(
        main_win.x() + main_win.w() - 400,
        main_win.y() + 30,
        400,
        80,
    );
    dlg.hide();
    dlg.show();
}

pub fn win_cb(_: &mut window::Window) {
    if app::event() == Event::Close {
        app::quit();
    }
}

pub fn editor_cb(_e: &mut text::TextEditor) {
    app::add_timeout3(0.01, |_| STATE.with(|s| s.was_modified(true)));
    // Debounced didChange: bump change_seq, schedule a send in configurable delay
    let debounce_ms: f64 = std::env::var("RED_LSP_DEBOUNCE_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v as f64 / 1000.0)
        .unwrap_or(0.20);
    if let Some((id, seq)) = STATE.with(|s| {
        if let Some(current_id) = s.current_id() {
            s.map.get(&current_id).map(|mb| (current_id, mb.change_seq))
        } else {
            None
        }
    }) {
        // bump seq
        STATE.with(move |s| {
            if let Some(mb) = s.map.get_mut(&id) {
                mb.change_seq = mb.change_seq.wrapping_add(1);
            }
        });
        let scheduled_seq = seq + 1;
        app::add_timeout3(debounce_ms, move |_| {
            STATE.with(move |s2| {
                if let Some(mb) = s2.map.get_mut(&id) {
                    if mb.change_seq == scheduled_seq {
                        // latest edit reached debounce; send didChange with current text
                        mb.version += 1;
                        let version = mb.version;
                        if let Some(path) = mb.current_file.clone() {
                            let text = mb.buf.text();
                            crate::lsp::with_client(|c| c.did_change_full(&path, &text, version));
                        }
                    }
                }
            });
        });
    }
}

pub fn new_file() {
    let dlg = dialog::input_default("Enter file name", "");
    if let Some(f) = dlg {
        fs::File::create(f).ok();
    }
}

pub fn new_dir() {
    let dlg = dialog::input_default("Enter directory name", "");
    if let Some(f) = dlg {
        fs::create_dir(f).ok();
    }
}

pub fn menu_cb(m: &mut impl MenuExt) {
    if let Ok(mpath) = m.item_pathname(None) {
        match mpath.as_str() {
            "&File/New File...\t" => {
                new_file();
            }
            "&File/New Dir...\t" => {
                new_dir();
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
                        let (modified, current_file, contents) = {
                            let e = s.map.get(&id).unwrap();
                            (e.modified, e.current_file.clone(), e.buf.text())
                        };
                        if let Some(ref current_file) = current_file {
                            if modified && current_file.exists() {
                                fs::write(current_file, contents).ok();
                                s.was_modified(false);
                                crate::lsp::with_client(|c| c.did_save(current_file));
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
                            fs::write(&c, buf.text()).expect("Failed to write to file!");
                            s.was_modified(false);
                            crate::lsp::with_client(|cl| cl.did_save(&c));
                        }
                    });
                }
            }
            "&File/Save All\t" => {
                STATE.with(|s| {
                    for v in s.map.values_mut() {
                        if v.modified && v.current_file.as_ref().unwrap().exists() {
                            fs::write(v.current_file.as_ref().unwrap(), v.buf.text()).ok();
                            v.modified = true;
                        }
                    }
                });
            }
            "&File/Quit\t" => app::quit(),
            "/Undo\t" | "&Edit/Undo\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.undo()
                }
            }),
            "/Redo\t" | "&Edit/Redo\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.redo()
                }
            }),
            "/Cut\t" | "&Edit/Cut\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.cut()
                }
            }),
            "/Copy\t" | "&Edit/Copy\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.copy()
                }
            }),
            "/Paste\t" | "&Edit/Paste\t" => STATE.with(|s| {
                if let Some(e) = s.current_editor() {
                    e.paste()
                }
            }),
            "/Find\t" | "&Edit/Find\t" => find(),
            "/Replace\t" | "&Edit/Replace\t" => replace(),
            "&View/File browser\t" => {
                let mut item = m.at(m.value()).unwrap();
                let fbr: group::Group = app::widget_from_id("fbr_group").unwrap();
                let mut parent = group::Flex::from_dyn_widget(&fbr.parent().unwrap()).unwrap();
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
                let term: group::Scroll = app::widget_from_id("term_group").unwrap();
                let mut parent = group::Flex::from_dyn_widget(&term.parent().unwrap()).unwrap();
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

pub fn tab_close_cb(g: &mut impl GroupExt) {
    if app::callback_reason() == CallbackReason::Closed {
        let ed = text::TextEditor::from_dyn_widget(&g.child(0).unwrap()).unwrap();
        let edid = ed.as_widget_ptr() as usize;
        let buf = ed.buffer().unwrap();
        // LSP didClose for this file if any
        STATE.with(move |s| {
            if let Some(v) = s.map.get(&edid) {
                if let Some(path) = v.current_file.as_ref() {
                    crate::lsp::with_client(|c| c.did_close(path));
                }
            }
        });
        let mut parent = g.parent().unwrap();
        parent.remove(g);
        unsafe {
            text::TextBuffer::delete(buf);
        }
        STATE.with(move |s| s.map.remove(&edid));
        parent.set_damage(true);
    }
}

#[cfg(feature = "term")]
pub fn tab_splitter_cb(f: &mut frame::Frame, ev: Event) -> bool {
    let mut parent = group::Flex::from_dyn_widget(&f.parent().unwrap()).unwrap();
    let term = app::widget_from_id::<group::Scroll>("term_group").unwrap();
    match ev {
        Event::Push => true,
        Event::Drag => {
            parent.fixed(&term, parent.h() + parent.y() - app::event_y());
            app::redraw();
            true
        }
        Event::Enter => {
            f.window().unwrap().set_cursor(Cursor::NS);
            true
        }
        Event::Leave => {
            f.window().unwrap().set_cursor(Cursor::Arrow);
            true
        }
        _ => false,
    }
}

pub fn fbr_splitter_cb(f: &mut frame::Frame, ev: Event) -> bool {
    let mut parent = group::Flex::from_dyn_widget(&f.parent().unwrap()).unwrap();
    let fbr: group::Group = app::widget_from_id("fbr_group").unwrap();
    match ev {
        Event::Push => true,
        Event::Drag => {
            parent.fixed(&fbr, app::event_x());
            app::redraw();
            true
        }
        Event::Enter => {
            f.window().unwrap().set_cursor(Cursor::WE);
            true
        }
        Event::Leave => {
            f.window().unwrap().set_cursor(Cursor::Arrow);
            true
        }
        _ => false,
    }
}
