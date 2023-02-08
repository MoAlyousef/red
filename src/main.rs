use fltk::{enums::*, prelude::*, *};
use std::{env, path::PathBuf};
mod state;
mod utils;
use crate::state::State;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MENU_HEIGHT: i32 = if cfg!(target_os = "macos") { 0 } else { 30 };

fn main() {
    let args: Vec<_> = env::args().collect();
    let current_path = if args.len() > 1 {
        PathBuf::from(args[1].clone())
    } else {
        PathBuf::new()
    };
    let a = app::App::default().with_scheme(app::Scheme::Oxy);
    app::get_system_colors();

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let state = State::new(buf.clone(), current_path.clone());
    app::GlobalState::new(state);

    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("RustyEd");
    w.set_xclass("red");
    {
        let mut m = menu::SysMenuBar::default().with_size(WIDTH, MENU_HEIGHT);
        utils::init_menu(&mut m);
        let mut row = group::Flex::default()
            .with_size(WIDTH, HEIGHT - MENU_HEIGHT)
            .row()
            .below_of(&m, 0);
        let mut fbr = browser::FileBrowser::default().with_type(browser::BrowserType::Hold);
        if current_path.exists() && current_path.is_dir() {
            fbr.load(current_path).expect("Not a valid directory!");
            row.set_size(&fbr, 200);
        } else if current_path.exists() {
            buf.load_file(current_path).unwrap();
            row.set_size(&fbr, 1);
        } else {
            row.set_size(&fbr, 1);
        }
        let mut ed = text::TextEditor::default().with_id("ed");
        ed.set_linenumber_width(40);
        ed.set_text_font(Font::Courier);
        ed.set_buffer(buf);
        ed.set_trigger(CallbackTrigger::Changed);
        ed.set_callback(utils::editor_cb);
        row.end();
        fbr.set_callback(utils::fbr_cb);
        w.resizable(&row);
    }
    w.end();
    w.show();
    w.set_callback(utils::win_cb);
    a.run().unwrap();
}
