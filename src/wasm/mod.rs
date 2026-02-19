/// WASM bindings for terminal UI
/// Beware: color support is completely LLM-generated
use crate::app::App;
use crate::input::{KeyCode, KeyEvent, KeyModifiers};
use crate::ui;
use ratatui::backend::Backend;
use ratatui::buffer::Cell;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Terminal;
use std::cell::RefCell;

use std::io::{self, Write};
use std::panic;
use wasm_bindgen::prelude::*;

// Bindings to JavaScript
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // Function to write data to the xterm.js instance
    #[wasm_bindgen(js_name = writeToTerminal)]
    fn write_to_terminal(data: &[u8]);
}

// Custom writer that bridges data to JS
struct WasmLogWriter;

impl Write for WasmLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        write_to_terminal(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Global state using RefCell/Rc since WASM is single-threaded
struct WasmState {
    app: App,
    terminal: Terminal<WasmBackend>,
}

thread_local! {
    static STATE: RefCell<Option<WasmState>> = const { RefCell::new(None) };
    static TERMINAL_SIZE: RefCell<(u16, u16)> = const { RefCell::new((80, 24)) };
}

// Custom Backend that emits ANSI codes
struct WasmBackend {
    writer: WasmLogWriter,
    buffer: String, // Internal buffer to format strings before writing
}

impl WasmBackend {
    fn new() -> Self {
        Self {
            writer: WasmLogWriter,
            buffer: String::new(),
        }
    }

    fn write_color(&mut self, color: Color, is_bg: bool) -> io::Result<()> {
        use std::fmt::Write;
        match color {
            Color::Reset => Ok(()), // Handled by reset code
            Color::Black => write!(self.buffer, "\x1b[{}0m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Red => write!(self.buffer, "\x1b[{}1m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Green => write!(self.buffer, "\x1b[{}2m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Yellow => write!(self.buffer, "\x1b[{}3m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Blue => write!(self.buffer, "\x1b[{}4m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Magenta => write!(self.buffer, "\x1b[{}5m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Cyan => write!(self.buffer, "\x1b[{}6m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Gray => write!(self.buffer, "\x1b[{}7m", if is_bg { "4" } else { "3" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::DarkGray => write!(self.buffer, "\x1b[{}0m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::LightRed => write!(self.buffer, "\x1b[{}1m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::LightGreen => write!(self.buffer, "\x1b[{}2m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::LightYellow => write!(self.buffer, "\x1b[{}3m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::LightBlue => write!(self.buffer, "\x1b[{}4m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::LightMagenta => write!(self.buffer, "\x1b[{}5m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::LightCyan => write!(self.buffer, "\x1b[{}6m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::White => write!(self.buffer, "\x1b[{}7m", if is_bg { "10" } else { "9" })
                .map_err(|_| io::Error::other("fmt error")),
            Color::Rgb(r, g, b) => write!(
                self.buffer,
                "\x1b[{};2;{};{};{}m",
                if is_bg { "48" } else { "38" },
                r,
                g,
                b
            )
            .map_err(|_| io::Error::other("fmt error")),
            Color::Indexed(i) => write!(
                self.buffer,
                "\x1b[{};5;{}m",
                if is_bg { "48" } else { "38" },
                i
            )
            .map_err(|_| io::Error::other("fmt error")),
        }
    }
}

impl Backend for WasmBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        use std::fmt::Write;

        let mut last_y = 0;
        let mut last_x = 0;
        let mut moved = false;

        for (x, y, cell) in content {
            // Move cursor if needed
            if y != last_y || x != last_x + 1 || !moved {
                write!(self.buffer, "\x1b[{};{}H", y + 1, x + 1)
                    .map_err(|_| io::Error::other("fmt error"))?;
                moved = true;
            }
            last_y = y;
            last_x = x;

            // Simple optimization: reset first
            write!(self.buffer, "\x1b[0m").map_err(|_| io::Error::other("fmt error"))?;

            self.write_color(cell.fg, false)?;
            self.write_color(cell.bg, true)?;

            if cell.modifier.contains(ratatui::style::Modifier::BOLD) {
                write!(self.buffer, "\x1b[1m").map_err(|_| io::Error::other("fmt error"))?;
            }
            if cell.modifier.contains(ratatui::style::Modifier::UNDERLINED) {
                write!(self.buffer, "\x1b[4m").map_err(|_| io::Error::other("fmt error"))?;
            }
            if cell.modifier.contains(ratatui::style::Modifier::REVERSED) {
                write!(self.buffer, "\x1b[7m").map_err(|_| io::Error::other("fmt error"))?;
            }

            write!(self.buffer, "{}", cell.symbol()).map_err(|_| io::Error::other("fmt error"))?;
        }

        self.writer.write_all(self.buffer.as_bytes())?;
        self.buffer.clear();
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.writer, "\x1b[?25l")
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.writer, "\x1b[?25h")
    }

    fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
        Ok(ratatui::layout::Position::new(0, 0))
    }

    fn set_cursor_position<P: Into<ratatui::layout::Position>>(
        &mut self,
        position: P,
    ) -> io::Result<()> {
        let p = position.into();
        write!(self.writer, "\x1b[{};{}H", p.y + 1, p.x + 1)
    }

    fn clear(&mut self) -> io::Result<()> {
        write!(self.writer, "\x1b[2J")
    }

    fn size(&self) -> io::Result<ratatui::layout::Size> {
        let (cols, rows) = TERMINAL_SIZE.with(|s| *s.borrow());
        Ok(ratatui::layout::Size::new(cols, rows))
    }

    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        let (cols, rows) = TERMINAL_SIZE.with(|s| *s.borrow());
        Ok(ratatui::backend::WindowSize {
            columns_rows: (cols, rows).into(),
            pixels: (0, 0).into(),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[wasm_bindgen]
pub fn init_app(file_name: String, data: Vec<u8>) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let app = App::new(file_name, data).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let backend = WasmBackend::new();
    let terminal = Terminal::new(backend).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let state = WasmState { app, terminal };

    // Draw once
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });

    redraw()?;

    Ok(())
}

fn redraw() -> Result<(), JsValue> {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            let _ = state
                .terminal
                .draw(|f| ui::draw(f, &mut state.app))
                .map_err(|e| JsValue::from_str(&e.to_string()));
        }
        Ok(())
    })
}

#[wasm_bindgen]
pub fn on_key(key: &str, ctrl: bool, alt: bool, shift: bool) -> Result<(), JsValue> {
    let code = match key {
        "Enter" => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "ArrowUp" => KeyCode::Up,
        "ArrowDown" => KeyCode::Down,
        "ArrowLeft" => KeyCode::Left,
        "ArrowRight" => KeyCode::Right,
        "Tab" => KeyCode::Tab,
        "Delete" => KeyCode::Delete,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Escape" => KeyCode::Esc,
        " " => KeyCode::Char(' '),
        c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
        _ => return Ok(()),
    };

    let mut modifiers = KeyModifiers::empty();
    if ctrl {
        modifiers.insert(KeyModifiers::CONTROL);
    }
    if alt {
        modifiers.insert(KeyModifiers::ALT);
    }
    if shift {
        modifiers.insert(KeyModifiers::SHIFT);
    }

    let event = KeyEvent { code, modifiers };

    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            let _ = ui::handle_input(&mut state.app, event);
            let _ = state.terminal.draw(|f| ui::draw(f, &mut state.app));
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub fn resize(cols: u16, rows: u16) -> Result<(), JsValue> {
    TERMINAL_SIZE.with(|s| *s.borrow_mut() = (cols, rows));

    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            let _ = state.terminal.resize(Rect {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            });
            let _ = state.terminal.draw(|f| ui::draw(f, &mut state.app));
        }
    });
    Ok(())
}
