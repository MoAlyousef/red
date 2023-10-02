#![allow(dead_code)]

use crate::utils;
use fltk::{enums::*, prelude::*, *};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::{
    env,
    io::{self, Read, Write},
    str,
    sync::{Arc, Mutex},
    thread,
};

pub struct PPTerm {
    st: text::SimpleTerminal,
    pair: Arc<Mutex<PtyPair>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PPTerm {
    pub fn new() -> Self {
        let mut st = text::SimpleTerminal::default().with_id("term");
        // SimpleTerminal handles many common ansi escape sequence
        st.set_ansi(true);
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
            env::set_var("TERM", "vt100");
            CommandBuilder::new("/bin/bash")
        };
        cmd.cwd(env::current_dir().unwrap());
        cmd.env("PATH", env::var("PATH").unwrap());

        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();
        let pair = Arc::new(Mutex::new(pair));
        let writer = Arc::new(Mutex::new(writer));

        #[cfg(not(windows))]
        {
            thread::spawn({
                let mut st = st.clone();
                move || {
                    let mut s = Vec::new();
                    while child.try_wait().is_ok() {
                        let mut msg = [0u8; 1024];
                        if let Ok(sz) = reader.read(&mut msg) {
                            let msg = &msg[0..sz];
                            s.extend_from_slice(&msg[0..sz]);
                            match str::from_utf8(&s) {
                                Ok(text) => {
                                    if text != "\x07" {
                                        st.append(text);
                                    }
                                    s.clear();
                                }
                                Err(z) => {
                                    let z = z.valid_up_to();
                                    st.append2(&msg[0..z]);
                                    s.extend_from_slice(&msg[z..]);
                                }
                            }
                            app::awake();
                        }
                        app::sleep(0.03);
                    }
                }
            });
        }

        #[cfg(windows)]
        {
            // windows quirk
            app::sleep(0.03);
            thread::spawn({
                let mut st = st.clone();
                move || {
                    // let mut s = Vec::new();
                    while child.try_wait().is_ok() {
                        let mut msg = [0u8; 1024];
                        if let Ok(sz) = reader.read(&mut msg) {
                            let msg = &msg[0..sz];
                            st.append2(msg);
                        }
                        app::sleep(0.03);
                    }
                }
            });
        }

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

        Self { st, pair, writer }
    }

    pub fn write_all(&self, s: &[u8]) -> Result<(), io::Error> {
        self.writer.lock().unwrap().write_all(s)
    }
}

fltk::widget_extends!(PPTerm, text::SimpleTerminal, st);

pub struct XTerm {
    xterm_win: window::Window,
}

impl XTerm {
    pub fn new() -> Self {
        let mut xterm_win = window::Window::default().with_id("term");
        xterm_win.end();
        xterm_win.set_color(Color::Black);
        if utils::can_use_xterm() {
            app::add_timeout3(0.1, {
                let xterm_win = xterm_win.clone();
                move |_h| {
                    #[allow(clippy::unnecessary_cast)]
                    let handle = xterm_win.raw_handle() as u64;
                    std::process::Command::new("xterm")
                        .args([
                            "-into",
                            &format!("{}", handle),
                            "-bg",
                            "black",
                            "-fg",
                            "white",
                            "-fa",
                            "'Monospace'",
                            "-fs",
                            "10",
                        ])
                        .spawn()
                        .unwrap();
                }
            });
        }
        Self { xterm_win }
    }
}

fltk::widget_extends!(XTerm, window::Window, xterm_win);
