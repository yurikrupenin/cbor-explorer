use crate::app::{App, PopupMode};
use crate::input::{KeyCode, KeyEvent};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.close_popup(),
        KeyCode::Enter => app.submit_input(),
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => app.move_cursor_right(),
        KeyCode::Char(c) => app.enter_char(c),
        _ => {}
    }
    Ok(())
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let width = 60;
    let height = 3;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let rect = Rect::new(x, y, width, height);

    frame.render_widget(Clear, rect);

    let (title, prompt) = match app.popups {
        PopupMode::Search => (" Search ", "/"),
        PopupMode::GotoOffset => (" Go to Offset ", ":"),
        _ => return,
    };

    let border_color = if app.search_error.is_some() {
        Color::Red
    } else {
        app.theme.border_focused
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(app.theme.popup_bg);

    let text = vec![
        Span::styled(
            format!("{} ", prompt),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&app.search_input),
    ];

    // Show cursor
    // Since we are not in main loop here, we just render the text.
    // We can simulate cursor by adding a block cursor at the position?
    // Ratatui doesn't have a built-in text field widget with cursor management easily exposed in a single call,
    // but `Paragraph` is fine.

    // Let's use frame.set_cursor_position for the real cursor effect
    let cursor_x = rect.x + 1 + prompt.len() as u16 + app.search_cursor_position as u16;
    let cursor_y = rect.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));

    let p = Paragraph::new(Line::from(text)).block(block);
    frame.render_widget(p, rect);
}
