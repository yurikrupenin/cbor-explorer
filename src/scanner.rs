/// This module impelemnts scanning of arbitrary data for
/// CBOR sequences. It uses some simple heuristics to detect
/// chunks of data likely to be CBOR, and then sorts
/// them using a scoring system.
///
/// This code seems to be somewhat good at detecting and sorting
/// likely CBOR chunks by prioritizing complex structures:
///
/// 1. The implementation prioritizes tree structures
///    with multiple children
/// 2. The implementation prioritizes maps with string keys
///
/// This is not a perfect algorithm, and it is biased towards
/// detecting CBOR uses where it is employed as "compact binary JSON".
///
/// You will probably be out of luck if your use case is embedding
/// short arrays of simple types via non-nesting linear sequences.
///
/// Ultimately, since CBOR does not contain any magic start
/// bytes[^1], it is extremely easy to come up with nothing but
/// false positive. The biggest problem of this implementation
/// is that it may skip actual CBOR sequences by assuming they
/// are part of erroneusly detected large arrays/byte strings/etc.
///
/// We do some attempts to minimize this by discarding suspiciously
/// long top-level data items (see MAX_TEXT_CHUNK_SIZE/MAX_BYTES_CHUNK_SIZE),
/// but this can be further tweaked (e.g. by discarding suspicios
/// top-level items only if they are not parts of long sequences).
///
/// Finally, the default-on Single mode is here to prevent us from
/// discarding any data unless the user enables heuristics explicitly.
///
/// [^1]: There's an optional "Self-described CBOR" tag, see the RFC:
///       https://www.rfc-editor.org/rfc/rfc8949.html#name-self-described-cbor.
///       It is not currently supported by this implementation,
///       would be great to introduce it some day.
use crate::cbor_parser::{CborParser, ParsedCbor};
use ciborium::value::Value;

// Scoring constants for the CBOR container detection heuristic.
// We do some assumptions here, they basically boil down to
// "programmers really love nested containers with multiple children,
// and they love maps with string keys the most".
const SCORE_MAP_BASE: usize = 20;
const SCORE_MAP_STRING_KEY_BONUS: usize = 100;
const SCORE_MAP_OTHER_KEY_BONUS: usize = 20;
const SCORE_ARRAY_BASE: usize = 10;
const SCORE_TAG_BASE: usize = 5;
const SCORE_TEXT_BASE: usize = 5;
const SCORE_TEXT_PRINTABLE_BONUS: usize = 15;
const SCORE_PRIMITIVE_BASE: usize = 5;
const SCORE_NESTING_BONUS: usize = 5;

const SCORE_THRESHOLD: usize = 30;

const MAX_TEXT_CHUNK_SIZE: usize = 128;
const MAX_BYTES_CHUNK_SIZE: usize = 128;

#[derive(Debug, Clone)]
pub struct CborChunk {
    pub offset: usize,
    pub items: Vec<ParsedCbor>,
    pub score: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanMode {
    Single, // Stop after first valid sequence, no heuristics
    Auto,   // Full scan with heuristics
}

pub struct CborScanner<'a> {
    input: &'a [u8],
}

