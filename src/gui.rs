use crate::{cbs, dialogs, fbr, utils};
use fltk::{enums::*, prelude::*, *};
use fltk_theme::{SchemeType, WidgetScheme};
use std::path::{Path, PathBuf};

#[cfg(feature = "term")]
use crate::term;

#[cfg(feature = "highlight")]
use crate::highlight;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MENU_HEIGHT: i32 = if cfg!(target_os = "macos") { 1 } else { 30 };

pub fn init_gui(current_file: &Option<PathBuf>, current_path: &Path) -> app::App {
    let a = app::App::default();
    let widget_scheme = WidgetScheme::new(SchemeType::Gleam);
    widget_scheme.apply();
    app::set_menu_linespacing(10);
    app::set_background_color(0x21, 0x25, 0x2b);
    app::set_background2_color(0x28, 0x2c, 0x34);
    app::set_foreground_color(0xab, 0xb2, 0xa2);
    app::set_color(Color::Selection, 0x32, 0x38, 0x42);
    // todo
    app::set_color(Color::Inactive, 88, 0, 0);

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let _find_dialog = dialogs::FindDialog::new();
    let _replace_dialog = dialogs::ReplaceDialog::new();
    let _image_dialog = dialogs::ImageDialog::new();

    let mut popup = menu::MenuButton::default().with_type(menu::MenuButtonType::Popup3);
    init_edit_menu(&mut popup);

    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("RustyEd");
    w.set_xclass("red");

    let mut col0 = group::Flex::default_fill().column();
    col0.set_pad(2);
    let mut m = menu::SysMenuBar::default().with_id("menu");
    m.set_color(Color::Background2);
    init_menu(&mut m, current_file.is_none());
    col0.fixed(&m, MENU_HEIGHT);
    let mut row = group::Flex::default();
    row.set_pad(0);
    let fbr = fbr::Fbr::new(current_path);
    if current_file.is_none() {
        row.fixed(&*fbr, 180);
    } else {
        row.fixed(&*fbr, 1);
    }
    let mut fbr_splitter = frame::Frame::default();
    fbr_splitter.handle(cbs::fbr_splitter_cb);
    row.fixed(&fbr_splitter, 4);
    let mut col = group::Flex::default().column();
    col.set_pad(0);
    let mut tabs = group::Tabs::default().with_id("tabs");
    tabs.handle(move |t, ev| match ev {
        Event::Push => {
            if app::event_mouse_button() == app::MouseButton::Right && app::event_y() > t.y() + 30 {
                popup.popup();
                true
            } else {
                false
            }
        }
        _ => false,
    });
    tabs.handle_overflow(group::TabsOverflow::Pulldown);
    tabs.end();
    tabs.auto_layout();
    #[cfg(feature = "term")]
    {
        let mut tab_splitter = frame::Frame::default();
        tab_splitter.handle(cbs::tab_splitter_cb);
        col.fixed(&tab_splitter, 4);
        let term = term::PPTerm::new();
        col.fixed(&*term, 160);
    }
    col.end();
    row.end();
    let info = frame::Frame::default()
        .with_label(&format!(
            "Directory: {}",
            utils::strip_unc_path(current_path)
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
    w.set_callback(cbs::win_cb);
    a
}

pub fn init_edit_menu(m: &mut (impl MenuExt + 'static)) {
    m.add(
        "&Edit/Undo\t",
        Shortcut::Ctrl | 'z',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&Edit/Redo\t",
        Shortcut::Ctrl | 'y',
        menu::MenuFlag::MenuDivider,
        cbs::menu_cb,
    );
    m.add(
        "&Edit/Cut\t",
        Shortcut::Ctrl | 'x',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&Edit/Copy\t",
        Shortcut::Ctrl | 'c',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&Edit/Paste\t",
        Shortcut::Ctrl | 'v',
        menu::MenuFlag::MenuDivider,
        cbs::menu_cb,
    );
    m.add(
        "&Edit/Find\t",
        Shortcut::Ctrl | 'f',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&Edit/Replace\t",
        Shortcut::Ctrl | 'h',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
}
pub fn init_menu(m: &mut (impl MenuExt + 'static), load_dir: bool) {
    m.add(
        "&File/New...\t",
        Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&File/Open...\t",
        Shortcut::Ctrl | 'o',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&File/Save\t",
        Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&File/Save as...\t",
        Shortcut::Ctrl | 'w',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&File/Save All\t",
        Shortcut::Ctrl | 'w',
        menu::MenuFlag::MenuDivider,
        cbs::menu_cb,
    );
    let idx = m.add(
        "&File/Quit\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.at(idx).unwrap().set_label_color(Color::Red);
    init_edit_menu(m);
    let idx = m.add(
        "&View/File browser\t",
        Shortcut::Ctrl | 'f',
        menu::MenuFlag::Toggle,
        cbs::menu_cb,
    );
    if load_dir {
        m.at(idx).unwrap().set();
    }
    #[cfg(feature = "term")]
    {
        let idx = m.add(
            "&View/Terminal\t",
            Shortcut::Ctrl | 'h',
            menu::MenuFlag::Toggle,
            cbs::menu_cb,
        );
        m.at(idx).unwrap().set();
    }
    m.add(
        "&Help/About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
}

pub fn init_editor(ed: &mut text::TextEditor) {
    ed.set_linenumber_width(40);
    ed.set_linenumber_size(12);
    ed.set_linenumber_fgcolor(Color::Yellow.darker());
    ed.set_linenumber_bgcolor(Color::Background);
    ed.set_text_font(Font::Courier);
    ed.set_trigger(CallbackTrigger::Changed);
    ed.set_callback(cbs::editor_cb);
}

pub fn create_ed(
    tabs: &mut group::Tabs,
    id: &str,
    current_path: &Option<PathBuf>,
) -> text::TextEditor {
    tabs.begin();
    let mut edrow = group::Flex::default()
        .row()
        .with_label(if let Some(current_path) = current_path.as_ref() {
            if current_path.is_dir() {
                "untitled"
            } else {
                current_path.file_name().unwrap().to_str().unwrap()
            }
        } else {
            "untitled"
        })
        .with_id(id);
    edrow.set_trigger(CallbackTrigger::Closed);
    edrow.set_callback(cbs::tab_close_cb);
    let mut ed = text::TextEditor::default().with_id("ed");
    init_editor(&mut ed);
    edrow.end();
    tabs.end();
    tabs.auto_layout();
    tabs.set_value(&edrow).ok();

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);
    if let Some(p) = current_path.as_ref() {
        buf.load_file(p).ok();
        #[cfg(feature = "highlight")]
        highlight::highlight(p, &mut ed, &mut buf);
    }
    ed.set_buffer(buf);
    ed
}
