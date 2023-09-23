use fltk::{enums::*, prelude::*, *};
use std::{env, path::PathBuf};
mod dialogs;
mod state;
#[cfg(feature = "portable-pty")]
mod term;
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
        env::current_dir().unwrap()
    };
    let a = app::App::default().with_scheme(app::Scheme::Oxy);
    app::get_system_colors();

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("RustyEd");
    w.set_xclass("red");
    let mut col = group::Flex::default_fill().column();
    let mut m = menu::SysMenuBar::default().with_id("menu");
    utils::init_menu(&mut m);
    col.fixed(&m, MENU_HEIGHT);
    let mut row = group::Flex::default();
    let mut fbr = browser::FileBrowser::default().with_type(browser::BrowserType::Hold);
    if current_path.exists() {
        if current_path.is_dir() {
            #[allow(clippy::redundant_clone)]
            fbr.load(current_path.clone())
                .expect("Not a valid directory!");
            if args.len() < 2 {
                row.fixed(&fbr, 1);
            } else {
                row.fixed(&fbr, 180);
            }
        } else {
            buf.load_file(current_path.clone()).unwrap();
            w.set_label(&format!("{} - RustyEd", current_path.display()));
            row.fixed(&fbr, 1);
        }
    } else {
        w.set_label(&format!("{} - RustyEd", current_path.display()));
        row.fixed(&fbr, 1);
    }
    #[allow(unused_mut)]
    let mut col = group::Flex::default().column();
    let mut tabs = group::Tabs::default().with_id("tabs");
    let mut edrow = group::Flex::default()
        .row()
        .with_label(if current_path.is_dir() {
            "untitled"
        } else {
            current_path.file_name().unwrap().to_str().unwrap()
        })
        .with_id("edrow");
    edrow.set_trigger(CallbackTrigger::Closed);
    edrow.set_callback(utils::tab_close_cb);
    let mut ed = text::TextEditor::default().with_id("ed");
    ed.set_linenumber_width(40);
    ed.set_text_font(Font::Courier);
    ed.set_buffer(buf.clone());
    ed.set_trigger(CallbackTrigger::Changed);
    ed.set_callback(utils::editor_cb);
    edrow.end();
    tabs.end();
    tabs.auto_layout();
    #[cfg(feature = "portable-pty")]
    {
        let term = crate::term::AnsiTerm::default();
        utils::init_term(&term, current_path.clone());
        col.fixed(&*term, 160);
    }
    col.end();
    row.end();
    fbr.set_callback(utils::fbr_cb);
    w.resizable(&row);
    col.end();
    w.end();
    w.make_resizable(true);
    w.show();
    w.set_callback(utils::win_cb);

    let state = State::new(&ed, buf, current_path, None, "edrow");
    app::GlobalState::new(state);

    a.run().unwrap();
}
