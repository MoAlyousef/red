#![allow(dead_code)]

use fltk::{enums::*, prelude::*, *};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    env,
    io::{self, Read, Write},
    mem, str,
    sync::{Arc, Mutex},
    thread,
};
use vte::{Params, Parser, Perform};

macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                eprintln!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
}

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

fn styles() -> Vec<text::StyleTableEntry> {
    vec![
        text::StyleTableEntry {
            color: Color::White,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Red,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Green,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Yellow,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::from_hex(0x61afef),
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Magenta,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Cyan,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::White,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Foreground,
            font: Font::Courier,
            size: 14,
        },
    ]
}

struct VteParser {
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
        buf.append(&self.temp_s);
        self.st.set_insert_position(buf.length());
        self.st
            .scroll(self.st.count_lines(0, buf.length(), true), 0);
        self.sbuf.append(&self.temp_b);
        self.temp_s.clear();
        self.temp_b.clear();
    }
}

impl Perform for VteParser {
    fn print(&mut self, c: char) {
        let mut tmp = [0u8; 4];
        let s = c.encode_utf8(&mut tmp);
        self.temp_s.push_str(s);
        self.temp_b.push(self.ch);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            8 => {
                // backspace
                let mut buf = self.st.buffer().unwrap();
                let c = buf.text().chars().last().unwrap();
                let mut tmp = [0u8; 4];
                let s = c.encode_utf8(&mut tmp);
                buf.remove(buf.length() - s.len() as i32, buf.length());
                self.sbuf.remove(self.sbuf.length() - 1, self.sbuf.length());
            }
            10 | 13 => {
                // crlf
                self.temp_s.push(byte as char);
                self.temp_b.push(self.ch);
            }
            0 | 7 => (), // tabs?
            _ => {
                debug!("unhandled byte: {}", byte);
            }
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        debug!(
            "[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}",
            params, intermediates, ignore, c
        );
    }

    fn put(&mut self, byte: u8) {
        debug!("[put] {:02x}", byte);
    }

    fn unhook(&mut self) {
        debug!("[unhook]");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        debug!(
            "[osc_dispatch] params={:?} bell_terminated={}",
            params, bell_terminated
        );
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        match c {
            'm' => {
                for p in params {
                    match p {
                        [31] => self.ch = 'B',
                        [32] => self.ch = 'C',
                        [33] => self.ch = 'D',
                        [34] => self.ch = 'E',
                        [35] => self.ch = 'F',
                        [36] => self.ch = 'G',
                        [37] => self.ch = 'H',
                        [38] => self.ch = 'I',
                        [39] => self.ch = 'J',
                        [0] => self.ch = 'A',
                        _ => {
                            // debug!(
                            //     "[csi_dispatch] params={:#?} intermediates={:?}, ignore={:?}, char={}",
                            //     params, intermediates, ignore, c
                            // );
                            self.ch = 'A';
                        }
                    }
                }
            }
            _ => {
                debug!(
                    "[csi_dispatch] params={:#?} intermediates={:?}, ignore={:?}, char={}",
                    params, intermediates, ignore, c
                );
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        debug!(
            "[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}",
            intermediates, ignore, byte
        );
    }
}

pub struct PPTerm {
    st: text::TextDisplay,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PPTerm {
    pub fn new() -> Self {
        let mut st = text::TextDisplay::default().with_id("term");
        st.show_cursor(true);
        st.set_color(Color::Black);
        st.set_cursor_style(text::Cursor::Block);
        st.wrap_mode(text::WrapMode::AtBounds, 0);
        let buf = text::TextBuffer::default();
        st.set_buffer(buf);
        let styles = styles();
        let sbuf = text::TextBuffer::default();
        st.set_highlight_data(sbuf.clone(), styles);
        let pair = native_pty_system()
            .openpty(PtySize {
                cols: 80,
                rows: 24,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create pty");

        let mut cmd = if cfg!(target_os = "windows") {
            CommandBuilder::new("cmd.exe")
        } else {
            CommandBuilder::new("/bin/bash")
        };
        cmd.cwd(env::current_dir().unwrap());
        cmd.env("PATH", env::var("PATH").unwrap());

        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();
        mem::forget(pair);
        let writer = Arc::new(Mutex::new(writer));

        let mut statemachine = Parser::new();
        let mut performer = VteParser::new(st.clone(), sbuf);

        #[cfg(windows)]
        app::sleep(0.05);

        thread::spawn({
            move || {
                while child.try_wait().is_ok() {
                    let mut msg = [0u8; 4096];
                    if let Ok(sz) = reader.read(&mut msg) {
                        let msg = &msg[0..sz];
                        for byte in msg {
                            statemachine.advance(&mut performer, *byte);
                        }
                        performer.myprint();
                        app::awake();
                    }
                    app::sleep(0.03);
                }
            }
        });

        st.handle({
            let writer = writer.clone();
            move |t, ev| match ev {
                Event::KeyDown => {
                    let key = app::event_key();
                    if key == Key::Up {
                        writer.lock().unwrap().write_all(UP).unwrap();
                    } else if key == Key::Down {
                        writer.lock().unwrap().write_all(DOWN).unwrap();
                    } else if key == Key::from_char('v')
                        && app::event_state() == EventState::Ctrl | EventState::Shift
                    {
                        app::paste_text2(t);
                    } else {
                        let txt = app::event_text();
                        writer.lock().unwrap().write_all(txt.as_bytes()).unwrap();
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

        Self { st, writer }
    }

    pub fn write_all(&self, s: &[u8]) -> Result<(), io::Error> {
        self.writer.lock().unwrap().write_all(s)
    }
}

fltk::widget_extends!(PPTerm, text::TextDisplay, st);