impl<'a> CborScanner<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    /// Scans the input for valid CBOR sequences.
    /// Returns a list of parsed chunks found in the input, sorted by score.
    pub fn scan_for_cbor_sequences(&self, mode: ScanMode) -> Vec<CborChunk> {
        let mut chunks: Vec<CborChunk> = Vec::new();
        let mut current_chunk_items: Vec<ParsedCbor> = Vec::new();
        let mut current_chunk_offset = 0;

        let mut pos = 0;

        while pos < self.input.len() {
            let mut parser = CborParser::new(&self.input[pos..]);

            // Try to parse an item
            if let Some(mut item) = parser.parse() {
                // Adjust item range to be absolute
                let item_len = item.range.len();
                // CborParser returns ranges relative to its input (slice starting at pos).
                // We need to shift all ranges in the item tree by `pos`.
                Self::adjust_ranges(&mut item, pos);

                let accept_item = match mode {
                    ScanMode::Single => true,
                    ScanMode::Auto => {
                        // Heuristic Checks
                        let is_complex = matches!(
                            item.value,
                            Value::Map(_) | Value::Array(_) | Value::Tag(_, _)
                        );

                        // A single text string
                        let is_significant_simple = match &item.value {
                            Value::Text(s) => s.len() > 2 && s.len() < MAX_TEXT_CHUNK_SIZE,
                            Value::Bytes(b) => b.len() > 2 && b.len() < MAX_BYTES_CHUNK_SIZE,
                            _ => false,
                        };

                        let is_adjacent = current_chunk_items
                            .last()
                            .map(|prev: &ParsedCbor| prev.range.end == item.range.start)
                            .unwrap_or(false);
                        let is_start = item.range.start == 0;
                        let is_anchored = is_start || is_adjacent;

                        is_complex || is_significant_simple || is_anchored
                    }
                };

                if accept_item {
                    // Check adjacency to current chunk
                    let is_adjacent = current_chunk_items
                        .last()
                        .map(|prev: &ParsedCbor| prev.range.end == item.range.start)
                        .unwrap_or(false);

                    if !is_adjacent && !current_chunk_items.is_empty() {
                        // Gap detected
                        self.finalize_chunk(
                            &mut chunks,
                            current_chunk_items,
                            current_chunk_offset,
                            mode,
                        );
                        current_chunk_items = Vec::new();
                        current_chunk_offset = item.range.start;

                        // Single mode: stop after first valid sequence
                        if mode == ScanMode::Single {
                            break;
                        }
                    } else if current_chunk_items.is_empty() {
                        current_chunk_offset = item.range.start;
                    }

                    current_chunk_items.push(item);
                    pos += item_len;
                    continue;
                }
            }

            // If parsed but rejected (heuristic) OR valid parser didn't find anything (shouldn't happen with `if let Some` above if parser works correctly for all valid items)
            // But wait, if `parser.parse()` fails, we fall through to here.

            // If invalid or ignored:
            if !current_chunk_items.is_empty() {
                self.finalize_chunk(&mut chunks, current_chunk_items, current_chunk_offset, mode);
                current_chunk_items = Vec::new();

                if mode == ScanMode::Single {
                    // In Single mode, encountering garbage after a valid sequence means we are done
                    break;
                }
            }

            pos += 1;
        }

        if !current_chunk_items.is_empty() {
            self.finalize_chunk(&mut chunks, current_chunk_items, current_chunk_offset, mode);
        }

        // Sort chunks by score descending only in Auto mode
        if mode == ScanMode::Auto {
            chunks.sort_by(|a, b| b.score.cmp(&a.score));
        }

        chunks
    }

    fn finalize_chunk(
        &self,
        chunks: &mut Vec<CborChunk>,
        items: Vec<ParsedCbor>,
        offset: usize,
        mode: ScanMode,
    ) {
        let score: usize = items.iter().map(|i| self.calculate_score(i)).sum();

        let keep_chunk = match mode {
            ScanMode::Single => true,
            ScanMode::Auto => score >= SCORE_THRESHOLD,
        };

        if keep_chunk {
            chunks.push(CborChunk {
                offset,
                items,
                score,
            });
        }
    }

    pub fn adjust_ranges(item: &mut ParsedCbor, offset: usize) {
        item.range.start += offset;
        item.range.end += offset;
        for child in &mut item.children {
            Self::adjust_ranges(child, offset);
        }
    }

    pub fn calculate_score(&self, item: &ParsedCbor) -> usize {
        Self::calculate_score_recursive(item, 0)
    }

    fn calculate_score_recursive(item: &ParsedCbor, depth: usize) -> usize {
        let mut score = match &item.value {
            Value::Map(m) => {
                let mut map_score = SCORE_MAP_BASE;

                for (k, _) in m {
                    if let Value::Text(_) = k {
                        map_score += SCORE_MAP_STRING_KEY_BONUS;
                    } else {
                        map_score += SCORE_MAP_OTHER_KEY_BONUS;
                    }
                }

                map_score
            }
            Value::Array(_) => SCORE_ARRAY_BASE,
            Value::Tag(_, _) => SCORE_TAG_BASE,
            Value::Text(s) => {
                let is_printable = s
                    .chars()
                    .all(|c| c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_whitespace());
                SCORE_TEXT_BASE
                    + if is_printable {
                        SCORE_TEXT_PRINTABLE_BONUS
                    } else {
                        0
                    }
            }
            Value::Bytes(_) => SCORE_TEXT_BASE,
            _ => SCORE_PRIMITIVE_BASE,
        };

        for child in &item.children {
            score += Self::calculate_score_recursive(child, depth + 1);
        }

        if depth > 0 {
            score += SCORE_NESTING_BONUS;
        }

        score
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_calculation() {
        let mut data = Vec::new();
        // Create a complex nested map: {"a": {"b": {"c": "d"}}}
        // A1 61 61 A1 61 62 A1 61 63 61 64
        let complex_map = vec![
            0xA1, 0x61, 0x61, // {"a": ...
            0xA1, 0x61, 0x62, //   {"b": ...
            0xA1, 0x61, 0x63, //     {"c": ...
            0x61, 0x64, //       "d"}}}
        ];
        data.extend_from_slice(&complex_map);

        let scanner = CborScanner::new(&data);
        let chunks = scanner.scan_for_cbor_sequences(ScanMode::Auto);

        assert_eq!(chunks.len(), 1);
        let chunk = &chunks[0];

        assert_eq!(
            chunk.score, 470,
            "Score should be exactly 470 for this nested structure"
        );
    }

    #[test]
    fn test_range_offset() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0xFF; 10]); // Garbage 10 bytes
                                             // Map: {"a": 1} -> A1 61 61 01 (length 4)
        let map_bytes = vec![0xA1, 0x61, 0x61, 0x01];
        data.extend_from_slice(&map_bytes);

        let scanner = CborScanner::new(&data);
        let chunks = scanner.scan_for_cbor_sequences(ScanMode::Auto);

        assert_eq!(chunks.len(), 1);
        let chunk = &chunks[0];
        assert_eq!(chunk.offset, 10);
        let item = &chunk.items[0];
        println!("Item range: {:?}", item.range);
        assert_eq!(item.range, 10..14);
    }
}
