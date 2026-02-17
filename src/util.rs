use crate::cbor_tree::ConfidenceLevel;
use crate::config;
use crate::theme::Theme;
use ratatui::style::Color;

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

pub fn get_confidence_level(score: usize) -> ConfidenceLevel {
    if score >= config::CONFIDENCE_HIGH {
        ConfidenceLevel::Highest
    } else if score >= config::CONFIDENCE_MEDIUM {
        ConfidenceLevel::High
    } else if score >= config::CONFIDENCE_LOW {
        ConfidenceLevel::Low
    } else {
        ConfidenceLevel::Garbage
    }
}

pub fn get_confidence_appearance(level: ConfidenceLevel, theme: &Theme) -> (String, Color) {
    match level {
        ConfidenceLevel::Highest => ("● ".to_string(), theme.confidence_colors.highest),
        ConfidenceLevel::High => ("◎ ".to_string(), theme.confidence_colors.high),
        ConfidenceLevel::Low => ("○ ".to_string(), theme.confidence_colors.low),
        ConfidenceLevel::Garbage => ("× ".to_string(), theme.confidence_colors.garbage),
    }
}
