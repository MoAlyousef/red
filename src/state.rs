use fltk::{app, prelude::WidgetExt, text, utils::oncelock::Lazy};
use std::path::PathBuf;

pub struct State {
    pub is_saved: bool,
    pub buf: text::TextBuffer,
    pub current_file: PathBuf,
    pub current_dir: PathBuf,
}

impl State {
    pub fn new(buf: text::TextBuffer, current_path: PathBuf) -> Self {
        State {
            is_saved: true,
            buf,
            current_file: if !current_path.is_dir() {
                current_path.clone()
            } else {
                PathBuf::new()
            },
            current_dir: if current_path.is_dir() {
                current_path
            } else {
                PathBuf::new()
            },
        }
    }
    pub fn saved(&mut self, flag: bool) {
        self.is_saved = flag;
        if flag {
            app::first_window()
                .unwrap()
                .set_label(&format!("{} - RustyEd", self.current_file.display()));
        } else {
            app::first_window()
                .unwrap()
                .set_label(&format!("*{} - RustyEd", self.current_file.display()));
        }
    }
}

pub static STATE: Lazy<app::GlobalState<State>> = Lazy::new(app::GlobalState::<State>::get);
