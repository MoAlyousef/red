use fltk::{enums::*, prelude::*, *};
mod state;
mod utils;
use crate::state::State;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MENU_HEIGHT: i32 = if cfg!(target_os = "macos") { 0 } else { 30 };

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Oxy);
    app::get_system_colors();

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let state = State::new(buf.clone());
    app::GlobalState::new(state);

    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Ted");
    w.set_xclass("ted");
    {
        let mut m = menu::SysMenuBar::default().with_size(WIDTH, MENU_HEIGHT);
        utils::init_menu(&mut m);

        let mut ed = text::TextEditor::default()
            .with_size(WIDTH, HEIGHT - MENU_HEIGHT)
            .below_of(&m, 0)
            .with_id("ed");
        ed.set_linenumber_width(40);
        ed.set_text_font(Font::Courier);
        ed.set_buffer(buf);
        ed.set_trigger(CallbackTrigger::Changed);
        ed.set_callback(utils::editor_cb);
        w.resizable(&ed);
    }
    w.end();
    w.show();
    w.set_callback(utils::win_cb);
    a.run().unwrap();
}
