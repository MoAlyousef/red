use fltk::{app, text, utils::oncelock::Lazy};
use std::path::PathBuf;

pub struct State {
    pub saved: bool,
    pub buf: text::TextBuffer,
    pub current_file: PathBuf,
    pub current_dir: PathBuf,
}

impl State {
    pub fn new(buf: text::TextBuffer, current_dir: PathBuf) -> Self {
        State {
            saved: false,
            buf,
            current_file: PathBuf::new(),
            current_dir,
        }
    }
}

pub static STATE: Lazy<app::GlobalState<State>> = Lazy::new(|| app::GlobalState::<State>::get());
