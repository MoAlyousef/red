use crate::{cbs, dialogs, fbr, utils};
#[cfg(feature = "lsp")]
use crate::lsp;
use fltk::{enums::*, prelude::*, *};
use fltk_theme::color_themes::fleet;
use fltk_theme::{ColorTheme, SchemeType, WidgetScheme};
use std::path::{Path, PathBuf};

#[cfg(feature = "term")]
use fltk_term as term;

#[cfg(feature = "highlight")]
use crate::highlight;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MENU_HEIGHT: i32 = if cfg!(target_os = "macos") { 1 } else { 30 };

#[cfg(feature = "lsp")]
fn schedule_status_refresh() {
    // Poll LSP status until it is no longer "starting"
    app::add_timeout3(0.3, |_| {
        crate::diagnostics::update_status_bar();
        let st = crate::lsp::status_text();
        if st == "starting" {
            schedule_status_refresh();
        }
    });
}

pub fn init_gui(current_file: &Option<PathBuf>, current_path: &Path) -> app::App {
    let a = app::App::default();
    let color_theme = ColorTheme::new(&fleet::GRUVBOX_DARK);
    color_theme.apply();
    let scheme = WidgetScheme::new(SchemeType::Fleet1);
    scheme.apply();
    app::set_menu_linespacing(10);

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let _find_dialog = dialogs::FindDialog::new();
    let _replace_dialog = dialogs::ReplaceDialog::new();
    let _image_dialog = dialogs::ImageDialog::new();
    let _completion_dialog = dialogs::CompletionDialog::new();

    let mut popup = menu::MenuButton::default().with_type(menu::MenuButtonType::Popup3);
    init_edit_menu(&mut popup, "");

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
    tabs.handle(move |t, ev| tabs_handle(t, ev, &mut popup));
    tabs.handle_overflow(group::TabsOverflow::Pulldown);
    tabs.end();
    tabs.auto_layout();
    #[cfg(feature = "term")]
    {
        //  Check if the double prompts is from the wait in fltk-term, originally used for windows!
        let mut tab_splitter = frame::Frame::default();
        tab_splitter.handle(cbs::tab_splitter_cb);
        col.fixed(&tab_splitter, 4);
        let term = term::PPTerm::default();
        col.fixed(&*term, 160);
        std::mem::forget(term);
    }
    col.end();
    row.end();
    #[cfg(feature = "lsp")]
    let lsp_status = lsp::status_text();
    #[cfg(not(feature = "lsp"))]
    let lsp_status = "disabled";
    let info = frame::Frame::default()
        .with_label(&format!(
            "Directory: {}   |   LSP: {}",
            utils::strip_unc_path(current_path),
            lsp_status
        ))
        .with_align(enums::Align::Left | enums::Align::Inside)
        .with_id("info");
    col0.fixed(&info, 20);
    col0.end();
    w.resizable(&row);
    w.end();
    w.make_resizable(true);
    w.show();
    w.set_callback(cbs::win_cb);
    // Kick status refresh until LSP becomes ready/disabled/unavailable
    #[cfg(feature = "lsp")]
    schedule_status_refresh();
    a
}

