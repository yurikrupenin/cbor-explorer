use crate::app::App;
use crate::cbor_tree::{CborNode, CborType};
use crate::config;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App, node: &CborNode, parent_area: Rect, is_tree_focus: bool) {
    if !app.show_popup {
        return;
    }

    // Calculate position
    let (popup_x, popup_y, popup_width, popup_height) =
        calculate_popup_position(frame, app, node, parent_area, is_tree_focus);
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area and draw popup
    frame.render_widget(Clear, popup_area);

    // Build breadcrumb line with depth-based colors
    let mut breadcrumb_spans: Vec<Span> = Vec::new();

    for (i, segment) in node.path.iter().enumerate() {
        let color = app.theme.get_depth_color(segment.depth);

        if i > 0 {
            breadcrumb_spans.push(Span::styled(" → ", Style::default().fg(Color::DarkGray)));
        }
        breadcrumb_spans.push(Span::styled(
            &segment.name,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    let value_display = if app.show_hex_integers
        && (node.value_type == CborType::Integer || node.value_type == CborType::Tag)
    {
        if let Ok(val) = node.full_value.parse::<i128>() {
            format!("0x{:x}", val)
        } else if let Ok(val) = node.full_value.parse::<u128>() {
            format!("0x{:x}", val)
        } else {
            node.full_value.clone()
        }
    } else if node.value_type == CborType::TextString {
        format!("\"{}\"", node.full_value)
    } else {
        node.full_value.clone()
    };

    // Build content lines
    let mut content_lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "Path:",
            Style::default()
                .fg(app.theme.header_fg)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(breadcrumb_spans),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Gray)),
            Span::styled(
                node.value_type.to_string(),
                Style::default().fg(app.theme.get_depth_color(node.depth)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Value: ", Style::default().fg(Color::Gray)),
            Span::styled(
                truncate_string(&value_display, (popup_width as usize).saturating_sub(10)),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Range: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!(
                    "{}..{} ({} bytes)",
                    node.range.start,
                    node.range.end,
                    node.range.len()
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    if node.has_children() {
        content_lines.push(Line::from(vec![
            Span::styled("Children: ", Style::default().fg(Color::Gray)),
            Span::styled(
                node.children.len().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]));
    }

    let popup_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.popup_border))
        .bg(app.theme.popup_bg);

    let popup_paragraph = Paragraph::new(content_lines)
        .block(popup_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(popup_paragraph, popup_area);
}

fn calculate_popup_position(
    frame: &Frame,
    app: &App,
    node: &CborNode,
    parent_area: Rect,
    is_tree_focus: bool,
) -> (u16, u16, u16, u16) {
    let popup_width = config::POPUP_MAX_WIDTH.min(frame.area().width.saturating_sub(4));
    let popup_height = (node.path.len() as u16 + 8).min(config::POPUP_MAX_HEIGHT);

    if is_tree_focus {
        // Tree Focus: Position to the right of tree area, vertically aligned with row
        let cursor_screen_row = parent_area.y + 1 + app.cursor_row as u16;

        // Add offset to avoid covering the line completely
        let offset_y = cursor_screen_row + 2; // +1 for next line, +1 for gap

        let y = if offset_y + popup_height > frame.area().height {
            frame.area().height.saturating_sub(popup_height + 1)
        } else {
            offset_y
        };

        let x = parent_area.x + parent_area.width.saturating_sub(popup_width + 2);
        (x, y, popup_width, popup_height)
    } else {
        // Hex Focus:
        let block_inner_y = parent_area.y + 1;
        let block_inner_x = parent_area.x + 1;

        let start_row = app.hex_offset / config::BYTES_PER_ROW;
        let current_row = app.hex_selected / config::BYTES_PER_ROW;
        let relative_row = current_row.saturating_sub(start_row);
        let col = app.hex_selected % config::BYTES_PER_ROW;

        let char_x = 11 + col * 3;

        let screen_x = block_inner_x + char_x as u16;
        let screen_y = block_inner_y + relative_row as u16;

        // Add vertical offset
        let y = if screen_y + 2 + popup_height < frame.area().height {
            screen_y + 2
        } else {
            screen_y.saturating_sub(popup_height)
        };

        let x = if screen_x + popup_width > frame.area().width {
            frame.area().width.saturating_sub(popup_width + 1)
        } else {
            screen_x
        };

        (x, y, popup_width, popup_height)
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
