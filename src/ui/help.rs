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
        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
            app.toggle_help();
        }
        _ => {}
    }
    Ok(())
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let help_width = 50;
    let help_height = 22;
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;

    let help_area = Rect::new(x, y, help_width, help_height);

    frame.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(app.theme.header_fg),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab      ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch focus (Tree/Hex)"),
        ]),
        Line::from(vec![
            Span::styled("↑/k ↓/j  ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate up/down"),
        ]),
        Line::from(vec![
            Span::styled("←/h →/l  ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate left/right (Hex)"),
        ]),
        Line::from(vec![
            Span::styled("Enter/Space", Style::default().fg(Color::Yellow)),
            Span::raw("Expand/collapse node"),
        ]),
        Line::from(vec![
            Span::styled("x        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle Hex/Dec Integers"),
        ]),
        Line::from(vec![
            Span::styled("t        ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch Theme"),
        ]),
        Line::from(vec![
            Span::styled("e        ", Style::default().fg(Color::Yellow)),
            Span::raw("Expand all nodes"),
        ]),
        Line::from(vec![
            Span::styled("c        ", Style::default().fg(Color::Yellow)),
            Span::raw("Collapse all nodes"),
        ]),
        Line::from(vec![
            Span::styled("g/Home   ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to start"),
        ]),
        Line::from(vec![
            Span::styled("G/End    ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to end"),
        ]),
        Line::from(vec![
            Span::styled("PgUp/Dn  ", Style::default().fg(Color::Yellow)),
            Span::raw("Page up/down"),
        ]),
        Line::from(vec![
            Span::styled("?        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("q        ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help_block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused))
        .bg(app.theme.popup_bg);

    let help_paragraph = Paragraph::new(help_text).block(help_block);
    frame.render_widget(help_paragraph, help_area);
}
