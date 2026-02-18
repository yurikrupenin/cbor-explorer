use crate::app::App;
use crate::scanner::{CborChunk, CborScanner, ScanMode};
use crate::ui::notification::NotificationSeverity;

pub trait Zoomable {
    fn zoom_in(&mut self);
    fn zoom_out(&mut self);
    fn zoom_toggle(&mut self);
}

impl Zoomable for App {
    fn zoom_in(&mut self) {
        let offset = self.hex_selected;

        // Scan for a single sequence starting from the offset
        let scanner = CborScanner::new(&self.raw_bytes[offset..]);
        // Force Single mode for Zoom In
        let mut chunks = scanner.scan_for_cbor_sequences(ScanMode::Single);

        if let Some(mut chunk) = chunks.pop() {
            // Adjust offsets to be absolute
            adjust_chunk_offset(&mut chunk, offset);

            let chunk_offset = chunk.offset;

            // Update state
            self.chunks = vec![chunk];
            self.is_zoomed = true;
            self.rebuild_tree();

            // Reset selection to the beginning of the new tree
            self.tree_selected = 0;
            self.tree_offset = 0;

            // Ensure hex selection matches the start of the chunk
            self.hex_selected = chunk_offset;
            self.adjust_hex_scroll();

            // Handle not having valid CBOR data directly at cursor,
            // inform the user we skipped some bytes before
            // encountering something that looks like valid data
            let message = if chunk_offset != offset {
                let skipped = chunk_offset - offset;
                format!(
                    "Zoomed in at offset 0x{:X} (skipped {} bytes of garbage)",
                    chunk_offset, skipped
                )
            } else {
                format!("Zoomed in at offset 0x{:X}", chunk_offset)
            };

            let severity = if chunk_offset != offset {
                NotificationSeverity::Warning
            } else {
                NotificationSeverity::Info
            };

            self.set_notification(message, severity, 5);
        } else {
            // Failed to find any CBOR data starting from the cursor position
            // and up to the end of the file.
            self.set_notification(
                "No valid CBOR sequence found starting from cursor".to_string(),
                NotificationSeverity::Error,
                5,
            );
        }
    }

    fn zoom_out(&mut self) {
        self.is_zoomed = false;
        // Force Auto mode when zooming out
        self.scan_mode = ScanMode::Auto;
        // Rescan entire file with current mode
        self.rescan_chunks();

        self.set_notification(
            "Zoomed out to full file".to_string(),
            NotificationSeverity::Info,
            5,
        );
    }

    fn zoom_toggle(&mut self) {
        if self.is_zoomed {
            self.zoom_out();
        } else {
            self.zoom_in();
        }
    }
}

// Helper to shift offsets in a chunk
fn adjust_chunk_offset(chunk: &mut CborChunk, offset: usize) {
    chunk.offset += offset;
    for item in &mut chunk.items {
        CborScanner::adjust_ranges(item, offset);
    }
}
