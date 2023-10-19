#![allow(dead_code)]
#![allow(clippy::single_match)]

use fltk::{enums::*, prelude::*, *};
use std::{
    io::{self, Write},
    str,
    sync::{Arc, Mutex},
};
mod pty;
mod styles;
mod vte_parser;

const UP: &[u8] = if cfg!(not(target_os = "windows")) {
    b"\x10"
} else {
    b"\x1b[A"
};
const DOWN: &[u8] = if cfg!(not(target_os = "windows")) {
    b"\x0E"
} else {
    b"\x1b[B"
};

pub(crate) struct VteParser {
    ch: char,
    st: text::TextDisplay,
    sbuf: text::TextBuffer,
    temp_s: String,
    temp_b: String,
}

impl VteParser {
    pub fn new(st: text::TextDisplay, sbuf: text::TextBuffer) -> Self {
        Self {
            ch: 'A',
            st,
            sbuf,
            temp_s: String::new(),
            temp_b: String::new(),
        }
    }
    pub fn myprint(&mut self) {
        let mut buf = self.st.buffer().unwrap();
        buf.append2(self.temp_s.as_bytes());
        self.sbuf.append2(self.temp_b.as_bytes());
        self.st.set_insert_position(buf.length());
        self.st
            .scroll(self.st.count_lines(0, buf.length(), true), 0);
        self.temp_s.clear();
        self.temp_b.clear();
    }
}

pub fn menu_cb(m: &mut impl MenuExt) {
    let term: text::TextDisplay = app::widget_from_id("term").unwrap();
    if let Ok(mpath) = m.item_pathname(None) {
        match mpath.as_str() {
            "Copy\t" => app::copy2(&term.buffer().unwrap().selection_text()),
            "Paste\t" => app::paste_text2(&term),
            _ => (),
        }
    }
}

pub fn init_menu(m: &mut (impl MenuExt + 'static)) {
    m.add(
        "Copy\t",
        Shortcut::Ctrl | Key::Insert,
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "Paste\t",
        Shortcut::Shift | Key::Insert,
        menu::MenuFlag::Normal,
        menu_cb,
    );
}

pub struct PPTerm {
    g: group::Group,
    st: text::TextDisplay,
    writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
}

impl PPTerm {
    pub fn new() -> Self {
        let mut g = group::Group::default().with_id("term_group");
        let mut st = text::TextDisplay::default().with_id("term");
        let mut m = menu::MenuButton::default()
            .with_type(menu::MenuButtonType::Popup3)
            .with_id("pop2");
        init_menu(&mut m);
        g.end();
        st.show_cursor(true);
        st.set_color(Color::Black);
        st.set_cursor_style(text::Cursor::Block);
        st.wrap_mode(text::WrapMode::AtBounds, 0);
        let buf = text::TextBuffer::default();
        st.set_buffer(buf);
        let styles = styles::init();
        let sbuf = text::TextBuffer::default();
        st.set_highlight_data(sbuf.clone(), styles);

        g.resize_callback({
            let mut st = st.clone();
            move |_, x, y, w, h| {
                m.resize(x, y, w, h);
                st.resize(x, y, w, h);
            }
        });

        let performer = VteParser::new(st.clone(), sbuf);
        let writer = pty::start(performer);

        if let Some(writer) = writer.as_ref() {
            st.handle({
                let writer = writer.clone();
                move |t, ev| match ev {
                    Event::KeyDown => {
                        let key = app::event_key();
                        match key {
                            #[cfg(windows)]
                            Key::BackSpace => writer.lock().unwrap().write_all(b"\x7f").unwrap(),
                            Key::Up => writer.lock().unwrap().write_all(UP).unwrap(),
                            Key::Down => writer.lock().unwrap().write_all(DOWN).unwrap(),
                            // Key::Left => writer.lock().unwrap().write_all(b"\x1b[D").unwrap(),
                            // Key::Right => writer.lock().unwrap().write_all(b"\x1b[C").unwrap(),
                            _ => {
                                if app::event_state() == EventState::Ctrl | EventState::Shift {
                                    if key == Key::from_char('v') {
                                        app::paste_text2(t);
                                    }
                                } else {
                                    let txt = app::event_text();
                                    writer.lock().unwrap().write_all(txt.as_bytes()).unwrap();
                                }
                            }
                        }
                        true
                    }
                    Event::Paste => {
                        let txt = app::event_text();
                        writer.lock().unwrap().write_all(txt.as_bytes()).unwrap();
                        true
                    }
                    _ => false,
                }
            });
        }

        Self { g, st, writer }
    }

    pub fn write_all(&self, s: &[u8]) -> Result<(), io::Error> {
        if let Some(writer) = &self.writer {
            writer.lock().unwrap().write_all(s)
        } else {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Failed to write to pty!",
            ))
        }
    }
}

fltk::widget_extends!(PPTerm, group::Group, g);
