use std::env;

mod cbs;
mod dialogs;
mod gui;
mod state;
mod utils;

#[cfg(feature = "highlight")]
mod highlight;

#[cfg(feature = "portable-pty")]
mod term;

fn main() {
    let (current_file, current_path) = utils::init_args(env::args());
    let a = gui::init_gui(&current_file, &current_path);
    state::init_state(current_file, current_path);
    a.run().unwrap();
}
