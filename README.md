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

![image](https://github.com/MoAlyousef/red/assets/37966791/c43a180f-d1db-4528-ace6-d3713dcda202)

## Known issues
- On Windows, the terminal defaults to cmd. Still some of the ansi escape sequences aren't handled on newer Windows versions, and setting the TERM env variable appears not to work (at least on my system).
