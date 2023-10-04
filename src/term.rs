#![allow(dead_code)]

use fltk::{enums::*, prelude::*, *};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    env,
    io::{self, Read, Write},
    mem,
    str,
    sync::{Arc, Mutex},
    thread,
};
use vte::{Params, Parser, Perform};

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
    ch: &'static str,
    st: text::SimpleTerminal,
    sbuf: text::TextBuffer,
}

impl VteParser {
    pub fn new(st: text::SimpleTerminal, sbuf: text::TextBuffer) -> Self {
        Self {
            ch: "A",
            st,
            sbuf,
        }
    }
}

impl Perform for VteParser {
    fn print(&mut self, c: char) {
        let mut tmp = [0u8; 4];
        let s = c.encode_utf8(&mut tmp);
        self.st.append(s);
        self.sbuf.append(self.ch);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            8 => { // backspace
                let mut buf = self.st.buffer().unwrap();
                buf.remove(buf.length() - 1, buf.length());
                self.sbuf.remove(buf.length() - 1, buf.length());
            },
            10 | 13 => { // crlf
                self.st.append(&(byte as char).to_string());
                self.sbuf.append(self.ch);
            },
            0 | 7 => (),
            _ => {
                println!("unhandled byte: {}", byte);
            }
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        println!(
            "[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}",
            params, intermediates, ignore, c
        );
    }

    fn put(&mut self, byte: u8) {
        println!("[put] {:02x}", byte);
    }

    fn unhook(&mut self) {
        println!("[unhook]");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        println!("[osc_dispatch] params={:?} bell_terminated={}", params, bell_terminated);
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'm' => {
                for p in params {
                    match p {
                        [31] => self.ch = "B",
                        [32] => self.ch = "C",
                        [33] => self.ch = "D",
                        [34] => self.ch = "E",
                        [35] => self.ch = "F",
                        [36] => self.ch = "G",
                        [37] => self.ch = "H",
                        [38] => self.ch = "I",
                        [39] => self.ch = "J",
                        [0] => self.ch = "A",
                        _ => {
                            // println!("ignored m param: {:?}", p);
                            self.ch = "A";
                        }
                    }
                }
            },
            _ => {
                println!("ignored csi char {}", c);
            },
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        println!(
            "[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}",
            intermediates, ignore, byte
        );
    }
}

pub struct PPTerm {
    st: text::SimpleTerminal,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PPTerm {
    pub fn new() -> Self {
        let mut st = text::SimpleTerminal::default().with_id("term");
        let styles = styles();
        let sbuf = text::TextBuffer::default();
        st.set_highlight_data(sbuf.clone(), styles);
        // SimpleTerminal handles many common ansi escape sequence
        // st.set_ansi(true);
        let pair = native_pty_system()
            .openpty(PtySize {
                cols: 80,
                rows: 24,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create pty");

        let mut cmd = if cfg!(target_os = "windows") {
            env::set_var("TERM", "xterm-mono");
            CommandBuilder::new("cmd.exe")
        } else {
            env::set_var("TERM", "vt100");
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
                        writer.lock().unwrap().write_all(b"\x10").unwrap();
                        t.scroll(t.count_lines(0, t.buffer().unwrap().length(), true), 0);
                    } else if key == Key::Down {
                        writer.lock().unwrap().write_all(b"\x0E").unwrap();
                    } else if key == Key::from_char('v') && app::event_state() == EventState::Ctrl {
                        app::paste(t);
                    } else {
                        let txt = app::event_text();
                        writer.lock().unwrap().write_all(txt.as_bytes()).unwrap();
                    }
                    true
                }
                Event::KeyUp => {
                    if app::event_key() == Key::Up {
                        t.scroll(t.count_lines(0, t.buffer().unwrap().length(), true), 0);
                        true
                    } else {
                        false
                    }
                }
                Event::Paste => {
                    let txt = app::event_text();
                    writer.lock().unwrap().write_all(txt.as_bytes()).unwrap();
                    true
                }
                _ => false,
            }
        });

        st.set_cursor_style(text::Cursor::Dim);
        // for a blinking cursors
        app::add_timeout3(1.0, {
            let mut st = st.clone();
            move |h| {
                if !st.has_focus() {
                    if st.cursor_style() == text::Cursor::Block {
                        st.set_cursor_style(text::Cursor::Dim);
                    } else {
                        st.set_cursor_style(text::Cursor::Block);
                    }
                } else {
                    st.set_cursor_style(text::Cursor::Block);
                }
                app::repeat_timeout3(1.0, h);
            }
        });

        Self { st, writer }
    }

    pub fn write_all(&self, s: &[u8]) -> Result<(), io::Error> {
        self.writer.lock().unwrap().write_all(s)
    }
}

fltk::widget_extends!(PPTerm, text::SimpleTerminal, st);
