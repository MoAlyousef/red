use fltk::{enums::*, prelude::*, *};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::process::{Command, Stdio};

pub struct AnsiTerm {
    st: text::SimpleTerminal,
}

impl AnsiTerm {
    pub fn new() -> Self {
        let mut st = text::SimpleTerminal::default().with_id("term");
        // SimpleTerminal handles many common ansi escape sequence
        st.set_ansi(true);

        std::env::set_var("TERM", "vt100");
        let mut cmd = if cfg!(target_os = "windows") {
            Command::new("cmd.exe")
        } else {
            let mut cmd = Command::new("/bin/bash");
            cmd.arg("-i");
            cmd
        };

        let pipe = unsafe { create_pipe() };
        let stdio = create_stdio(pipe.1);
        let stderr = create_stdio(pipe.1);
        let mut child = cmd
            .stdout(stdio)
            .stderr(stderr)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let mut writer = child.stdin.take().unwrap();
        let mut reader = unsafe { File::from_raw_fd(pipe.0) };

        std::thread::spawn({
            let mut st = st.clone();
            move || {
                while child.try_wait().is_ok() {
                    let mut msg = [0u8; 1024];
                    if let Ok(sz) = reader.read(&mut msg) {
                        let msg = &msg[0..sz];
                        if msg != b"\x07" {
                            st.append2(msg);
                            app::awake();
                        }
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

        Self { st }
    }
}

fltk::widget_extends!(AnsiTerm, text::SimpleTerminal, st);

unsafe fn create_pipe() -> (i32, i32) {
    use std::os::raw::*;
    #[cfg(not(target_os = "windows"))]
    {
        let mut fds: [c_int; 2] = [0; 2];
        extern "C" {
            fn pipe(arg: *mut i32) -> i32;
        }
        let res = pipe(fds.as_mut_ptr());
        if res != 0 {
            panic!("Failed to create pipe!");
        }
        (fds[0], fds[1])
    }

    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn CreatePipe(rp: *mut isize, wp: *mut isize, attrs: *mut (), sz: c_ulong) -> c_int;
        }
        let mut rp = -1isize;
        let mut wp = -1isize;
        let res = CreatePipe(&mut rp as _, &mut wp as _, std::ptr::null_mut(), 0);
        if res == 0 {
            panic!("Failed to create pipe!");
        }
        (rp as i32, wp as i32)
    }
}

fn create_stdio(fd: i32) -> Stdio {
    #[cfg(unix)]
    unsafe {
        Stdio::from_raw_fd(fd)
    }
    #[cfg(windows)]
    unsafe {
        Stdio::from_raw_handle(fd)
    }
}
