use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let shortcuts = vec![
        ("q", "Quit"),
        ("?", "Help"),
        ("Tab", "Switch View"),
        ("x", "Hex/Dec"),
        ("t", "Theme"),
        ("/", "Search"),
        (":", "Go to"),
        ("Space", "Popup"),
    ];

    let mut spans = Vec::new();
    for (key, desc) in shortcuts {
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default()
                .fg(app.theme.bg)
                .bg(app.theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {} ", desc),
            Style::default().fg(app.theme.header_fg).bg(app.theme.bg),
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(app.theme.bg));
    frame.render_widget(paragraph, area);
}
