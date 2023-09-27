use fltk::{enums::*, prelude::*, *};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AnsiTerm {
    st: text::SimpleTerminal,
    pub writer1: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
}

impl AnsiTerm {
    pub fn new<L: Into<Option<&'static str>>>(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        label: L,
        current_path: PathBuf,
    ) -> Self {
        let mut st = text::SimpleTerminal::new(x, y, w, h, label).with_id("term");
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

        let mut cmd = if cfg!(target_os = "windows") {
            CommandBuilder::new("cmd.exe")
        } else if cfg!(target_os = "macos") {
            CommandBuilder::new("/bin/zsh")
        } else {
            std::env::set_var("TERM", "VT100");
            let mut cmd = CommandBuilder::new("/bin/bash");
            cmd.args(["-i"]);
            cmd
        };
        cmd.cwd(current_path);

        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let writer = Arc::new(Mutex::new(pair.master.take_writer().unwrap()));
        let writer1 = writer.clone();
        let mut reader = pair.master.try_clone_reader().unwrap();

        std::thread::spawn({
            let mut st = st.clone();
            move || {
                while child.try_wait().is_ok() {
                    let mut msg = [0u8; 1024];
                    if let Ok(sz) = reader.read(&mut msg) {
                        let msg = &msg[0..sz];
                        // we want to handle some escape sequences that the default SimpleTerminal doesn't
                        format(msg, &mut st);
                        app::awake();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
            }
        });

        st.handle(move |_t, ev| match ev {
            Event::KeyDown => {
                let mut writer = writer.lock().unwrap();
                writer.write_all(app::event_text().as_bytes()).unwrap();
                true
            }
            _ => false,
        });

        Self { st, writer1 }
    }
}

fltk::widget_extends!(AnsiTerm, text::SimpleTerminal, st);

fn format(msg: &[u8], st: &mut text::SimpleTerminal) {
    // handles the sticky title-bell sequence
    // if let Some(pos0) = msg.windows(4).position(|m| m == b"\x1b]0;") {
    //     let mut pos1 = pos0 + 1;
    //     while pos1 < msg.len() - 1 && msg[pos1] != b'\x1b' {
    //         pos1 += 1;
    //     }
    //     let pre = &msg[0..pos0];
    //     let post = &msg[pos1..];
    //     if !pre.is_empty() {
    //         if let Ok(s) = String::from_utf8(pre.to_vec()) {
    //             st.append(&s);
    //         }
    //     }
    //     if !post.is_empty() {
    //         st.append2(post);
    //     }
    // } else
    if msg != b"\x07" {
        st.append2(msg);
    }
}
