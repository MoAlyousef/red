use fltk::{enums::*, prelude::*, *};
use std::path::PathBuf;

mod utils;
use crate::utils::*;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

struct State {
    saved: bool,
    buf: text::TextBuffer,
    current_file: PathBuf,
}

impl State {
    fn new(buf: text::TextBuffer) -> Self {
        State {
            saved: false,
            buf,
            current_file: PathBuf::new(),
        }
    }
}

lazy_static::lazy_static! {
    static ref STATE: app::GlobalState<State> = app::GlobalState::<State>::get();
}

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Oxy);
    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);
    let state = State::new(buf.clone());
    app::GlobalState::new(state);
    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Ted");
    {
        let mut m = menu::MenuBar::new(0, 0, WIDTH, 30, None);
        m.add_choice(
            "File/New|File/Open...|File/Save...|File/_Save As...|File/Quit|Edit/Cut|Edit/Copy|Edit/Paste|Help/About"
        );
        m.set_callback(menu_cb);
        let mut ed = text::TextEditor::new(0, 30, WIDTH, HEIGHT - 30, None).with_id("ed");
        ed.set_linenumber_width(40);
        ed.set_buffer(buf);
        ed.set_trigger(CallbackTrigger::Changed);
        ed.set_callback(editor_cb);
        w.resizable(&ed);
    }
    w.end();
    w.show();
    w.set_callback(win_cb);
    a.run().unwrap();
}
