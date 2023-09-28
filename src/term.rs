use fltk::{enums::*, prelude::*, *};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    env,
    io::{Read, Write},
};

pub struct AnsiTerm {
    st: text::SimpleTerminal,
}

impl AnsiTerm {
    pub fn new() -> Self {
        let mut st = text::SimpleTerminal::default().with_id("term");
        // SimpleTerminal handles many common ansi escape sequence
        st.set_ansi(true);
        let pair = native_pty_system()
            .openpty(PtySize {
                cols: 120,
                rows: 30,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create pty");

        std::env::set_var("TERM", "vt100");
        let mut cmd = if cfg!(target_os = "windows") {
            CommandBuilder::new("cmd.exe")
        } else {
            CommandBuilder::new("/bin/bash")
        };
        cmd.cwd(env::current_dir().unwrap());

        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let mut writer = pair.master.take_writer().unwrap();
        let mut reader = pair.master.try_clone_reader().unwrap();

        std::thread::spawn({
            let mut st = st.clone();
            move || {
                while child.try_wait().is_ok() {
                    let mut msg = [0u8; 1024];
                    if let Ok(sz) = reader.read(&mut msg) {
                        let msg = &msg[0..sz];
                        // we want to handle some escape sequences that the default SimpleTerminal doesn't
                        if msg != b"\x07" {
                            st.append2(msg);
                        }
                        app::awake();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
            }
        });

        st.handle(move |_t, ev| match ev {
            Event::KeyDown => {
                writer.write_all(app::event_text().as_bytes()).unwrap();
                true
            }
            _ => false,
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

        Self { st }
    }
}

fltk::widget_extends!(AnsiTerm, text::SimpleTerminal, st);
