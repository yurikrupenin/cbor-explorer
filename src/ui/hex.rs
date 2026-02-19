use crate::app::{App, Focus};
use crate::cbor_tree::CborNode;
use crate::config::{self, BYTES_PER_ROW};
use crate::input::KeyEvent;
use crate::util;
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

// ...

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match config::resolve_key(key) {
        config::KeyAction::Up => {
            if app.hex_selected >= BYTES_PER_ROW {
                app.hex_selected -= BYTES_PER_ROW;
            } else {
                app.hex_selected = 0;
            }
            app.adjust_hex_scroll();
            app.update_tree_selection_from_hex();
            app.adjust_tree_scroll();
        }
        config::KeyAction::Down => {
            let max = app.raw_bytes.len().saturating_sub(1);
            if app.hex_selected + BYTES_PER_ROW <= max {
                app.hex_selected += BYTES_PER_ROW;
            } else {
                app.hex_selected = max;
            }
            app.adjust_hex_scroll();
            app.update_tree_selection_from_hex();
            app.adjust_tree_scroll();
        }
        config::KeyAction::Left => {
            if app.hex_selected > 0 {
                app.hex_selected -= 1;
                app.adjust_hex_scroll();
                app.update_tree_selection_from_hex();
                app.adjust_tree_scroll();
            }
        }
        config::KeyAction::Right => {
            if app.hex_selected < app.raw_bytes.len().saturating_sub(1) {
                app.hex_selected += 1;
                app.adjust_hex_scroll();
                app.update_tree_selection_from_hex();
                app.adjust_tree_scroll();
            }
        }
        config::KeyAction::Top => {
            app.hex_selected = 0;
            app.hex_offset = 0;
            app.update_tree_selection_from_hex();
            app.adjust_tree_scroll();
        }
        config::KeyAction::Bottom => {
            app.hex_selected = app.raw_bytes.len().saturating_sub(1);
            app.hex_selected = app.raw_bytes.len().saturating_sub(1);
            app.adjust_hex_scroll();
            app.update_tree_selection_from_hex();
            app.adjust_tree_scroll();
        }
        config::KeyAction::PageUp => {
            let page_size = app.visible_hex_height.saturating_sub(2) * BYTES_PER_ROW;
            app.hex_selected = app.hex_selected.saturating_sub(page_size);
            app.hex_selected = app.hex_selected.saturating_sub(page_size);
            app.adjust_hex_scroll();
            app.update_tree_selection_from_hex();
            app.adjust_tree_scroll();
        }
        config::KeyAction::PageDown => {
            let max = app.raw_bytes.len().saturating_sub(1);
            let page_size = app.visible_hex_height.saturating_sub(2) * BYTES_PER_ROW;
            app.hex_selected = (app.hex_selected + page_size).min(max);
            app.hex_selected = (app.hex_selected + page_size).min(max);
            app.adjust_hex_scroll();
            app.update_tree_selection_from_hex();
            app.adjust_tree_scroll();
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_scroll_up(app: &mut App) {
    let step = BYTES_PER_ROW * config::MOUSE_SCROLL_LINES_HEX;
    if app.hex_selected >= step {
        app.hex_selected -= step;
    } else {
        app.hex_selected = 0;
    }
    app.adjust_hex_scroll();
    app.update_tree_selection_from_hex();
    app.adjust_tree_scroll();
}

pub fn handle_scroll_down(app: &mut App) {
    let max = app.raw_bytes.len().saturating_sub(1);
    let step = BYTES_PER_ROW * config::MOUSE_SCROLL_LINES_HEX;
    if app.hex_selected + step <= max {
        app.hex_selected += step;
    } else {
        app.hex_selected = max;
    }
    app.adjust_hex_scroll();
    app.update_tree_selection_from_hex();
    app.adjust_tree_scroll();
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Hex;
    let title = if app.is_zoomed {
        if let Some(chunk) = app.chunks.first() {
            format!(
                " Hex View (Zoomed at 0x{:X}) - {} bytes ",
                chunk.offset,
                app.raw_bytes.len()
            )
        } else {
            " Hex View (Zoomed) - No data found ".to_string()
        }
    } else {
        format!(" Hex View - {} bytes ", app.raw_bytes.len())
    };

    let zoom_border_color = if app.is_zoomed {
        app.theme.accent_color
    } else {
        app.theme.border_focused
    };

    let border_style = if is_focused {
        if app.is_zoomed {
            Style::default()
                .fg(zoom_border_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.border_focused)
        }
    } else if app.is_zoomed {
        Style::default().fg(zoom_border_color)
    } else {
        Style::default().fg(app.theme.border_unfocused)
    };

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(if app.is_zoomed {
                    zoom_border_color
                } else {
                    app.theme.header_fg
                })
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
            let byte_type = util::get_byte_type(byte);

            let is_in_zoomed_range = is_byte_in_zoomed_range(app, byte_idx);

            // Default color from byte type
            let base_color = if !is_in_zoomed_range {
                app.theme.byte_colors.null // Dimmed color for outside bytes
            } else {
                match byte_type {
                    util::ByteType::Null => app.theme.byte_colors.null,
                    util::ByteType::AsciiPrintable => app.theme.byte_colors.ascii_printable,
                    util::ByteType::AsciiWhitespace => app.theme.byte_colors.ascii_whitespace,
                    util::ByteType::AsciiOther => app.theme.byte_colors.ascii_other,
                    util::ByteType::NonAscii => app.theme.byte_colors.non_ascii,
                }
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
                bg_color = app.theme.selection_fg;
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

            let byte_type = util::get_byte_type(byte);
            let is_in_zoomed_range = is_byte_in_zoomed_range(app, byte_idx);
            let base_color = if !is_in_zoomed_range {
                app.theme.byte_colors.null
            } else {
                match byte_type {
                    util::ByteType::Null => app.theme.byte_colors.null,
                    util::ByteType::AsciiPrintable => app.theme.byte_colors.ascii_printable,
                    util::ByteType::AsciiWhitespace => app.theme.byte_colors.ascii_whitespace,
                    util::ByteType::AsciiOther => app.theme.byte_colors.ascii_other,
                    util::ByteType::NonAscii => app.theme.byte_colors.non_ascii,
                }
            };

            let mut bg_color = app.theme.bg;
            let mut fg_color = base_color;

            if let Some((_idx, node)) = highlight_priorities
                .iter()
                .enumerate()
                .find(|(_, n)| n.range.contains(&byte_idx))
            {
                // For ASCII text, use depth color for BACKGROUND
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

fn is_byte_in_zoomed_range(app: &App, byte_idx: usize) -> bool {
    if !app.is_zoomed {
        return true;
    }

    if let Some(tree) = &app.tree {
        tree.range.contains(&byte_idx)
    } else {
        false
    }
}
