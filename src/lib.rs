pub mod app;
pub mod cbor_parser;
pub mod cbor_tree;
pub mod config;
pub mod config_store;
pub mod input;
pub mod scanner;
pub mod theme;
pub mod ui;
pub mod util;
pub mod zoom;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use app::App;
#[cfg(not(target_arch = "wasm32"))]
use color_eyre::Result;
#[cfg(not(target_arch = "wasm32"))]
use ratatui::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        return Ok(());
                    }

                    // Convert crossterm key to crate::input::KeyEvent
                    // TODO: Do we... actually need this? Blindly copypasted
                    //       from tutorial+LLM-slop.
                    let code = match key.code {
                        KeyCode::Char(c) => input::KeyCode::Char(c),
                        KeyCode::Enter => input::KeyCode::Enter,
                        KeyCode::Backspace => input::KeyCode::Backspace,
                        KeyCode::Left => input::KeyCode::Left,
                        KeyCode::Right => input::KeyCode::Right,
                        KeyCode::Up => input::KeyCode::Up,
                        KeyCode::Down => input::KeyCode::Down,
                        KeyCode::Home => input::KeyCode::Home,
                        KeyCode::End => input::KeyCode::End,
                        KeyCode::PageUp => input::KeyCode::PageUp,
                        KeyCode::PageDown => input::KeyCode::PageDown,
                        KeyCode::Tab => input::KeyCode::Tab,
                        KeyCode::BackTab => input::KeyCode::BackTab,
                        KeyCode::Delete => input::KeyCode::Delete,
                        KeyCode::Insert => input::KeyCode::Insert,
                        KeyCode::F(n) => input::KeyCode::F(n),
                        KeyCode::Null => input::KeyCode::Null,
                        KeyCode::Esc => input::KeyCode::Esc,
                        _ => input::KeyCode::Null,
                    };

                    let mut modifiers = input::KeyModifiers::empty();
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        modifiers.insert(input::KeyModifiers::SHIFT);
                    }
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        modifiers.insert(input::KeyModifiers::CONTROL);
                    }
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        modifiers.insert(input::KeyModifiers::ALT);
                    }
                    if key.modifiers.contains(KeyModifiers::SUPER) {
                        modifiers.insert(input::KeyModifiers::SUPER);
                    }
                    if key.modifiers.contains(KeyModifiers::HYPER) {
                        modifiers.insert(input::KeyModifiers::HYPER);
                    }
                    if key.modifiers.contains(KeyModifiers::META) {
                        modifiers.insert(input::KeyModifiers::META);
                    }

                    let input_key = input::KeyEvent { code, modifiers };

                    // Delegate input handling to UI widgets
                    ui::handle_input(app, input_key)?;
                }
                Event::Mouse(mouse) => {
                    let kind = match mouse.kind {
                        event::MouseEventKind::Down(btn) => {
                            Some(input::MouseEventKind::Down(map_mouse_btn(btn)))
                        }
                        event::MouseEventKind::Up(btn) => {
                            Some(input::MouseEventKind::Up(map_mouse_btn(btn)))
                        }
                        event::MouseEventKind::Drag(btn) => {
                            Some(input::MouseEventKind::Drag(map_mouse_btn(btn)))
                        }
                        event::MouseEventKind::Moved => Some(input::MouseEventKind::Moved),
                        event::MouseEventKind::ScrollDown => {
                            Some(input::MouseEventKind::ScrollDown)
                        }
                        event::MouseEventKind::ScrollUp => Some(input::MouseEventKind::ScrollUp),
                        _ => None,
                    };

                    if let Some(kind) = kind {
                        let mut modifiers = input::KeyModifiers::empty();
                        if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                            modifiers.insert(input::KeyModifiers::SHIFT);
                        }
                        if mouse.modifiers.contains(KeyModifiers::CONTROL) {
                            modifiers.insert(input::KeyModifiers::CONTROL);
                        }
                        if mouse.modifiers.contains(KeyModifiers::ALT) {
                            modifiers.insert(input::KeyModifiers::ALT);
                        }

                        let input_mouse = input::MouseEvent {
                            kind,
                            column: mouse.column,
                            row: mouse.row,
                            modifiers,
                        };

                        ui::handle_mouse_input(app, input_mouse)?;
                    }
                }
                _ => {}
            }
        }

        app.tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn map_mouse_btn(btn: crossterm::event::MouseButton) -> input::MouseButton {
    match btn {
        crossterm::event::MouseButton::Left => input::MouseButton::Left,
        crossterm::event::MouseButton::Right => input::MouseButton::Right,
        crossterm::event::MouseButton::Middle => input::MouseButton::Middle,
    }
}
