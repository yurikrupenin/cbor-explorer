use crate::cbor_parser::CborParser;
use crate::cbor_tree::CborNode;
use crate::config::BYTES_PER_ROW;
use crate::theme::Theme;
use color_eyre::Result;
use std::path::Path;

use crate::config_store::{AppConfig, ConfigStore};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Tree,
    Hex,
}

#[derive(PartialEq)]
pub enum PopupMode {
    None,
    ThemeSelect,
    Search,
    GotoOffset,
}

pub struct App {
    pub raw_bytes: Vec<u8>,
    pub tree: Option<CborNode>,
    pub parse_error: Option<String>,
    pub focus: Focus,
    pub tree_selected: usize,
    pub tree_offset: usize, // Scroll offset for tree view
    pub hex_offset: usize,
    pub hex_selected: usize,
    pub visible_tree_height: usize,
    pub visible_hex_height: usize,
    pub file_name: String,
    pub show_help: bool,
    pub cursor_row: usize, // Track cursor row for popup positioning
    pub theme: Theme,
    pub original_theme: Option<Theme>, // Store original theme for cancelling selection
    pub theme_index: usize,            // Current selection index in dialog
    pub show_hex_integers: bool,
    pub popups: PopupMode,
    pub show_popup: bool, // Toggle detail popup visibility
    pub themes: Vec<Theme>,
    // Search state
    pub search_input: String,
    pub search_cursor_position: usize,
    pub search_error: Option<String>,
    pub last_search_query: Option<String>,
    // Config
    pub config_store: Option<ConfigStore>,
    pub config: AppConfig,
    pub should_quit: bool,
}

impl App {
    pub fn new(path: &Path) -> Result<Self> {
        let raw_bytes = std::fs::read(path)?;
        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Try to parse CBOR using custom parser
        let (tree, parse_error) = {
            let mut parser = CborParser::new(&raw_bytes);
            match parser.parse() {
                Some(parsed) => (Some(parsed.to_node(None, 0, vec![])), None),
                None => (None, Some("Failed to parse CBOR or empty file".to_string())),
            }
        };

        // Initialize themes
        let themes = vec![
            Theme::tokyo_night(),
            Theme::dracula(),
            Theme::solarized(),
            Theme::monokai(),
            Theme::nord(),
            Theme::gruvbox(),
            Theme::one_dark(),
            Theme::catppuccin(),
            Theme::github_light(),
            Theme::github_dark(),
        ];

        // Load config
        let config_store = ConfigStore::new();
        let config = if let Some(store) = &config_store {
            store.load()
        } else {
            AppConfig::default()
        };

        // Apply config
        let theme = themes
            .iter()
            .find(|t| t.name == config.theme)
            .cloned()
            .unwrap_or_else(Theme::tokyo_night);

        let show_hex_integers = config.show_hex_integers;

        Ok(App {
            raw_bytes,
            tree,
            parse_error,
            focus: Focus::Tree,
            tree_selected: 0,
            tree_offset: 0,
            hex_offset: 0,
            hex_selected: 0,
            visible_tree_height: 20,
            visible_hex_height: 20,
            file_name,
            show_help: false,
            cursor_row: 0,
            theme,
            original_theme: None,
            theme_index: 0,
            show_hex_integers,
            popups: PopupMode::None,
            show_popup: true,
            themes,
            search_input: String::new(),
            search_cursor_position: 0,
            search_error: None,
            last_search_query: None,
            config_store,
            config,
            should_quit: false,
        })
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tree => {
                // Sync Hex cursor to Tree selection
                if let Some(node) = self.get_selected_node() {
                    self.hex_selected = node.range.start;
                    self.adjust_hex_scroll();
                }
                Focus::Hex
            }
            Focus::Hex => {
                // Tree selection is already synced during hex navigation, but ensure visibility
                self.adjust_tree_scroll();
                Focus::Tree
            }
        };
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_hex_integers(&mut self) {
        self.show_hex_integers = !self.show_hex_integers;
        self.config.show_hex_integers = self.show_hex_integers;
        self.save_config();
    }

    pub fn toggle_popup(&mut self) {
        self.show_popup = !self.show_popup;
    }

    pub fn open_theme_dialog(&mut self) {
        self.original_theme = Some(self.theme.clone());
        self.popups = PopupMode::ThemeSelect;

        // Find current theme index
        if let Some(idx) = self.themes.iter().position(|t| t.name == self.theme.name) {
            self.theme_index = idx;
        } else {
            self.theme_index = 0;
        }
    }

    pub fn close_theme_dialog(&mut self) {
        self.popups = PopupMode::None;
        self.original_theme = None;
    }

