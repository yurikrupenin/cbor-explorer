use crate::app::{App, Focus, PopupMode};
use crate::cbor_tree::{CborNode, CborType};
use crate::config::{self, BYTES_PER_ROW};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main layout: Content, Status Line, Shortcuts Bar
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
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

    draw_tree_view(frame, app, left_area);
    draw_hex_view(frame, app, main_chunks[1]);
    draw_status_line(frame, app, outer_chunks[1]);
    draw_shortcuts_bar(frame, app, outer_chunks[2]);

    // Draw cursor-following popup (for Tree or Hex focus)
    if !app.show_help {
        if app.focus == Focus::Tree {
            if let Some(node) = app.get_selected_node() {
                draw_cursor_popup(frame, app, node, left_area, true);
            }
        } else if app.focus == Focus::Hex {
            if let Some(node) = app.get_node_at_hex_cursor() {
                draw_cursor_popup(frame, app, node, main_chunks[1], false);
            }
        }
    }

    // Draw overlays
    if app.show_help {
        draw_help_overlay(frame, app, size);
    } else if app.popups == PopupMode::ThemeSelect {
        draw_theme_dialog(frame, app, size);
    } else if app.popups == PopupMode::Search || app.popups == PopupMode::GotoOffset {
        draw_input_dialog(frame, app, size);
    }
}

fn draw_tree_view(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::Tree;
    let border_style = if is_focused {
        Style::default().fg(app.theme.border_focused)
    } else {
        Style::default().fg(app.theme.border_unfocused)
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" CBOR Tree - {} ", app.file_name),
            Style::default()
                .fg(app.theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style)
        .bg(app.theme.bg);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(error) = &app.parse_error {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(app.theme.byte_colors.non_ascii))
            .wrap(Wrap { trim: true });
        frame.render_widget(error_text, inner);
        return;
    }

    if let Some(tree) = &app.tree {
        let flat_nodes = tree.flatten();
        let visible_height = inner.height as usize;

        // Use tree_offset from App state
        let scroll_offset = app.tree_offset;

        // Get the selected node's path/depth for highlighting ancestors
        let selected_path: Vec<String> = flat_nodes
            .get(app.tree_selected)
            .map(|n| n.path.iter().map(|p| p.name.clone()).collect())
            .unwrap_or_default();

        let mut lines: Vec<Line> = Vec::new();

        for (i, node) in flat_nodes
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
        {
            let is_selected = i == app.tree_selected;

            // Track cursor row for popup positioning
            if is_selected {
                app.cursor_row = i - scroll_offset;
            }

            // Check if this node is an ancestor of the selected node
            let is_ancestor = is_focused && !is_selected && is_node_ancestor(node, &selected_path);

            lines.push(draw_tree_node(
                node,
                app,
                is_selected,
                is_focused,
                is_ancestor,
            ));
        }

        let paragraph = Paragraph::new(lines).style(Style::default().bg(app.theme.bg));
        frame.render_widget(paragraph, inner);
    }
}

