use crate::state::STATE;
use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

pub struct FindDialog {
    win: window::Window,
    idx: Rc<RefCell<usize>>,
}

impl FindDialog {
    pub fn new() -> Self {
        let idx = Rc::from(RefCell::from(0));
        let win = window::Window::new(100, 100, 300, 50, "Find").with_id("find_dialog").center_screen();
        let mut row = group::Flex::default_fill();
        row.set_margin(10);
        let f = frame::Frame::default().with_label("Find:");
        row.set_size(&f, 30);
        let mut i = input::Input::default();
        i.set_callback({
            let idx = idx.clone();
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
        row.set_size(&b, 60);
        row.end();
        win.end();
        Self { win, idx }
    }
    pub fn show(&mut self) {
        self.win.show();
    }
}
