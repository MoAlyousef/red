# red

red or the RustyEditor is a lightweight and minimal text editor which supports multiple-tabs (à la vscode) and which integrates a file browser and a terminal. The editor component supports syntax highlighting via [tree-sitter-highlight](https://github.com/tree-sitter/tree-sitter/tree/master/highlight).

The terminal itself is quite minimal, supports a limited subset of ansi escape sequences (don't expect to run vim in it!), and is powered by [portable-pty](https://github.com/wez/wezterm/pty). 

## Building
```bash
git clone https://github.com/MoAlyousef/red
cd red
cargo build --release
```

To disable building with tree-sitter:
```bash
cargo build --no-default-features --release
```