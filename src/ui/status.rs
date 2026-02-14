use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(node) = app.get_selected_node() {
        let path_str = node
            .path
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(" → ");

        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled(" Path: ", Style::default().fg(Color::Gray).bg(app.theme.bg)),
            Span::styled(
                path_str,
                Style::default()
                    .fg(app.theme.header_fg)
                    .bg(app.theme.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " | Value: ",
                Style::default().fg(Color::Gray).bg(app.theme.bg),
            ),
            Span::styled(
                node.value_preview.clone(),
                Style::default().fg(app.theme.fg).bg(app.theme.bg),
            ),
        ]))
        .style(Style::default().bg(app.theme.bg));

        frame.render_widget(paragraph, area);
    }
}