fn draw_tree_node(
    node: &CborNode,
    app: &App,
    is_selected: bool,
    is_focused: bool,
    is_ancestor: bool,
) -> Line<'static> {
    let indent = "  ".repeat(node.depth);

    let expand_icon = if node.has_children() {
        if node.expanded {
            "▼ "
        } else {
            "▶ "
        }
    } else {
        "  "
    };

    let key_part = node
        .key
        .as_ref()
        .map(|k| format!("{}: ", k))
        .unwrap_or_default();

    // Determine highlighting color based on depth relationship
    let (_line_style, key_color) = if is_selected {
        // Selected item (Active or inactive focus)
        let color = app.theme.get_depth_color(node.depth);
        let style = Style::default()
            .bg(app.theme.selection_bg)
            .fg(app.theme.selection_fg)
            .add_modifier(Modifier::BOLD);

        // Maybe dim it slightly if not focused? User didn't ask for it, said "highlighted".
        // But distinct focus is good. Let's keep it same for now as requested.
        (style, color)
    } else if is_ancestor && is_focused {
        // Ancestor
        let color = app.theme.get_depth_color(node.depth);
        (
            Style::default().bg(app.theme.selection_bg), // Slightly different shade?
            color,
        )
    } else {
        (Style::default().fg(app.theme.fg), app.theme.fg)
    };

    let type_color = if is_selected || is_ancestor {
        key_color
    } else {
        app.theme.fg
    };

    let value_display = if app.show_hex_integers
        && (node.value_type == CborType::Integer || node.value_type == CborType::Tag)
    {
        // Try to parse integer and show as hex
        // This is a rough heuristic, ideally we'd store the numeric value properly
        if let Ok(val) = node.full_value.parse::<i128>() {
            format!("0x{:x}", val)
        } else if let Ok(val) = node.full_value.parse::<u128>() {
            format!("0x{:x}", val)
        } else {
            node.value_preview.clone()
        }
    } else {
        node.value_preview.clone()
    };

    let spans = vec![
        Span::raw(indent),
        Span::styled(expand_icon, Style::default().fg(app.theme.header_fg)),
        Span::styled(key_part, Style::default().fg(key_color)),
        Span::styled(
            format!("[{}] ", node.value_type),
            Style::default().fg(type_color),
        ),
        Span::styled(
            value_display,
            Style::default().fg(if is_selected || is_ancestor {
                app.theme.fg
            } else {
                Color::DarkGray
            }),
        ),
    ];

    Line::from(spans).style(_line_style)
}

fn is_node_ancestor(node: &CborNode, selected_path: &[String]) -> bool {
    if node.path.len() >= selected_path.len() {
        return false;
    }
    // Check if node's path is a prefix of the selected path
    node.path
        .iter()
        .zip(selected_path.iter())
        .all(|(a, b)| a.name == *b)
}

fn draw_hex_view(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Hex;
    let border_style = if is_focused {
        Style::default().fg(app.theme.border_focused)
    } else {
        Style::default().fg(app.theme.border_unfocused)
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" Hex View - {} bytes ", app.raw_bytes.len()),
            Style::default()
                .fg(app.theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style)
        .bg(app.theme.bg);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_rows = inner.height as usize;
    let start_row = app.hex_offset / BYTES_PER_ROW;

    // Determine the highlight path based on focus
    let highlight_path: Vec<&CborNode> = if let Some(tree) = &app.tree {
        match app.focus {
            Focus::Tree => {
                if let Some(node) = app.get_selected_node() {
                    tree.get_path_to_offset(node.range.start)
                } else {
                    Vec::new()
                }
            }
            Focus::Hex => tree.get_path_to_offset(app.hex_selected),
        }
    } else {
        Vec::new()
    };

    // Reverse highlight path to search from deepest to root
    let highlight_priorities: Vec<&CborNode> = highlight_path.into_iter().rev().collect();

    let mut lines: Vec<Line> = Vec::new();

    for row in start_row..(start_row + visible_rows) {
        let offset = row * BYTES_PER_ROW;
        if offset >= app.raw_bytes.len() {
            break;
        }

        lines.push(draw_hex_row(offset, app, &highlight_priorities, is_focused));
    }

    let paragraph = Paragraph::new(lines).style(Style::default().bg(app.theme.bg));
    frame.render_widget(paragraph, inner);
}

