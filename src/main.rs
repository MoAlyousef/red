use std::env;

mod cbs;
#[cfg(feature = "lsp")]
mod completion;
#[cfg(feature = "lsp")]
mod diagnostics;
mod dialogs;
mod fbr;
mod gui;
#[cfg(feature = "lsp")]
mod lsp;
mod state;
mod utils;

#[cfg(feature = "highlight")]
mod highlight;

fn main() {
    let (current_file, current_path) = utils::init_args(env::args());
    // Install diagnostics UI wake handler early
    #[cfg(feature = "lsp")]
    diagnostics::install_awake_handler();
    let a = gui::init_gui(&current_file, &current_path);
    state::init_state(current_file, current_path);
    // Initialize LSP (rust-analyzer) using current directory as root
    // This will be a no-op if rust-analyzer is not available
    #[cfg(feature = "lsp")]
    lsp::init(std::env::current_dir().unwrap());
    a.run().unwrap();
}
