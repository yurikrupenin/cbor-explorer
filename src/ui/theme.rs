use crate::app::App;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_theme_selection_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_theme_selection_down(),
        KeyCode::Enter => app.confirm_theme_selection(),
        KeyCode::Esc => app.cancel_theme_selection(),
        // Still allow numeric selection for quick access
        KeyCode::Char(c) => {
            if let Some(digit) = c.to_digit(10) {
                if digit > 0 {
                    app.apply_theme((digit - 1) as usize);
                }
            }
        }
        _ => {}
    }
    Ok(())
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

        lines.push(Line::from(vec![Span::styled(
            format!("{}{}. {}", prefix, i + 1, theme.name),
            style,
        )]));
    }

    let block = Block::default()
        .title(" Themes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .bg(app.theme.popup_bg);

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, rect);
}
