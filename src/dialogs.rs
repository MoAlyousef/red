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
        let mut win = window::Window::new(0, 0, 300, 50, "Find");
        win.set_border(false);
        let mut row = group::Flex::default_fill();
        row.set_margin(10);
        let f = frame::Frame::default().with_label("Find:");
        row.fixed(&f, 30);
        let mut i = input::Input::default();
        i.set_trigger(enums::CallbackTrigger::EnterKeyAlways);
        i.set_callback({
            move |i| {
                let val = i.value();
                if !val.is_empty() {
                    STATE.with({
                        let idx = idx.clone();
                        move |s| {
                            let text = s.buf.text();
                            let v: Vec<_> = text.match_indices(&val).collect();
                            let mut idx = idx.borrow_mut();
                            let curr = v[*idx];
                            let mut ed: text::TextEditor = app::widget_from_id("ed").unwrap();
                            s.buf.select(curr.0 as i32, (curr.0 + val.len()) as i32);
                            ed.scroll(ed.count_lines(0, curr.0 as i32, true), 0);
                            *idx += 1;
                            if *idx == v.len() {
                                *idx = 0;
                            }
                        }
                    });
                }
            }
        });
        let mut b = button::Button::default().with_label("Next");
        b.set_callback(move |_| i.do_callback());
        row.fixed(&b, 60);
        row.end();
        win.end();
        win.set_callback(|w| {
            window::Window::delete(w.clone());
        });
        Self { win }
    }
    pub fn show(&mut self) {
        let win = app::first_window().unwrap();
        self.win
            .resize(win.x() + win.w() - 300, win.y() + 30, 300, 50);
        self.win.show();
    }
}

pub struct ReplaceDialog {
    win: window::Window,
}

impl ReplaceDialog {
    pub fn new() -> Self {
        let mut win = window::Window::new(0, 0, 300, 80, "Replace");
        win.set_border(false);
        let mut col = group::Flex::default_fill().column();
        col.set_margin(5);
        let mut row = group::Flex::default();
        let f = frame::Frame::default().with_label("Search:");
        row.fixed(&f, 60);
        let search = input::Input::default();
        let f = frame::Frame::default();
        row.fixed(&f, 60);
        row.end();
        let mut row = group::Flex::default();
        let f = frame::Frame::default().with_label("Replace:");
        row.fixed(&f, 60);
        let replace = input::Input::default();
        let mut b = button::Button::default().with_label("Apply!");
        b.set_callback(move |_| {
            let search = search.value();
            let replace = replace.value();
            STATE.with({
                move |s| {
                    let text = s.buf.text();
                    let ntext = text.replace(&search, &replace);
                    s.buf.set_text(&ntext);
                    s.was_modified(true);
                }
            });
        });
        row.fixed(&b, 60);
        row.end();
        col.end();
        win.end();
        win.set_callback(|w| {
            window::Window::delete(w.clone());
        });
        Self { win }
    }
    pub fn show(&mut self) {
        let win = app::first_window().unwrap();
        self.win
            .resize(win.x() + win.w() - 300, win.y() + 30, 300, 80);
        self.win.show();
    }
}
