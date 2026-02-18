pub mod details;
pub mod help;
pub mod hex;
pub mod notification;
pub mod search;
pub mod shortcuts;
pub mod status;
pub mod theme;
pub mod tree;

use crate::app::{App, Focus, PopupMode};
use crate::config;
use crate::zoom::Zoomable;
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
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
            Constraint::Percentage(100 - config::LAYOUT_SPLIT_PERCENT),
        ])
        .split(outer_chunks[0]);

    // Left side: Tree view
    let left_area = main_chunks[0];
    app.visible_tree_height =
        (left_area.height as usize).saturating_sub(config::BORDER_HEIGHT_ADJUSTMENT);
    app.visible_hex_height =
        (main_chunks[1].height as usize).saturating_sub(config::BORDER_HEIGHT_ADJUSTMENT);

    // Report back rectangle sizes back to app after drawing;
    // used to handle mouse input
    app.tree_area = left_area;
    app.hex_area = main_chunks[1];

    tree::draw(frame, app, left_area);
    hex::draw(frame, app, main_chunks[1]);
    status::draw(frame, app, outer_chunks[1]);
    shortcuts::draw(frame, app, outer_chunks[2]);

    // Draw cursor-following popup (for Tree or Hex focus)
    if app.popups == PopupMode::None {
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
    if app.popups == PopupMode::Help {
        help::draw(frame, app, size);
    } else if app.popups == PopupMode::ThemeSelect {
        theme::draw(frame, app, size);
    } else if app.popups == PopupMode::Search || app.popups == PopupMode::GotoOffset {
        search::draw(frame, app, size);
    }

    // Draw notification
    if let Some(notification) = &app.notification {
        notification::draw(frame, notification, outer_chunks[0], &app.theme);
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    // Priority: Help Overlay -> Popups -> Main View

    // Check pop-ups first
    match app.popups {
        // Already in a pop-up mode where a pop-up accepts input?
        // Let the pop-up handle it.
        PopupMode::ThemeSelect => theme::handle_input(app, key)?,
        PopupMode::Help => help::handle_input(app, key)?,
        PopupMode::Search | PopupMode::GotoOffset => search::handle_input(app, key)?,

        // No active pop-up, check global shortcuts first
        PopupMode::None => {
            match config::resolve_key(key) {
                config::KeyAction::Quit => {
                    app.should_quit = true;
                    return Ok(());
                }
                config::KeyAction::SwitchFocus => {
                    app.toggle_focus();
                    return Ok(());
                }
                config::KeyAction::Help => {
                    app.toggle_help();
                    return Ok(());
                }
                config::KeyAction::Sort => {
                    app.toggle_sort();
                    return Ok(());
                }
                config::KeyAction::Mode => {
                    app.toggle_scan_mode();
                    return Ok(());
                }
                config::KeyAction::Zoom => {
                    app.zoom_toggle();
                    return Ok(());
                }
                config::KeyAction::ToggleHexInt => {
                    app.toggle_hex_integers();
                    return Ok(());
                }
                config::KeyAction::ThemeSelect => {
                    theme::open(app);
                    return Ok(());
                }
                config::KeyAction::Search => {
                    app.open_search();
                    return Ok(());
                }
                config::KeyAction::Goto => {
                    app.open_goto();
                    return Ok(());
                }
                config::KeyAction::Next => {
                    app.find_next();
                    return Ok(());
                }
                config::KeyAction::Prev => {
                    app.find_previous();
                    return Ok(());
                }
                config::KeyAction::TogglePopup => {
                    app.toggle_popup();
                    return Ok(());
                }
                _ => {}
            }

            // Input is not handled by a pop-up or global shortcuts:
            // Dispatch to the currently focused widget.
            match app.focus {
                Focus::Tree => tree::handle_input(app, key)?,
                Focus::Hex => hex::handle_input(app, key)?,
            }
        }
    }
    Ok(())
}
pub fn handle_mouse_input(app: &mut App, mouse: MouseEvent) -> Result<()> {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let x = mouse.column;
            let y = mouse.row;

            if mouse_is_inside(app.tree_area, x, y) {
                app.focus = Focus::Tree;
            } else if mouse_is_inside(app.hex_area, x, y) {
                app.focus = Focus::Hex;
            }
        }
        MouseEventKind::ScrollDown => {
            let x = mouse.column;
            let y = mouse.row;

            if mouse_is_inside(app.tree_area, x, y) {
                tree::handle_scroll_down(app);
            } else if mouse_is_inside(app.hex_area, x, y) {
                hex::handle_scroll_down(app);
            }
        }
        MouseEventKind::ScrollUp => {
            let x = mouse.column;
            let y = mouse.row;

            if mouse_is_inside(app.tree_area, x, y) {
                tree::handle_scroll_up(app);
            } else if mouse_is_inside(app.hex_area, x, y) {
                hex::handle_scroll_up(app);
            }
        }
        _ => {}
    }
    Ok(())
}

fn mouse_is_inside(area: ratatui::layout::Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}
