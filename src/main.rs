use fltk::{enums::*, prelude::*, *};
use fltk_theme::{SchemeType, WidgetScheme};
use std::{env, path::PathBuf};

mod dialogs;
mod highlight;
mod state;
mod term;
mod utils;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MENU_HEIGHT: i32 = if cfg!(target_os = "macos") { 1 } else { 30 };

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut current_file: Option<PathBuf> = None;
    // fix our working dir
    if args.len() > 1 {
        let path = PathBuf::from(args[1].clone());
        if path.exists() {
            if path.is_dir() {
                env::set_current_dir(path.clone()).unwrap();
            } else {
                current_file = Some(PathBuf::from(path.file_name().unwrap()));
                if let Some(parent) = path.parent() {
                    if parent.exists() {
                        env::set_current_dir(parent).unwrap();
                    }
                }
            }
        }
        path
    } else {
        env::current_dir().unwrap()
    };

    let current_path = env::current_dir().unwrap().canonicalize().unwrap();

    let a = app::App::default();
    let widget_scheme = WidgetScheme::new(SchemeType::Gleam);
    widget_scheme.apply();
    app::set_menu_linespacing(10);
    app::set_background_color(0x21, 0x25, 0x2b);
    app::set_background2_color(0x28, 0x2c, 0x34);
    app::set_foreground_color(0xab, 0xb2, 0xa2);
    app::set_color(Color::Selection, 0x32, 0x38, 0x42);

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let _find_dialog = dialogs::FindDialog::new();
    let _replace_dialog = dialogs::ReplaceDialog::new();
    let _image_dialog = dialogs::ImageDialog::new();

    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("RustyEd");
    w.set_xclass("red");
    let mut col0 = group::Flex::default_fill().column();
    col0.set_pad(2);
    let mut m = menu::SysMenuBar::default().with_id("menu");
    m.set_color(Color::Background2);
    utils::init_menu(&mut m, current_file.is_none());
    col0.fixed(&m, MENU_HEIGHT);
    let mut row = group::Flex::default();
    row.set_pad(0);
    let mut fbr = browser::FileBrowser::default()
        .with_type(browser::BrowserType::Hold)
        .with_id("fbr");
    fbr.load(current_path.clone())
        .expect("Failed to load working directory");
    fbr.set_color(Color::Background.darker());
    if current_file.is_none() {
        row.fixed(&fbr, 180);
    } else {
        w.set_label(&format!("{} - RustyEd", current_path.display()));
        row.fixed(&fbr, 1);
    }
    let mut fbr_splitter = frame::Frame::default();
    fbr_splitter.handle(utils::fbr_splitter_cb);
    row.fixed(&fbr_splitter, 4);
    let mut col = group::Flex::default().column();
    col.set_pad(0);
    let mut tabs = group::Tabs::default().with_id("tabs");
    tabs.handle_overflow(group::TabsOverflow::Pulldown);
    tabs.end();
    tabs.auto_layout();
    let mut tab_splitter = frame::Frame::default();
    tab_splitter.handle(utils::tab_splitter_cb);
    col.fixed(&tab_splitter, 4);
    if utils::can_use_xterm() {
        let term = term::XTerm::new();
        col.fixed(&*term, 160);
    } else {
        let term = term::PPTerm::new();
        col.fixed(&*term, 160);
    }
    col.end();
    row.end();
    let info = frame::Frame::default()
        .with_label(&format!(
            "Directory: {}",
            utils::strip_unc_path(&current_path)
        ))
        .with_align(enums::Align::Left | enums::Align::Inside)
        .with_id("info");
    col0.fixed(&info, 20);
    col0.end();
    w.resizable(&row);
    w.end();
    w.make_resizable(true);
    w.show();

    // callbacks
    fbr.set_callback(utils::fbr_cb);
    w.set_callback(utils::win_cb);

    let mut state = state::State::new(current_path);
    state.append(current_file);
    app::GlobalState::new(state);

    a.run().unwrap();
}
