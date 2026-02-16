use crate::cbor_tree::{CborNode, CborType};
use crate::config::BYTES_PER_ROW;
use crate::theme::Theme;
use color_eyre::Result;
use std::path::Path;

use crate::config_store::{AppConfig, ConfigStore};
use crate::scanner::{CborChunk, CborScanner, ScanMode};

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
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Score,  // Descending score
    Offset, // Ascending offset
}

pub struct App {
    pub raw_bytes: Vec<u8>,
    pub tree: Option<CborNode>,
    pub chunks: Vec<CborChunk>,
    pub sort_mode: SortMode,
    pub scan_mode: ScanMode,
    pub parse_error: Option<String>,
    pub focus: Focus,
    pub tree_selected: usize,
    pub tree_offset: usize, // Scroll offset for tree view
    pub hex_offset: usize,
    pub hex_selected: usize,
    pub visible_tree_height: usize,
    pub visible_hex_height: usize,
    pub file_name: String,
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
        // Initialize scanner and chunks
        // Default to Single mode as requested
        let scan_mode = ScanMode::Single;
        let (chunks, parse_error) = {
            let scanner = CborScanner::new(&raw_bytes);
            let chunks = scanner.scan_for_cbor_sequences(scan_mode);

            if chunks.is_empty() {
                (Vec::new(), Some("No valid CBOR data found".to_string()))
            } else {
                (chunks, None)
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

        let mut app = App {
            raw_bytes,
            tree: None, // Will be built by rebuild_tree
            chunks,
            sort_mode: SortMode::Score,
            scan_mode,
            parse_error,
            focus: Focus::Tree,
            tree_selected: 0,
            tree_offset: 0,
            hex_offset: 0,
            hex_selected: 0,
            visible_tree_height: 20,
            visible_hex_height: 20,
            file_name,
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
        };

        app.rebuild_tree();
        Ok(app)
    }

    pub fn toggle_scan_mode(&mut self) {
        self.scan_mode = match self.scan_mode {
            ScanMode::Single => ScanMode::Auto,
            ScanMode::Auto => ScanMode::Single,
        };
        self.rescan_chunks();
    }

    fn rescan_chunks(&mut self) {
        let scanner = CborScanner::new(&self.raw_bytes);
        self.chunks = scanner.scan_for_cbor_sequences(self.scan_mode);

        if self.chunks.is_empty() {
            self.parse_error = Some("No valid CBOR data found".to_string());
        } else {
            self.parse_error = None;
        }

        self.rebuild_tree();
        self.tree_selected = 0;
        self.tree_offset = 0;
    }

    pub fn toggle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Score => SortMode::Offset,
            SortMode::Offset => SortMode::Score,
        };
        self.rebuild_tree();

        // Reset selection and scrolling.
        // TODO: Would be nice to keep focus on the
        //       active item after reordering the tree
        self.tree_selected = 0;
        self.tree_offset = 0;
    }

    fn rebuild_tree(&mut self) {
        // No valid data, nothing to do.
        if self.chunks.is_empty() {
            self.tree = None;
            return;
        }

        // Sort chunks based on mode.
        match self.sort_mode {
            SortMode::Score => self.chunks.sort_by(|a, b| b.score.cmp(&a.score)),
            SortMode::Offset => self.chunks.sort_by(|a, b| a.offset.cmp(&b.offset)),
        }

        // We only have a single sequence in the file that looks like CBOR,
        // no rebuild is required.
        if self.chunks.len() == 1 && self.chunks[0].items.len() == 1 {
            self.tree = Some(self.chunks[0].items[0].to_node(None, 0, vec![]));
            return;
        }

        // Otherwise: multiple CBOR-looking sequences are found.
        //
        // Create a faux root node for display purposes;
        // attach every chunk we found to this item as children.
        let mut synthetic_root = CborNode {
            key: Some(format!(
                "Found CBOR Data (Mode: {:?}, Sorted by {:?})",
                self.scan_mode, self.sort_mode
            )),
            value_type: CborType::Map,
            value_preview: format!("{} chunks found", self.chunks.len()),
            full_value: format!("Found {} CBOR data chunks", self.chunks.len()),
            children: vec![],
            expanded: true,
            depth: 0,
            path: vec![],
            range: 0..self.raw_bytes.len(),
        };

        let children: Vec<CborNode> = self
            .chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                // Create a node for the chunk
                let chunk_range_end = chunk
                    .items
                    .last()
                    .map(|item| item.range.end)
                    .unwrap_or(chunk.offset);

                let chunk_name = format!("Chunk #{}", i + 1);

                // Faux path for top-level node in every chunk
                let chunk_path = vec![
                    crate::cbor_tree::PathSegment {
                        name: "root".to_string(),
                        depth: 0,
                    },
                    crate::cbor_tree::PathSegment {
                        name: chunk_name.clone(),
                        depth: 1,
                    },
                ];

                // Create a node for the chunk
                let mut chunk_node = CborNode {
                    key: Some(format!("Chunk #{} (Score: {})", i + 1, chunk.score)),
                    value_type: CborType::Array,
                    value_preview: format!(
                        "offset 0x{:X}, {} items",
                        chunk.offset,
                        chunk.items.len()
                    ),
                    full_value: format!(
                        "Chunk at offset 0x{:X} with {} items. Score: {}",
                        chunk.offset,
                        chunk.items.len(),
                        chunk.score
                    ),
                    children: vec![],
                    expanded: i == 0, // Expand only the first (best/first) chunk
                    depth: 1,
                    path: chunk_path.clone(),
                    range: chunk.offset..chunk_range_end,
                };

                // Attach the chunk's subitems to the node
                let items_nodes: Vec<CborNode> = chunk
                    .items
                    .iter()
                    .enumerate()
                    .map(|(j, item)| {
                        item.to_node(Some(format!("Item {}", j)), 2, chunk_path.clone())
                    })
                    .collect();

                chunk_node.children = items_nodes;
                chunk_node
            })
            .collect();

        synthetic_root.children = children;
        self.tree = Some(synthetic_root);
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
        if self.popups == PopupMode::Help {
            self.popups = PopupMode::None;
        } else {
            self.popups = PopupMode::Help;
        }
    }

    pub fn toggle_hex_integers(&mut self) {
        self.show_hex_integers = !self.show_hex_integers;
        self.config.show_hex_integers = self.show_hex_integers;
        self.save_config();
    }

    pub fn toggle_popup(&mut self) {
        self.show_popup = !self.show_popup;
    }

    pub fn save_config(&self) {
        if let Some(store) = &self.config_store {
            store.save(&self.config);
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

    pub fn update_hex_selection_from_tree(&mut self) {
        if let Some(node) = self.get_selected_node() {
            self.hex_selected = node.range.start;
            self.adjust_hex_scroll();
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
