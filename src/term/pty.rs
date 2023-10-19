use fltk::app;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use vte::Parser;

pub(crate) fn start(
    mut performer: crate::term::VteParser,
) -> Option<Arc<Mutex<Box<dyn Write + Send>>>> {
    let pair = native_pty_system()
        .openpty(PtySize {
            cols: 120,
            rows: 16,
            pixel_width: 0,
            pixel_height: 0,
        })
        .ok()?;

    let mut cmd = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd.exe")
    } else {
        CommandBuilder::new("/bin/bash")
    };
    cmd.cwd(env::current_dir().ok()?);
    cmd.env("PATH", env::var("PATH").ok()?);

    let mut child = pair.slave.spawn_command(cmd).ok()?;
    let mut reader = pair.master.try_clone_reader().ok()?;
    let writer = pair.master.take_writer().ok()?;
    let writer = Arc::new(Mutex::new(writer));
    std::mem::forget(pair);

    let mut statemachine = Parser::new();

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
    Some(writer)
}
