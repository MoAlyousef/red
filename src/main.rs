use std::env;

mod cbs;
mod dialogs;
mod fbr;
mod gui;
mod state;
mod utils;

#[cfg(feature = "highlight")]
mod highlight;

unsafe extern "C" {
    pub unsafe fn Fl_load_system_icons();
}

fn main() {
    unsafe {
        Fl_load_system_icons();
    }
    let (current_file, current_path) = utils::init_args(env::args());
    let a = gui::init_gui(&current_file, &current_path);
    state::init_state(current_file, current_path);
    a.run().unwrap();
}
