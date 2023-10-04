# red

red or the RustyEditor is a lightweight and minimal text editor which supports multiple-tabs (à la vscode) and which integrates a file browser and a terminal. The editor component supports syntax highlighting via [tree-sitter-highlight](https://github.com/tree-sitter/tree-sitter/tree/master/highlight).

The terminal itself is quite minimal, supports a limited subset of ansi escape sequences using [vte](https://github.com/alacritty/vte), i.e. don't expect to run vim in it!, and is powered by [portable-pty](https://github.com/wez/wezterm/pty). 

## Building
```bash
git clone https://github.com/MoAlyousef/red
cd red
cargo build --release
```

To disable building with tree-sitter and the terminal:
```bash
cargo build --no-default-features --release
```

To build with native wayland support on Linux:
```bash
cargo build --features=fltk/use-wayland --release
```

![image](https://github.com/MoAlyousef/red/assets/37966791/c43a180f-d1db-4528-ace6-d3713dcda202)

## Known issues
- On Windows, the terminal defaults to cmd. More ansi escape sequences need to be handled to support powershell. 
- If you're running linux and no icons appear in the FileBrowser, you probably need to install icon files or upgrade your system, which I had to on kubuntu 20.04.
- Highlighting via tree-sitter seems to vary between different language modules. tree-sitter-json seems quite limited for example.

## ToDo
- Add a settings dialog.
- Save and get user settings using fltk [Preferences](https://docs.rs/fltk/latest/fltk/app/prefs/struct.Preferences.html).
- Enable using FLTK's FileChooser instead of the system provided one via the settings.
- Support popup menus via MenuButton with type MenuButtonType::Popup3 on the FileBrowser, editor and terminal.
- Support user provided color schemes for the app and the highlighting.

