#![allow(clippy::upper_case_acronyms)]

use fltk::{enums::*, prelude::*, *};
use std::fs::File;
use std::io::{Read, Write};
#[cfg(not(target_os = "windows"))]
use std::os::fd::FromRawFd;
#[cfg(target_os = "windows")]
use std::os::windows::io::FromRawHandle;
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
            Command::new("powershell.exe")
        } else {
            let mut cmd = Command::new("/bin/bash");
            cmd.arg("-i");
            cmd
        };

        let pipe = unsafe { create_pipe() };
        let mut child = cmd
            .stdout(file_from_raw(pipe.1))
            .stderr(file_from_raw(pipe.1))
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let mut writer = child.stdin.take().unwrap();
        let mut reader = file_from_raw(pipe.0);

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
                let bytes = app::event_text().into_bytes();
                // if bytes == b"\x03" {
                // writer.write_all(&bytes).unwrap();
                // }
                writer.write_all(&bytes).unwrap();
                true
            }
            _ => false,
        });

        Self { st }
    }
}

fltk::widget_extends!(AnsiTerm, text::SimpleTerminal, st);

#[cfg(not(target_os = "windows"))]
unsafe fn create_pipe() -> (i32, i32) {
    use std::os::raw::*;
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
unsafe fn create_pipe() -> (i32, i32) {
    use std::os::raw::*;
    type HANDLE = *mut c_void;
    type PHANDLE = *mut HANDLE;
    extern "system" {
        fn CreatePipe(rp: PHANDLE, wp: PHANDLE, attrs: *mut (), sz: c_ulong) -> c_int;
    }
    let mut rp = std::ptr::null_mut();
    let mut wp = std::ptr::null_mut();
    let res = CreatePipe(
        &mut rp as PHANDLE,
        &mut wp as PHANDLE,
        std::ptr::null_mut(),
        0,
    );
    if res == 0 {
        panic!("Failed to create pipe!");
    }
    (rp as isize as i32, wp as isize as i32)
}

fn file_from_raw(handle: i32) -> File {
    #[cfg(not(target_os = "windows"))]
    unsafe {
        File::from_raw_fd(handle)
    }

    #[cfg(target_os = "windows")]
    unsafe {
        File::from_raw_handle(handle as isize as *mut _)
    }
}
