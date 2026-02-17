# cbx: CBOR Explorer

A TUI application for inspecting CBOR (Concise Binary Object Representation) files, built with Rust and Ratatui.

![A demo GIF recording](demo.gif)

## Installation

### Pre-built Binaries

Pre-built binaries for Linux (.deb, .snap), Windows, and macOS (untested) are available in [Releases](https://github.com/yurikrupenin/cbor-explorer/releases).

### Cargo

```bash
cargo install --git https://github.com/yurikrupenin/cbor-explorer
```

### Build from Source

```bash
cargo build --release
./target/release/cbx <file.cbor>
```

## Usage

```bash
cbx <FILE>
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `?` | Toggle Help |
| `q` | Quit |
| `Tab` | Switch focus (Tree / Hex) |
| `t` | Switch Theme |
| `/` | Search |
| `n` / `N` | Next / Previous match |
| `:` | Go to Offset |
| `m` | Toggle Scan Mode (Single / Auto) |
| `s` | Sort chunks (Score / Offset) |
| `x` | Toggle Hex / Dec integers |
| `e` | Expand all nodes |
| `c` | Collapse all nodes |
| `Enter` / `Space` | Toggle node expansion |
| `j` / `↓` | Move Down |
| `k` / `↑` | Move Up |
| `h` / `←` | Move Left (Hex view) |
| `l` / `→` | Move Right (Hex view) |
| `g` / `Home` | Go to Start |
| `G` / `End` | Go to End |

### Modes

By default, `cbx` assumes that the file contains a single CBOR sequence.

If your file contains multiple embedded CBOR sequences, press `m` to toggle **Auto Mode**. This uses simple heuristics to scan the file and identify potential CBOR sequences.

The scanning is not perfect but works pretty well for discovering complex nested structures.

## License

MIT
