use crate::cbor_tree::CborNode;
use crate::cbor_parser::CborParser;
use crate::config::BYTES_PER_ROW;
use crate::theme::Theme;
use color_eyre::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Tree,
    Hex,
}

#[derive(PartialEq)]
pub enum PopupMode {
    None,
    ThemeSelect,
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
    pub cursor_row: usize,  // Track cursor row for popup positioning
    pub theme: Theme,
    pub original_theme: Option<Theme>, // Store original theme for cancelling selection
    pub theme_index: usize, // Current selection index in dialog
    pub show_hex_integers: bool,
    pub popups: PopupMode,
    pub show_popup: bool, // Toggle detail popup visibility
    pub themes: Vec<Theme>,
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
                Some(parsed) => {
                     (Some(parsed.to_node(None, 0, vec![])), None)
                },
                None => (None, Some("Failed to parse CBOR or empty file".to_string())),
            }
        };

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
            theme: Theme::default(),
            original_theme: None,
            theme_index: 0,
            show_hex_integers: false, // Default to decimal
            popups: PopupMode::None,
            show_popup: true,
            themes: vec![
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
            ],
        })
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tree => Focus::Hex,
            Focus::Hex => Focus::Tree,
        };
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_hex_integers(&mut self) {
        self.show_hex_integers = !self.show_hex_integers;
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
        self.close_theme_dialog();
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

    fn adjust_tree_scroll(&mut self) {
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

    pub fn move_up(&mut self) {
        match self.focus {
            Focus::Tree => {
                if self.tree_selected > 0 {
                    self.tree_selected -= 1;
                    self.adjust_tree_scroll();
                }
            }
            Focus::Hex => {
                if self.hex_selected >= BYTES_PER_ROW {
                    self.hex_selected -= BYTES_PER_ROW;
                } else {
                    self.hex_selected = 0;
                }
                self.adjust_hex_scroll();
                self.update_tree_selection_from_hex();
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.focus {
            Focus::Tree => {
                if let Some(tree) = &self.tree {
                    let max = tree.flatten().len().saturating_sub(1);
                    if self.tree_selected < max {
                        self.tree_selected += 1;
                        self.adjust_tree_scroll();
                    }
                }
            }
            Focus::Hex => {
                let max = self.raw_bytes.len().saturating_sub(1);
                if self.hex_selected + BYTES_PER_ROW <= max {
                    self.hex_selected += BYTES_PER_ROW;
                } else {
                    self.hex_selected = max;
                }
                self.adjust_hex_scroll();
                self.update_tree_selection_from_hex();
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.focus == Focus::Hex && self.hex_selected > 0 {
            self.hex_selected -= 1;
            self.adjust_hex_scroll();
            self.update_tree_selection_from_hex();
        }
    }

    pub fn move_right(&mut self) {
        if self.focus == Focus::Hex && self.hex_selected < self.raw_bytes.len().saturating_sub(1) {
            self.hex_selected += 1;
            self.adjust_hex_scroll();
            self.update_tree_selection_from_hex();
        }
    }

    fn update_tree_selection_from_hex(&mut self) {
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

    fn adjust_hex_scroll(&mut self) {
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

    pub fn go_to_start(&mut self) {
        match self.focus {
            Focus::Tree => {
                self.tree_selected = 0;
                self.adjust_tree_scroll();
            }
            Focus::Hex => {
                self.hex_selected = 0;
                self.hex_offset = 0;
                self.update_tree_selection_from_hex();
            }
        }
    }

    pub fn go_to_end(&mut self) {
        match self.focus {
            Focus::Tree => {
                if let Some(tree) = &self.tree {
                    self.tree_selected = tree.flatten().len().saturating_sub(1);
                    self.adjust_tree_scroll();
                }
            }
            Focus::Hex => {
                self.hex_selected = self.raw_bytes.len().saturating_sub(1);
                self.adjust_hex_scroll();
                self.update_tree_selection_from_hex();
            }
        }
    }

    pub fn page_up(&mut self) {
        match self.focus {
            Focus::Tree => {
                let page_size = self.visible_tree_height.saturating_sub(2);
                self.tree_selected = self.tree_selected.saturating_sub(page_size);
                self.adjust_tree_scroll();
            }
            Focus::Hex => {
                let page_size = self.visible_hex_height.saturating_sub(2) * BYTES_PER_ROW;
                self.hex_selected = self.hex_selected.saturating_sub(page_size);
                self.adjust_hex_scroll();
                self.update_tree_selection_from_hex();
            }
        }
    }

    pub fn page_down(&mut self) {
        match self.focus {
            Focus::Tree => {
                if let Some(tree) = &self.tree {
                    let max = tree.flatten().len().saturating_sub(1);
                    let page_size = self.visible_tree_height.saturating_sub(2);
                    self.tree_selected = (self.tree_selected + page_size).min(max);
                    self.adjust_tree_scroll();
                }
            }
            Focus::Hex => {
                let max = self.raw_bytes.len().saturating_sub(1);
                let page_size = self.visible_hex_height.saturating_sub(2) * BYTES_PER_ROW;
                self.hex_selected = (self.hex_selected + page_size).min(max);
                self.adjust_hex_scroll();
                self.update_tree_selection_from_hex();
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
        self.tree.as_ref().and_then(|tree| {
            tree.get_path_to_offset(self.hex_selected).last().copied()
        })
    }


}


