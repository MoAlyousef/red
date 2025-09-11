use std::env;

mod cbs;
mod completion;
mod diagnostics;
mod dialogs;
mod fbr;
mod gui;
mod lsp;
mod state;
mod utils;

#[cfg(feature = "highlight")]
mod highlight;

fn main() {
    let (current_file, current_path) = utils::init_args(env::args());
    // Install diagnostics UI wake handler early
    diagnostics::install_awake_handler();
    let a = gui::init_gui(&current_file, &current_path);
    state::init_state(current_file, current_path);
    // Initialize LSP (rust-analyzer) using current directory as root
    // This will be a no-op if rust-analyzer is not available
    lsp::init(std::env::current_dir().unwrap());
    a.run().unwrap();
}
