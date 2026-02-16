use crate::app::{App, Focus};
use crate::cbor_tree::{CborNode, CborType};
use crate::config;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

// Handle input for the Tree View
// ...

pub fn handle_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match config::resolve_key(key) {
        config::KeyAction::Up => {
            if app.tree_selected > 0 {
                app.tree_selected -= 1;
                app.adjust_tree_scroll();
                app.update_hex_selection_from_tree();
            }
        }
        config::KeyAction::Down => {
            if let Some(tree) = &app.tree {
                let max = tree.flatten().len().saturating_sub(1);
                if app.tree_selected < max {
                    app.tree_selected += 1;
                    app.adjust_tree_scroll();
                    app.update_hex_selection_from_tree();
                }
            }
        }
        config::KeyAction::Top => {
            app.tree_selected = 0;
            app.adjust_tree_scroll();
            app.update_hex_selection_from_tree();
        }
        config::KeyAction::Bottom => {
            if let Some(tree) = &app.tree {
                app.tree_selected = tree.flatten().len().saturating_sub(1);
                app.adjust_tree_scroll();
                app.update_hex_selection_from_tree();
            }
        }
        config::KeyAction::PageUp => {
            let page_size = app.visible_tree_height.saturating_sub(2);
            app.tree_selected = app.tree_selected.saturating_sub(page_size);
            app.adjust_tree_scroll();
            app.update_hex_selection_from_tree();
        }
        config::KeyAction::PageDown => {
            if let Some(tree) = &app.tree {
                let max = tree.flatten().len().saturating_sub(1);
                let page_size = app.visible_tree_height.saturating_sub(2);
                app.tree_selected = (app.tree_selected + page_size).min(max);
                app.adjust_tree_scroll();
                app.update_hex_selection_from_tree();
            }
        }
        config::KeyAction::Expand | config::KeyAction::Enter => {
            app.toggle_expand();
        }
        config::KeyAction::ExpandAll => {
            app.expand_all();
        }
        config::KeyAction::CollapseAll => {
            app.collapse_all();
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_scroll_up(app: &mut App) {
    if app.tree_selected > 0 {
        app.tree_selected = app
            .tree_selected
            .saturating_sub(config::MOUSE_SCROLL_LINES_TREE);
        app.adjust_tree_scroll();
        app.update_hex_selection_from_tree();
    }
}

pub fn handle_scroll_down(app: &mut App) {
    if let Some(tree) = &app.tree {
        let max = tree.flatten().len();
        if max > 0 {
            let max_idx = max - 1;
            if app.tree_selected < max_idx {
                app.tree_selected =
                    (app.tree_selected + config::MOUSE_SCROLL_LINES_TREE).min(max_idx);
                app.adjust_tree_scroll();
                app.update_hex_selection_from_tree();
            }
        }
    }
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
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
