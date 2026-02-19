use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use std::time::Duration;

const NOTIFICATION_HEIGHT: u16 = 3;
const NOTIFICATION_HORIZONTAL_PADDING: u16 = 4;
const NOTIFICATION_LEFT_OFFSET: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
}

pub struct Notification {
    pub message: String,
    pub severity: NotificationSeverity,
    #[cfg(not(target_arch = "wasm32"))]
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn new(message: String, severity: NotificationSeverity, duration_secs: u64) -> Self {
        Self {
            message,
            severity,
            #[cfg(not(target_arch = "wasm32"))]
            created_at: Instant::now(),
            duration: Duration::from_secs(duration_secs),
        }
    }

    pub fn is_expired(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return self.created_at.elapsed() > self.duration;

        #[cfg(target_arch = "wasm32")]
        return false; // TODO: no timer support on WASM currently -> notifications get stuck
    }
}

/// Draws a pop-up notification in a bottom left corner
// (TODO: configurable?) of the provided area Rect.
pub fn draw(frame: &mut Frame, notification: &Notification, area: Rect, theme: &Theme) {
    let severity_style = match notification.severity {
        NotificationSeverity::Info => Style::default().fg(theme.notification_colors.info),
        NotificationSeverity::Warning => Style::default()
            .fg(theme.notification_colors.warning)
            .bold(),
        NotificationSeverity::Error => Style::default().fg(theme.notification_colors.error).bold(),
    };

    let text = Span::styled(&notification.message, severity_style);

    // Calculate width based on text length + padding
    let width = (notification.message.len() as u16) + NOTIFICATION_HORIZONTAL_PADDING;
    let height = NOTIFICATION_HEIGHT; // Border + text
    let inner_width = area.width - 2;

    let x = area.x + NOTIFICATION_LEFT_OFFSET;
    let y = area.y + area.height.saturating_sub(height);

    let popup_area = Rect::new(x, y, width.min(inner_width), height);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.popup_border))
        .bg(theme.popup_bg);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}
