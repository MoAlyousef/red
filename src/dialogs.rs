#![allow(dead_code)]

use crate::state::STATE;
use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

pub struct FindDialog {
    win: window::Window,
}

impl FindDialog {
    pub fn new() -> Self {
        let idx = Rc::from(RefCell::from(0));
        let mut win = window::Window::new(0, 0, 400, 50, "Find").with_id("find");
        win.set_border(false);
        let mut row = group::Flex::default_fill();
        row.set_margin(10);
        let f = frame::Frame::default().with_label("Find:");
        row.fixed(&f, 30);
        let mut i = input::Input::default();
        i.set_trigger(enums::CallbackTrigger::Changed);
        let mut reg = button::ToggleButton::default().with_label(".*");
        reg.set_selection_color(reg.color().lighter());
        reg.set_tooltip("Use regex");
        row.fixed(&reg, 30);
        let mut b = button::Button::default().with_label("Next");
        let status = frame::Frame::default().with_label("");
        row.fixed(&status, 70);
        // Live count while typing
        i.set_callback({
            let mut status = status.clone();
            let reg = reg.clone();
            let idx_rc = idx.clone();
            move |i| {
                let val = i.value();
                let reg_val = reg.value();
                if reg_val && regex::Regex::new(&val).is_err() {
                    i.set_text_color(enums::Color::Red);
                    status.set_label("err");
                    return;
                } else {
                    i.set_text_color(enums::Color::Foreground);
                }
                if !val.is_empty() {
                    let val2 = val.clone();
                    let mut status2 = status.clone();
                    STATE.with(move |s| {
                        if let Some(buf) = s.buf().as_ref() {
                            let text = buf.text();
                            if reg_val {
                                if let Ok(re) = regex::Regex::new(&val2) {
                                    let total = re.find_iter(&text).count();
                                    status2.set_label(&format!("{}", total));
                                }
                            } else {
                                let total = text.match_indices(&val2).count();
                                status2.set_label(&format!("{}", total));
                            }
                        }
                    });
                    // reset the navigation index on pattern change
                    *idx_rc.borrow_mut() = 0;
                } else {
                    status.set_label("");
                    *idx_rc.borrow_mut() = 0;
                }
            }
        });

        // Recompute counts when regex mode toggles
        reg.set_callback({
            let mut status = status.clone();
            let mut i = i.clone();
            let idx_rc = idx.clone();
            let reg2 = reg.clone();
            move |_| {
                let val = i.value();
                let reg_val = reg2.value();
                if reg_val && regex::Regex::new(&val).is_err() {
                    i.set_text_color(enums::Color::Red);
                    status.set_label("err");
                    return;
                } else {
                    i.set_text_color(enums::Color::Foreground);
                }
                if !val.is_empty() {
                    let val2 = val.clone();
                    let mut status2 = status.clone();
                    STATE.with(move |s| {
                        if let Some(buf) = s.buf().as_ref() {
                            let text = buf.text();
                            if reg_val {
                                if let Ok(re) = regex::Regex::new(&val2) {
                                    let total = re.find_iter(&text).count();
                                    status2.set_label(&format!("{}", total));
                                }
                            } else {
                                let total = text.match_indices(&val2).count();
                                status2.set_label(&format!("{}", total));
                            }
                        }
                    });
                    *idx_rc.borrow_mut() = 0;
                } else {
                    status.set_label("");
                    *idx_rc.borrow_mut() = 0;
                }
            }
        });

        // Navigate to next match on button click
        b.set_callback({
            let mut status = status.clone();
            let reg = reg.clone();
            let idx_rc = idx.clone();
            let i = i.clone();
            move |_| {
                let val = i.value();
                let reg_val = reg.value();
                if val.is_empty() {
                    status.set_label("");
                    return;
                }
                if reg_val && regex::Regex::new(&val).is_err() {
                    status.set_label("err");
                    return;
                }
                let val2 = val.clone();
                let mut status2 = status.clone();
                let idx2 = idx_rc.clone();
                STATE.with(move |s| {
                    if let Some(buf) = s.buf().as_mut() {
                        let text = buf.text();
                        if reg_val {
                            if let Ok(re) = regex::Regex::new(&val2) {
                                let v: Vec<_> = re.find_iter(&text).map(|m| m.range()).collect();
                                if !v.is_empty() {
                                    let mut idx = idx2.borrow_mut();
                                    if *idx >= v.len() {
                                        *idx = 0;
                                    }
                                    let curr = &v[*idx];
                                    let mut ed: text::TextEditor = s.current_editor().unwrap();
                                    buf.select(curr.start as i32, curr.end as i32);
                                    ed.scroll(ed.count_lines(0, curr.start as i32, true), 0);
                                    status2.set_label(&format!("{}/{}", *idx + 1, v.len()));
                                    *idx += 1;
                                    if *idx == v.len() {
                                        *idx = 0;
                                    }
                                } else {
                                    status2.set_label("0/0");
                                }
                            }
                        } else {
                            let v: Vec<_> = text.match_indices(&val2).collect();
                            if !v.is_empty() {
                                let mut idx = idx2.borrow_mut();
                                if *idx >= v.len() {
                                    *idx = 0;
                                }
                                let curr = v[*idx];
                                let mut ed: text::TextEditor = s.current_editor().unwrap();
                                buf.select(curr.0 as i32, (curr.0 + val2.len()) as i32);
                                ed.scroll(ed.count_lines(0, curr.0 as i32, true), 0);
                                status2.set_label(&format!("{}/{}", *idx + 1, v.len()));
                                *idx += 1;
                                if *idx == v.len() {
                                    *idx = 0;
                                }
                            } else {
                                status2.set_label("0/0");
                            }
                        }
                    }
                });
            }
        });

        // Enter key triggers navigation to next
        i.handle({
            let mut status = status.clone();
            let reg = reg.clone();
            let idx_rc = idx.clone();
            move |i, ev| match ev {
                enums::Event::KeyDown => {
                    let k = app::event_key();
                    if k == enums::Key::Enter || k == enums::Key::KPEnter {
                        let val = i.value();
                        let reg_val = reg.value();
                        if val.is_empty() {
                            status.set_label("");
                            return true;
                        }
                        if reg_val && regex::Regex::new(&val).is_err() {
                            status.set_label("err");
                            return true;
                        }
                        let val2 = val.clone();
                        let mut status2 = status.clone();
                        let idx2 = idx_rc.clone();
                        STATE.with(move |s| {
                            if let Some(buf) = s.buf().as_mut() {
                                let text = buf.text();
                                if reg_val {
                                    if let Ok(re) = regex::Regex::new(&val2) {
                                        let v: Vec<_> =
                                            re.find_iter(&text).map(|m| m.range()).collect();
                                        if !v.is_empty() {
                                            let mut idx = idx2.borrow_mut();
                                            if *idx >= v.len() {
                                                *idx = 0;
                                            }
                                            let curr = &v[*idx];
                                            let mut ed: text::TextEditor =
                                                s.current_editor().unwrap();
                                            buf.select(curr.start as i32, curr.end as i32);
                                            ed.scroll(
                                                ed.count_lines(0, curr.start as i32, true),
                                                0,
                                            );
                                            status2.set_label(&format!("{}/{}", *idx + 1, v.len()));
                                            *idx += 1;
                                            if *idx == v.len() {
                                                *idx = 0;
                                            }
                                        } else {
                                            status2.set_label("0/0");
                                        }
                                    }
                                } else {
                                    let v: Vec<_> = text.match_indices(&val2).collect();
                                    if !v.is_empty() {
                                        let mut idx = idx2.borrow_mut();
                                        if *idx >= v.len() {
                                            *idx = 0;
                                        }
                                        let curr = v[*idx];
                                        let mut ed: text::TextEditor = s.current_editor().unwrap();
                                        buf.select(curr.0 as i32, (curr.0 + val2.len()) as i32);
                                        ed.scroll(ed.count_lines(0, curr.0 as i32, true), 0);
                                        status2.set_label(&format!("{}/{}", *idx + 1, v.len()));
                                        *idx += 1;
                                        if *idx == v.len() {
                                            *idx = 0;
                                        }
                                    } else {
                                        status2.set_label("0/0");
                                    }
                                }
                            }
                        });
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        });
        row.fixed(&b, 60);
        row.end();
        win.end();
        win.handle(|win, ev| match ev {
            enums::Event::Hide => {
                win.hide();
                true
            }
            enums::Event::Close => {
                win.hide();
                true
            }
            _ => false,
        });
        Self { win }
    }
}

pub struct ReplaceDialog {
    win: window::Window,
}

impl ReplaceDialog {
    pub fn new() -> Self {
        let mut win = window::Window::new(0, 0, 400, 80, "Replace").with_id("replace");
        win.set_border(false);
        let mut col = group::Flex::default_fill().column();
        col.set_margin(5);
        let mut row = group::Flex::default();
        let f = frame::Frame::default().with_label("Search:");
        row.fixed(&f, 60);
        let mut search = input::Input::default();
        search.set_trigger(enums::CallbackTrigger::Changed);
        let mut reg = button::ToggleButton::default().with_label(".*");
        reg.set_selection_color(reg.color().lighter());
        reg.set_tooltip("Use regex");
        row.fixed(&reg, 30);
        let status = frame::Frame::default().with_label("");
        row.fixed(&status, 90);
        row.end();
        let mut row = group::Flex::default();
        let f = frame::Frame::default().with_label("Replace:");
        row.fixed(&f, 60);
        let replace = input::Input::default();
        let mut in_sel_btn = button::ToggleButton::default().with_label("Sel");
        in_sel_btn.set_tooltip("Replace in selection");
        row.fixed(&in_sel_btn, 40);
        let mut b_next = button::Button::default().with_label("Next");
        b_next.set_tooltip("Replace next");
        row.fixed(&b_next, 60);
        let mut b_all = button::Button::default().with_label("All");
        b_all.set_tooltip("Replace all");
        row.fixed(&b_all, 50);
        row.end();
        col.end();
        win.end();
        search.set_callback({
            let reg = reg.clone();
            move |i| {
                let val = i.value();
                let reg_val = reg.value();
                if reg_val && regex::Regex::new(&val).is_err() {
                    i.set_text_color(enums::Color::Red);
                } else {
                    i.set_text_color(enums::Color::Foreground);
                }
            }
        });
        // Replace next occurrence starting from cursor
        b_next.set_callback({
            let search = search.clone();
            let replace = replace.clone();
            let reg = reg.clone();
            let in_sel_btn = in_sel_btn.clone();
            move |_| {
                let search = search.value();
                let replace = replace.value();
                let reg_val = reg.value();
                if search.is_empty() {
                    return;
                }
                if reg_val && regex::Regex::new(&search).is_err() {
                    return;
                }
                let sel_only = in_sel_btn.value();
                STATE.with({
                    move |s| {
                        if let (Some(mut ed), Some(mut buf)) = (s.current_editor(), s.buf()) {
                            let text = buf.text();
                            let from = ed.insert_position();
                            let (range_start, range_end) = if sel_only {
                                if let Some((a, b)) = buf.selection_position() {
                                    let (a, b) = if a <= b {
                                        (a as usize, b as usize)
                                    } else {
                                        (b as usize, a as usize)
                                    };
                                    (a, b)
                                } else {
                                    (0usize, text.len())
                                }
                            } else {
                                (0usize, text.len())
                            };
                            let clamp_from =
                                from.clamp(range_start as i32, range_end as i32) as usize;
                            if reg_val {
                                if let Ok(re) = regex::Regex::new(&search) {
                                    let mut found = re
                                        .find_at(&text, clamp_from)
                                        .filter(|m| m.start() < range_end);
                                    if found.is_none() {
                                        found = re
                                            .find_at(&text, range_start)
                                            .filter(|m| m.start() < range_end);
                                    }
                                    if let Some(m) = found {
                                        let start = m.start();
                                        let end = m.end();
                                        let rep_cow =
                                            re.replace(&text[start..end], replace.as_str());
                                        buf.replace(start as i32, end as i32, rep_cow.as_ref());
                                        let new_end = start + rep_cow.len();
                                        ed.scroll(ed.count_lines(0, start as i32, true), 0);
                                        buf.select(start as i32, new_end as i32);
                                        ed.set_insert_position(new_end as i32);
                                    }
                                }
                            } else {
                                let mut off = None;
                                if let Some(i) = text[clamp_from..range_end].find(&search) {
                                    off = Some(clamp_from + i);
                                } else if let Some(i) = text.find(&search) {
                                    if i >= range_start && i < range_end {
                                        off = Some(i);
                                    }
                                }
                                if let Some(start) = off {
                                    let end = start + search.len();
                                    buf.replace(start as i32, end as i32, &replace);
                                    let new_end = start + replace.len();
                                    ed.scroll(ed.count_lines(0, start as i32, true), 0);
                                    buf.select(start as i32, new_end as i32);
                                    ed.set_insert_position(new_end as i32);
                                }
                            }
                        }
                    }
                });
            }
        });

        // Replace all occurrences as a single undo step
        b_all.set_callback(move |_| {
            let search = search.value();
            let replace = replace.value();
            let reg_val = reg.value();
            if search.is_empty() {
                return;
            }
            if reg_val && regex::Regex::new(&search).is_err() {
                return;
            }
            let in_sel_btn = in_sel_btn.clone();
            let sel_only = in_sel_btn.value();
            let mut status2 = status.clone();
            STATE.with({
                move |s| {
                    if let Some(buf) = s.buf().as_mut() {
                        let text = buf.text();
                        let (range_start, range_end) = if sel_only {
                            if let Some((a, b)) = buf.selection_position() {
                                let (a, b) = if a <= b {
                                    (a as usize, b as usize)
                                } else {
                                    (b as usize, a as usize)
                                };
                                (a, b)
                            } else {
                                (0usize, text.len())
                            }
                        } else {
                            (0usize, text.len())
                        };
                        let mut replaced = 0usize;
                        if reg_val {
                            if let Ok(re) = regex::Regex::new(&search) {
                                let sub = &text[range_start..range_end];
                                replaced = re.find_iter(sub).count();
                                if replaced > 0 {
                                    let nsub = re.replace_all(sub, replace.as_str());
                                    buf.replace(
                                        range_start as i32,
                                        range_end as i32,
                                        nsub.as_ref(),
                                    );
                                }
                            }
                        } else {
                            let sub = &text[range_start..range_end];
                            replaced = sub.matches(&search).count();
                            if replaced > 0 {
                                let nsub = sub.replace(&search, &replace);
                                buf.replace(range_start as i32, range_end as i32, &nsub);
                            }
                        }
                        status2.set_label(&format!("Replaced: {}", replaced));
                        s.was_modified(true);
                    }
                }
            });
        });
        win.handle(|win, ev| match ev {
            enums::Event::Hide => {
                win.hide();
                true
            }
            enums::Event::Close => {
                win.hide();
                true
            }
            _ => false,
        });
        Self { win }
    }
}

pub struct ImageDialog {
    win: window::Window,
}

impl ImageDialog {
    pub fn new() -> Self {
        let mut win = window::Window::default()
            .with_size(400, 300)
            .with_id("image_dialog");
        let mut f = frame::Frame::default_fill();
        win.end();
        win.resize_callback(move |_win, _, _, w, h| f.resize(0, 0, w, h));
        Self { win }
    }
}

// Lightweight completion dialog: spawned hidden at startup
pub struct CompletionDialog {
    win: window::Window,
}

impl CompletionDialog {
    pub fn new() -> Self {
        let mut win = window::Window::new(0, 0, 360, 220, "").with_id("completion");
        win.set_border(false);
        let _list = browser::HoldBrowser::new(0, 0, 360, 220, None).with_id("completion_list");
        win.end();
        win.handle(|win, ev| match ev {
            enums::Event::Hide | enums::Event::Close => {
                win.hide();
                true
            }
            _ => false,
        });
        // Keep hidden until invoked
        win.hide();
        Self { win }
    }
}
