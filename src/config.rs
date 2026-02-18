use crossterm::event::KeyCode;

// Confidence levels for CBOR data
// (see scanner.rs for details)
pub const CONFIDENCE_HIGH: usize = 500;
pub const CONFIDENCE_MEDIUM: usize = 300;
pub const CONFIDENCE_LOW: usize = 100;

// UI constants
pub const BYTES_PER_ROW: usize = 16;
pub const LAYOUT_SPLIT_PERCENT: u16 = 30;
pub const BORDER_HEIGHT_ADJUSTMENT: usize = 2; // Top + Bottom borders
pub const POPUP_MAX_WIDTH: u16 = 50;
pub const POPUP_MAX_HEIGHT: u16 = 20;

// Mouse controls
pub const MOUSE_SCROLL_LINES_HEX: usize = 1;
pub const MOUSE_SCROLL_LINES_TREE: usize = 1;

// Navigation
pub const KEY_UP: &[KeyCode] = &[KeyCode::Up, KeyCode::Char('k')];
pub const KEY_DOWN: &[KeyCode] = &[KeyCode::Down, KeyCode::Char('j')];
pub const KEY_LEFT: &[KeyCode] = &[KeyCode::Left, KeyCode::Char('h')];
pub const KEY_RIGHT: &[KeyCode] = &[KeyCode::Right, KeyCode::Char('l')];
pub const KEY_TOP: &[KeyCode] = &[KeyCode::Home, KeyCode::Char('g')];
pub const KEY_BOTTOM: &[KeyCode] = &[KeyCode::End, KeyCode::Char('G')];
pub const KEY_PAGE_UP: &[KeyCode] = &[KeyCode::PageUp];
pub const KEY_PAGE_DOWN: &[KeyCode] = &[KeyCode::PageDown];

// Global Actions
pub const KEY_QUIT: &[KeyCode] = &[KeyCode::Char('q')];
pub const KEY_HELP_TOGGLE: &[KeyCode] = &[KeyCode::Char('?')];
pub const KEY_SWITCH_FOCUS: &[KeyCode] = &[KeyCode::Tab];
pub const KEY_TOGGLE_HEX_INT: &[KeyCode] = &[KeyCode::Char('x')];
pub const KEY_THEME_SELECT: &[KeyCode] = &[KeyCode::Char('t')];
pub const KEY_SEARCH: &[KeyCode] = &[KeyCode::Char('/')];
pub const KEY_GOTO: &[KeyCode] = &[KeyCode::Char(':')];
pub const KEY_NEXT: &[KeyCode] = &[KeyCode::Char('n')];
pub const KEY_PREV: &[KeyCode] = &[KeyCode::Char('N')];
pub const KEY_TOGGLE_POPUP: &[KeyCode] = &[KeyCode::Char(' ')];
pub const KEY_SORT: &[KeyCode] = &[KeyCode::Char('s')];
pub const KEY_MODE: &[KeyCode] = &[KeyCode::Char('m')];
pub const KEY_ZOOM: &[KeyCode] = &[KeyCode::Char('z')];

// Tree Actions
pub const KEY_EXPAND: &[KeyCode] = &[KeyCode::Enter, KeyCode::Right];
pub const KEY_EXPAND_ALL: &[KeyCode] = &[KeyCode::Char('e')];
pub const KEY_COLLAPSE_ALL: &[KeyCode] = &[KeyCode::Char('c')];

// Input Actions (Search/Goto/Theme)
pub const KEY_ENTER: &[KeyCode] = &[KeyCode::Enter];
pub const KEY_ESC: &[KeyCode] = &[KeyCode::Esc];
pub const KEY_BACKSPACE: &[KeyCode] = &[KeyCode::Backspace];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Quit,
    Help,
    SwitchFocus,
    ToggleHexInt,
    ThemeSelect,
    Search,
    Goto,
    Next,
    Prev,
    TogglePopup,
    Sort,
    Mode,
    Zoom,
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
    PageUp,
    PageDown,
    Expand,
    ExpandAll,
    CollapseAll,
    Enter,
    Esc,
    Backspace,
    Char(char),
    None,
}

use crossterm::event::KeyEvent;

pub fn resolve_key(key: KeyEvent) -> KeyAction {
    let code = key.code;
    if KEY_ENTER.contains(&code) {
        return KeyAction::Enter;
    }
    if KEY_ESC.contains(&code) {
        return KeyAction::Esc;
    }
    if KEY_QUIT.contains(&code) {
        return KeyAction::Quit;
    }
    if KEY_HELP_TOGGLE.contains(&code) {
        return KeyAction::Help;
    }
    if KEY_SWITCH_FOCUS.contains(&code) {
        return KeyAction::SwitchFocus;
    }
    if KEY_TOGGLE_HEX_INT.contains(&code) {
        return KeyAction::ToggleHexInt;
    }
    if KEY_THEME_SELECT.contains(&code) {
        return KeyAction::ThemeSelect;
    }
    if KEY_SEARCH.contains(&code) {
        return KeyAction::Search;
    }
    if KEY_GOTO.contains(&code) {
        return KeyAction::Goto;
    }
    if KEY_NEXT.contains(&code) {
        return KeyAction::Next;
    }
    if KEY_PREV.contains(&code) {
        return KeyAction::Prev;
    }
    if KEY_TOGGLE_POPUP.contains(&code) {
        return KeyAction::TogglePopup;
    }
    if KEY_SORT.contains(&code) {
        return KeyAction::Sort;
    }
    if KEY_MODE.contains(&code) {
        return KeyAction::Mode;
    }
    if KEY_ZOOM.contains(&code) {
        return KeyAction::Zoom;
    }
    if KEY_UP.contains(&code) {
        return KeyAction::Up;
    }
    if KEY_DOWN.contains(&code) {
        return KeyAction::Down;
    }
    if KEY_LEFT.contains(&code) {
        return KeyAction::Left;
    }
    if KEY_RIGHT.contains(&code) {
        return KeyAction::Right;
    }
    if KEY_TOP.contains(&code) {
        return KeyAction::Top;
    }
    if KEY_BOTTOM.contains(&code) {
        return KeyAction::Bottom;
    }
    if KEY_PAGE_UP.contains(&code) {
        return KeyAction::PageUp;
    }
    if KEY_PAGE_DOWN.contains(&code) {
        return KeyAction::PageDown;
    }
    if KEY_EXPAND.contains(&code) {
        return KeyAction::Expand;
    }
    if KEY_EXPAND_ALL.contains(&code) {
        return KeyAction::ExpandAll;
    }
    if KEY_COLLAPSE_ALL.contains(&code) {
        return KeyAction::CollapseAll;
    }
    if KEY_BACKSPACE.contains(&code) {
        return KeyAction::Backspace;
    }

    if let KeyCode::Char(c) = code {
        return KeyAction::Char(c);
    }
    KeyAction::None
}