    pub fn apply_theme(&mut self, index: usize) {
        if index < self.themes.len() {
            self.theme = self.themes[index].clone();
            self.theme_index = index;
        }
    }

    pub fn confirm_theme_selection(&mut self) {
        self.config.theme = self.theme.name.clone();
        self.save_config();
        self.close_theme_dialog();
    }

    pub fn save_config(&self) {
        if let Some(store) = &self.config_store {
            store.save(&self.config);
        }
    }

    pub fn cancel_theme_selection(&mut self) {
        if let Some(original) = self.original_theme.take() {
            self.theme = original;
        }
        self.popups = PopupMode::None;
    }

    pub fn move_theme_selection_up(&mut self) {
        if self.theme_index > 0 {
            self.theme_index -= 1;
            self.apply_theme(self.theme_index);
        }
    }

    pub fn move_theme_selection_down(&mut self) {
        if self.theme_index < self.themes.len() - 1 {
            self.theme_index += 1;
            self.apply_theme(self.theme_index);
        }
    }

    pub fn adjust_tree_scroll(&mut self) {
        let scrolloff = 1;

        // Ensure cursor is visible with context above
        if self.tree_selected < self.tree_offset + scrolloff {
            self.tree_offset = self.tree_selected.saturating_sub(scrolloff);
        }

        // Ensure cursor is visible with context below
        // visible_tree_height is updated in ui.rs based on inner area height, so it IS the number of visible rows.
        let visible_rows = self.visible_tree_height;

        if visible_rows > 0 {
            let max_visible_idx = self.tree_offset + visible_rows.saturating_sub(1);
            let bottom_threshold = max_visible_idx.saturating_sub(scrolloff);

            if self.tree_selected > bottom_threshold {
                self.tree_offset += self.tree_selected - bottom_threshold;
            }
        }
    }

    pub fn update_tree_selection_from_hex(&mut self) {
        if let Some(tree) = &mut self.tree {
            // Expand path to selected hex byte
            tree.expand_path_to_offset(self.hex_selected);

            let flat = tree.flatten();
            // Find the deepest visible node that contains the hex_selected offset
            let mut best_index = None;
            let mut min_len = usize::MAX;

            for (i, node) in flat.iter().enumerate() {
                if node.range.contains(&self.hex_selected) {
                    let len = node.range.len();
                    if len < min_len {
                        min_len = len;
                        best_index = Some(i);
                    }
                }
            }

            if let Some(idx) = best_index {
                self.tree_selected = idx;
            }
        }
    }

    pub fn adjust_hex_scroll(&mut self) {
        let row = self.hex_selected / BYTES_PER_ROW;
        let visible_rows = self.visible_hex_height.saturating_sub(2);
        let current_offset_row = self.hex_offset / BYTES_PER_ROW;

        if row < current_offset_row {
            self.hex_offset = row * BYTES_PER_ROW;
        } else if row >= current_offset_row + visible_rows {
            self.hex_offset = (row - visible_rows + 1) * BYTES_PER_ROW;
        }
    }

    pub fn toggle_expand(&mut self) {
        if self.focus == Focus::Tree {
            if let Some(tree) = &mut self.tree {
                if let Some(node) = tree.get_node_at_index_mut(self.tree_selected) {
                    if node.has_children() {
                        node.expanded = !node.expanded;
                    }
                }
            }
        }
    }

    pub fn expand_all(&mut self) {
        if let Some(tree) = &mut self.tree {
            tree.expand_all();
        }
    }

    pub fn collapse_all(&mut self) {
        if let Some(tree) = &mut self.tree {
            tree.collapse_all();
        }
    }

    pub fn get_selected_node(&self) -> Option<&CborNode> {
        self.tree.as_ref().and_then(|tree| {
            let flat = tree.flatten();
            flat.get(self.tree_selected).copied()
        })
    }

    pub fn get_node_at_hex_cursor(&self) -> Option<&CborNode> {
        self.tree
            .as_ref()
            .and_then(|tree| tree.get_path_to_offset(self.hex_selected).last().copied())
    }

    pub fn open_search(&mut self) {
        self.popups = PopupMode::Search;
        self.search_input.clear();
        self.search_cursor_position = 0;
        self.search_error = None;
    }

    pub fn open_goto(&mut self) {
        self.popups = PopupMode::GotoOffset;
        self.search_input.clear();
        self.search_cursor_position = 0;
        self.search_error = None;
    }

    pub fn close_popup(&mut self) {
        self.popups = PopupMode::None;
        self.search_error = None;
    }

