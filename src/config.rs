#![allow(dead_code)]
pub const BYTES_PER_ROW: usize = 16;
pub const LAYOUT_SPLIT_PERCENT: u16 = 30;
pub const BORDER_HEIGHT_ADJUSTMENT: usize = 2; // Top + Bottom borders
pub const POPUP_MAX_WIDTH: u16 = 50;
pub const POPUP_MAX_HEIGHT: u16 = 20;

pub mod keys {
    pub const SWITCH_FOCUS: char = '\t';
    pub const EXPAND: char = ' ';
    pub const EXPAND_ALL: char = 'e';
    pub const COLLAPSE_ALL: char = 'c';
    pub const QUIT: char = 'q';
    pub const HELP: char = '?';
    pub const TOGGLE_HEX_INT: char = 'x';
    pub const THEME_SELECT: char = 't';
    pub const ESC: char = '\x1b'; // Escape key often maps to this char in some contexts, but ratatui handles KeyCode::Esc separately
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteType {
    Null,
    AsciiPrintable,
    AsciiWhitespace,
    AsciiOther,
    NonAscii,
}

pub fn get_byte_type(byte: u8) -> ByteType {
    if byte == 0x00 {
        ByteType::Null
    } else if byte.is_ascii_graphic() || byte == b' ' {
        ByteType::AsciiPrintable
    } else if byte.is_ascii_whitespace() {
        ByteType::AsciiWhitespace
    } else if byte.is_ascii() {
        ByteType::AsciiOther
    } else {
        ByteType::NonAscii
    }
}
