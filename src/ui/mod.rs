pub mod details;
pub mod help;
pub mod hex;
pub mod input;
pub mod shortcuts;
pub mod status;
pub mod theme;
pub mod tree;

use crate::app::{App, Focus, PopupMode};
use crate::config;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main layout: Content, Status Line, Shortcuts Bar
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Content
            Constraint::Length(1), // Status Line
            Constraint::Length(1), // Shortcuts Bar
        ])
        .split(size);

    // Content layout: Horizontal split for Tree and Hex views
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(config::LAYOUT_SPLIT_PERCENT),
            Constraint::Percentage(config::LAYOUT_SPLIT_PERCENT),
        ])
        .split(outer_chunks[0]);

    // Left side: Tree view
    let left_area = main_chunks[0];
    app.visible_tree_height =
        (left_area.height as usize).saturating_sub(config::BORDER_HEIGHT_ADJUSTMENT);
    app.visible_hex_height =
        (main_chunks[1].height as usize).saturating_sub(config::BORDER_HEIGHT_ADJUSTMENT);

    tree::draw(frame, app, left_area);
    hex::draw(frame, app, main_chunks[1]);
    status::draw(frame, app, outer_chunks[1]);
    shortcuts::draw(frame, app, outer_chunks[2]);

    // Draw cursor-following popup (for Tree or Hex focus)
    if !app.show_help {
        if app.focus == Focus::Tree {
            if let Some(node) = app.get_selected_node() {
                details::draw(frame, app, node, left_area, true);
            }
        } else if app.focus == Focus::Hex {
            if let Some(node) = app.get_node_at_hex_cursor() {
                details::draw(frame, app, node, main_chunks[1], false);
            }
        }
    }

    // Draw overlays
    if app.show_help {
        help::draw(frame, app, size);
    } else if app.popups == PopupMode::ThemeSelect {
        theme::draw(frame, app, size);
    } else if app.popups == PopupMode::Search || app.popups == PopupMode::GotoOffset {
        input::draw(frame, app, size);
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    // Priority: Help Overlay -> Popups -> Main View

    if app.show_help {
        return help::handle_input(app, key);
    }

    match app.popups {
        PopupMode::ThemeSelect => theme::handle_input(app, key)?,
        PopupMode::Search | PopupMode::GotoOffset => input::handle_input(app, key)?,
        PopupMode::None => {
            // Check global shortcuts first
            match key.code {
                KeyCode::Char(config::keys::QUIT) => {
                    app.should_quit = true;
                    return Ok(());
                }
                KeyCode::Tab => {
                    app.toggle_focus();
                    return Ok(());
                }
                KeyCode::Char(config::keys::HELP) => {
                    app.toggle_help();
                    return Ok(());
                }
                KeyCode::Char(config::keys::TOGGLE_HEX_INT) => {
                    app.toggle_hex_integers();
                    return Ok(());
                }
                KeyCode::Char(config::keys::THEME_SELECT) => {
                    app.open_theme_dialog();
                    return Ok(());
                }
                KeyCode::Char('/') => {
                    app.open_search();
                    return Ok(());
                }
                KeyCode::Char(':') => {
                    app.open_goto();
                    return Ok(());
                }
                KeyCode::Char('n') => {
                    app.find_next();
                    return Ok(());
                }
                KeyCode::Char('N') => {
                    app.find_previous();
                    return Ok(());
                }
                KeyCode::Char(' ') => {
                    app.toggle_popup();
                    return Ok(());
                }
                _ => {}
            }

            // Dispatch to focused widget.
            // Note that keys pressed that matched above might still fall through if we didn't return Ok(()).
            // But we do return Ok(()).

            // If we didn't match a global key, check focused widget
            match app.focus {
                Focus::Tree => tree::handle_input(app, key)?,
                Focus::Hex => hex::handle_input(app, key)?,
            }
        }
    }
    Ok(())
}