    pub fn enter_char(&mut self, c: char) {
        if self.popups == PopupMode::Search || self.popups == PopupMode::GotoOffset {
            self.search_input.insert(self.search_cursor_position, c);
            self.search_cursor_position += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if (self.popups == PopupMode::Search || self.popups == PopupMode::GotoOffset)
            && self.search_cursor_position > 0
        {
            self.search_input.remove(self.search_cursor_position - 1);
            self.search_cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if (self.popups == PopupMode::Search || self.popups == PopupMode::GotoOffset)
            && self.search_cursor_position > 0
        {
            self.search_cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if (self.popups == PopupMode::Search || self.popups == PopupMode::GotoOffset)
            && self.search_cursor_position < self.search_input.len()
        {
            self.search_cursor_position += 1;
        }
    }

    pub fn submit_input(&mut self) {
        match self.popups {
            PopupMode::Search => self.submit_search(),
            PopupMode::GotoOffset => self.submit_goto(),
            _ => {}
        }
    }

    fn submit_search(&mut self) {
        if self.search_input.is_empty() {
            self.close_popup();
            return;
        }

        let query = self.search_input.clone();
        self.last_search_query = Some(query.clone());
        self.execute_search(&query, true); // true = forward
        if self.search_error.is_none() {
            self.close_popup();
        }
    }

    fn submit_goto(&mut self) {
        if self.search_input.is_empty() {
            self.close_popup();
            return;
        }

        // Parse input
        let input = self.search_input.trim();
        let offset = if input.starts_with("0x") || input.starts_with("0X") {
            usize::from_str_radix(&input[2..], 16)
        } else {
            input.parse::<usize>()
        };

        match offset {
            Ok(off) => {
                if off < self.raw_bytes.len() {
                    self.focus = Focus::Hex;
                    self.hex_selected = off;
                    self.adjust_hex_scroll();
                    self.update_tree_selection_from_hex();
                    self.close_popup();
                } else {
                    self.search_error = Some("Offset out of bounds".to_string());
                }
            }
            Err(_) => {
                self.search_error = Some("Invalid number".to_string());
            }
        }
    }

    pub fn find_next(&mut self) {
        if let Some(query) = self.last_search_query.clone() {
            self.execute_search(&query, true);
        }
    }

    pub fn find_previous(&mut self) {
        if let Some(query) = self.last_search_query.clone() {
            self.execute_search(&query, false);
        }
    }

    fn execute_search(&mut self, query: &str, forward: bool) {
        let current_range_start = if let Some(node) = self.get_selected_node() {
            node.range.start
        } else {
            0
        };

        let found_node_offset = if let Some(tree) = &self.tree {
            let all_nodes = tree.flatten_all();
            let query_lower = query.to_lowercase();

            let current_idx = all_nodes
                .iter()
                .position(|n| n.range.start == current_range_start)
                .unwrap_or(0);

            let mut result = None;

            if forward {
                // Search forward
                let start_idx = current_idx + 1;

                // First pass: from next to end
                for node in all_nodes.iter().skip(start_idx) {
                    if self.node_matches_query(node, &query_lower) {
                        result = Some(node.range.start);
                        break;
                    }
                }

                // Wrap around: from start to current
                if result.is_none() {
                    for node in all_nodes.iter().take(start_idx) {
                        if self.node_matches_query(node, &query_lower) {
                            result = Some(node.range.start);
                            break;
                        }
                    }
                }
            } else {
                // Backward
                let mut indices: Vec<usize> = Vec::new();
                if current_idx > 0 {
                    indices.extend((0..current_idx).rev());
                }
                indices.extend((current_idx..all_nodes.len()).rev()); // Wrap around

                for i in indices {
                    if self.node_matches_query(all_nodes[i], &query_lower) {
                        result = Some(all_nodes[i].range.start);
                        break;
                    }
                }
            }
            result
        } else {
            None
        };

        if let Some(offset) = found_node_offset {
            if let Some(tree) = &mut self.tree {
                // 1. Expand path to this node
                tree.expand_path_to_offset(offset);

                // 2. Re-flatten visible nodes to find the new tree_selected index
                let flat = tree.flatten();
                if let Some(new_idx) = flat.iter().position(|n| n.range.start == offset) {
                    self.tree_selected = new_idx;
                }
            }
            // Update UI state
            self.focus = Focus::Tree;
            self.adjust_tree_scroll();
            self.search_error = None;
        } else {
            self.search_error = Some(format!("'{}' not found", query));
        }
    }

    fn node_matches_query(&self, node: &CborNode, query: &str) -> bool {
        if let Some(key) = &node.key {
            if key.to_lowercase().contains(query) {
                return true;
            }
        }
        if node.value_preview.to_lowercase().contains(query) {
            return true;
        }
        if node.full_value.to_lowercase().contains(query) {
            return true;
        }
        false
    }
}
