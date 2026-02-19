use crate::input::KeyEvent;
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::config;

// ...

use crate::app::{App, PopupMode};

pub fn open(app: &mut App) {
    app.original_theme = Some(app.theme.clone());
    app.popups = PopupMode::ThemeSelect;

    // Find current theme index
    if let Some(idx) = app.themes.iter().position(|t| t.name == app.theme.name) {
        app.theme_index = idx;
    } else {
        app.theme_index = 0;
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match config::resolve_key(key) {
        config::KeyAction::Up => move_up(app),
        config::KeyAction::Down => move_down(app),
        config::KeyAction::Enter => confirm(app),
        config::KeyAction::Esc => cancel(app),
        _ => {}
    }
    Ok(())
}

fn move_up(app: &mut App) {
    if app.theme_index > 0 {
        app.theme_index -= 1;
        apply(app, app.theme_index);
    }
}

fn move_down(app: &mut App) {
    if app.theme_index < app.themes.len() - 1 {
        app.theme_index += 1;
        apply(app, app.theme_index);
    }
}

fn apply(app: &mut App, index: usize) {
    if index < app.themes.len() {
        app.theme = app.themes[index].clone();
        app.theme_index = index;
    }
}

fn confirm(app: &mut App) {
    app.config.theme = app.theme.name.clone();
    app.save_config();
    close(app);
}

fn cancel(app: &mut App) {
    if let Some(original) = app.original_theme.take() {
        app.theme = original;
    }
    close(app);
}

fn close(app: &mut App) {
    app.popups = PopupMode::None;
    app.original_theme = None;
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let width = 40; // Increased width for better readability
    let height = (app.themes.len() as u16) + 4;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let rect = Rect::new(x, y, width, height);

    frame.render_widget(Clear, rect);

    let mut lines = Vec::new();
    lines.push(Line::from("Select Theme:"));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" to confirm, "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" to cancel"),
    ]));

    for (i, theme) in app.themes.iter().enumerate() {
        let is_selected = i == app.theme_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(app.theme.selection_fg)
                .bg(app.theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.fg)
        };

        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, theme.name),
            style,
        )));
    }

    let block = Block::default()
        .title(" Themes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .bg(app.theme.popup_bg);

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, rect);
}