pub fn tabs_handle(t: &mut group::Tabs, ev: Event, popup: &mut menu::MenuButton) -> bool {
    match ev {
        Event::Push => {
            if app::event_mouse_button() == app::MouseButton::Right
                && app::event_y() > t.y() + 30
                && t.children() > 0
            {
                popup.popup();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn init_edit_menu(m: &mut (impl MenuExt + 'static), header: &str) {
    m.add(
        &format!("{}Undo\t", header),
        Shortcut::Ctrl | 'z',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        &format!("{}Redo\t", header),
        Shortcut::Ctrl | 'y',
        menu::MenuFlag::MenuDivider,
        cbs::menu_cb,
    );
    m.add(
        &format!("{}Cut\t", header),
        Shortcut::Ctrl | 'x',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        &format!("{}Copy\t", header),
        Shortcut::Ctrl | 'c',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        &format!("{}Paste\t", header),
        Shortcut::Ctrl | 'v',
        menu::MenuFlag::MenuDivider,
        cbs::menu_cb,
    );
    m.add(
        &format!("{}Find\t", header),
        Shortcut::Ctrl | 'f',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        &format!("{}Replace\t", header),
        Shortcut::Ctrl | 'h',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
}
pub fn init_menu(m: &mut (impl MenuExt + 'static), load_dir: bool) {
    m.add(
        "&File/New File...\t",
        Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&File/New Dir...\t",
        Shortcut::Ctrl | Shortcut::Shift | 'n',
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
        Shortcut::Ctrl | Shortcut::Shift | 'w',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    m.add(
        "&File/Save All\t",
        Shortcut::None,
        menu::MenuFlag::MenuDivider,
        cbs::menu_cb,
    );
    let _idx = m.add(
        "&File/Quit\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        cbs::menu_cb,
    );
    // m.at(_idx).unwrap().set_label_color(Color::Red.darker());
    init_edit_menu(m, "&Edit/");
    let idx = m.add(
        "&View/File browser\t",
        Shortcut::None,
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
            Shortcut::None,
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
    // Handle Ctrl+Space for completion
    ed.handle(|e, ev| {
        if let Event::Shortcut | Event::KeyDown = ev {
            #[cfg(feature = "lsp")]
            if app::event_state().contains(Shortcut::Ctrl) && app::event_key() == Key::from_char(' ') {
                // Avoid activity if LSP is unavailable
                if !crate::lsp::is_available() {
                    return true;
                }
                // request completion at current caret position
                if let Some(buf) = e.buffer() {
                    let pos = e.insert_position();
                    let text = buf.text();
                    let lsp_pos = crate::lsp::compute_position_from_offset(&text, pos as usize);
                    let ed_ptr = e.as_widget_ptr() as usize;
                    // find path for current editor
                    crate::state::STATE.with(move |s| {
                        if let Some(v) = s.map.get(&ed_ptr) {
                            if let Some(path) = v.current_file.as_ref() {
                                let path = path.clone();
                                crate::lsp::with_client(move |c| {
                                    c.completion(&path, lsp_pos, move |resp| {
                                        // Build popup entries
                                        let items = match resp {
                                            lsp_types::CompletionResponse::Array(arr) => arr,
                                            lsp_types::CompletionResponse::List(list) => list.items,
                                        };
                                        let mut entries = Vec::with_capacity(items.len());
                                        for item in items.into_iter() {
                                            let insert_text = item.insert_text.clone().unwrap_or_else(|| item.label.clone());
                                            let edit_range: Option<lsp_types::Range> = item.text_edit.as_ref().map(|te| match te {
                                                lsp_types::CompletionTextEdit::Edit(ed_) => ed_.range,
                                                lsp_types::CompletionTextEdit::InsertAndReplace(ir) => ir.replace,
                                            });
                                            entries.push(crate::completion::CompletionEntry {
                                                label: item.label,
                                                insert_text,
                                                edit_range,
                                                kind: item.kind,
                                                detail: item.detail,
                                            });
                                        }
                                        let entries_arc = std::sync::Arc::new(entries);
                                        app::awake_callback(move || {
                                            let entries_vec = (*entries_arc).clone();
                                            let maybe_ed = crate::state::STATE.with(|s3| s3.current_editor());
                                            if let Some(ed_now) = maybe_ed {
                                                crate::completion::show_popup(&ed_now, entries_vec);
                                            }
                                        });
                                    });
                                });
                            }
                        }
                    });
                }
                return true;
            }
        }
        false
    });
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
        std::thread::spawn({
            let p = p.clone();
            let mut ed = ed.clone();
            let mut buf = buf.clone();
            move || {
                highlight::highlight(&p, &mut ed, &mut buf);
            }
        });
    }
    ed.set_buffer(buf);
    ed
}