fn draw_hex_row(
    offset: usize,
    app: &App,
    highlight_priorities: &[&CborNode],
    is_focused: bool,
) -> Line<'static> {
    let mut spans: Vec<Span> = vec![Span::styled(
        format!("{:08x}  ", offset),
        Style::default().fg(Color::DarkGray),
    )];

    // Hex bytes
    for col in 0..BYTES_PER_ROW {
        let byte_idx = offset + col;
        if byte_idx < app.raw_bytes.len() {
            let byte = app.raw_bytes[byte_idx];
            let byte_type = config::get_byte_type(byte);

            // Default color from byte type
            let base_color = match byte_type {
                config::ByteType::Null => app.theme.byte_colors.null,
                config::ByteType::AsciiPrintable => app.theme.byte_colors.ascii_printable,
                config::ByteType::AsciiWhitespace => app.theme.byte_colors.ascii_whitespace,
                config::ByteType::AsciiOther => app.theme.byte_colors.ascii_other,
                config::ByteType::NonAscii => app.theme.byte_colors.non_ascii,
            };

            let mut bg_color = app.theme.bg;
            let mut fg_color = base_color;

            // Apply Tree Selection Highlighting
            if let Some((_idx, node)) = highlight_priorities
                .iter()
                .enumerate()
                .find(|(_, n)| n.range.contains(&byte_idx))
            {
                // For Hex bytes, use depth color for FOREGROUND
                fg_color = app.theme.get_depth_color(node.depth);
            }

            // Apply Cursor Highlighting
            if is_focused && byte_idx == app.hex_selected {
                bg_color = app.theme.selection_fg; // Invert?
                fg_color = app.theme.selection_bg;
            }

            let style = Style::default().fg(fg_color).bg(bg_color);

            spans.push(Span::styled(format!("{:02x}", byte), style));
            spans.push(Span::raw(" "));
        } else {
            spans.push(Span::raw("   "));
        }

        if col == 7 {
            spans.push(Span::raw(" "));
        }
    }

    spans.push(Span::raw(" │ "));

    // ASCII representation
    for col in 0..BYTES_PER_ROW {
        let byte_idx = offset + col;
        if byte_idx < app.raw_bytes.len() {
            let byte = app.raw_bytes[byte_idx];
            let ch = if byte.is_ascii_graphic() || byte == b' ' {
                byte as char
            } else {
                '.'
            };

            let byte_type = config::get_byte_type(byte);
            let base_color = match byte_type {
                config::ByteType::Null => app.theme.byte_colors.null,
                config::ByteType::AsciiPrintable => app.theme.byte_colors.ascii_printable,
                config::ByteType::AsciiWhitespace => app.theme.byte_colors.ascii_whitespace,
                config::ByteType::AsciiOther => app.theme.byte_colors.ascii_other,
                config::ByteType::NonAscii => app.theme.byte_colors.non_ascii,
            };

            let mut bg_color = app.theme.bg;
            let mut fg_color = base_color;

            if let Some((_idx, node)) = highlight_priorities
                .iter()
                .enumerate()
                .find(|(_, n)| n.range.contains(&byte_idx))
            {
                // For ASCII text, use depth color for BACKGROUND (as per user request previously)
                bg_color = app.theme.get_depth_color(node.depth);
                fg_color = Color::Black; // Contrast against bright depth colors
            }

            if is_focused && byte_idx == app.hex_selected {
                bg_color = app.theme.selection_fg;
                fg_color = app.theme.selection_bg;
            }

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(fg_color).bg(bg_color),
            ));
        }
    }

    Line::from(spans)
}

fn draw_shortcuts_bar(frame: &mut Frame, app: &App, area: Rect) {
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

fn draw_status_line(frame: &mut Frame, app: &App, area: Rect) {
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

fn draw_cursor_popup(
    frame: &mut Frame,
    app: &App,
    node: &CborNode,
    parent_area: Rect,
    is_tree_focus: bool,
) {
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

        let start_row = app.hex_offset / BYTES_PER_ROW;
        let current_row = app.hex_selected / BYTES_PER_ROW;
        let relative_row = current_row.saturating_sub(start_row);
        let col = app.hex_selected % BYTES_PER_ROW;

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

fn draw_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let help_width = 50;
    let help_height = 22; // Increased size for new shortcuts
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

fn draw_theme_dialog(frame: &mut Frame, app: &App, area: Rect) {
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

fn draw_input_dialog(frame: &mut Frame, app: &App, area: Rect) {
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
    // but `Paragraph` is fine. We can just modify the text or use `frame.set_cursor`.

    // Let's use frame.set_cursor for the real cursor effect
    let cursor_x = rect.x + 1 + prompt.len() as u16 + app.search_cursor_position as u16;
    let cursor_y = rect.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));



    let p = Paragraph::new(Line::from(text)).block(block);
    frame.render_widget(p, rect);
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
