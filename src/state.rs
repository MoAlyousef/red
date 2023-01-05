use fltk::{app, text};
use std::path::PathBuf;

pub struct State {
    pub saved: bool,
    pub buf: text::TextBuffer,
    pub current_file: PathBuf,
}

impl State {
    pub fn new(buf: text::TextBuffer) -> Self {
        State {
            saved: false,
            buf,
            current_file: PathBuf::new(),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref STATE: app::GlobalState<State> = app::GlobalState::<State>::get();
}
