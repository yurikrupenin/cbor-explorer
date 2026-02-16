use crate::cbor_tree::{CborNode, CborType, PathSegment};
use ciborium::value::Value;
use std::ops::Range;

/// Parsed CBOR item with its byte range
#[derive(Debug, Clone)]
pub struct ParsedCbor {
    pub value: Value,
    pub range: Range<usize>,
    pub children: Vec<ParsedCbor>,
}

impl ParsedCbor {
    pub fn to_node(
        &self,
        key: Option<String>,
        depth: usize,
        parent_path: Vec<PathSegment>,
    ) -> CborNode {
        let current_name = key.clone().unwrap_or_else(|| "root".to_string());
        let mut path = parent_path;
        path.push(PathSegment {
            name: current_name,
            depth,
        });

        let (value_type, value_preview, full_value, expanded) = match &self.value {
            Value::Null => (
                CborType::Null,
                "null".to_string(),
                "null".to_string(),
                false,
            ),
            Value::Bool(b) => (CborType::Bool, b.to_string(), b.to_string(), false),
            Value::Integer(i) => {
                let val: i128 = (*i).into();
                (CborType::Integer, val.to_string(), val.to_string(), false)
            }
            Value::Float(f) => (
                CborType::Float,
                format!("{:.6}", f),
                format!("{}", f),
                false,
            ),
            Value::Bytes(bytes) => {
                let full = bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let preview = if bytes.len() <= 16 {
                    full.clone()
                } else {
                    format!(
                        "{} ... ({} bytes)",
                        bytes[..8]
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(" "),
                        bytes.len()
                    )
                };
                (CborType::ByteString, preview, full, false)
            }
            Value::Text(s) => {
                let preview = if s.len() <= 40 {
                    format!("\"{}\"", s)
                } else {
                    format!("\"{}...\" ({} chars)", &s[..37], s.len())
                };
                (CborType::TextString, preview, s.clone(), false)
            }
            Value::Array(arr) => {
                let is_all_integers = arr.iter().all(|v| matches!(v, Value::Integer(_)));
                let preview = format!("[{} items]", arr.len());
                let full_value = if is_all_integers && !arr.is_empty() {
                    let items: Vec<String> = arr
                        .iter()
                        .map(|v| {
                            if let Value::Integer(i) = v {
                                let val: i128 = (*i).into();
                                format!("0x{:X}", val)
                            } else {
                                String::new()
                            }
                        })
                        .collect();
                    format!("[{}]", items.join(", "))
                } else {
                    format!("Array with {} items", arr.len())
                };
                (CborType::Array, preview, full_value, depth < 2)
            }
            Value::Map(map) => (
                CborType::Map,
                format!("{{{} entries}}", map.len()),
                format!("Map with {} entries", map.len()),
                depth < 2,
            ),
            Value::Tag(tag, _) => (
                CborType::Tag,
                format!("tag({})", tag),
                format!("Tag {}", tag),
                true,
            ),
            _ => (
                CborType::Null,
                "unknown".to_string(),
                "unknown".to_string(),
                false,
            ),
        };

        let mut children = Vec::new();
        if let Value::Array(_) = &self.value {
            for (i, child_parsed) in self.children.iter().enumerate() {
                children.push(child_parsed.to_node(
                    Some(format!("[{}]", i)),
                    depth + 1,
                    path.clone(),
                ));
            }
        } else if let Value::Map(map) = &self.value {
            let mut child_idx = 0;
            for _ in map {
                if child_idx + 1 < self.children.len() {
                    let key_parsed = &self.children[child_idx];
                    let val_parsed = &self.children[child_idx + 1];

                    let key_str = crate::cbor_tree::value_to_key_string(&key_parsed.value);

                    let mut node = val_parsed.to_node(Some(key_str), depth + 1, path.clone());

                    // Extend range to include key for better visualization in Hex view
                    node.range = key_parsed.range.start..val_parsed.range.end;

                    children.push(node);
                    child_idx += 2;
                }
            }
        } else if let Value::Tag(_, _) = &self.value {
            if let Some(child_parsed) = self.children.first() {
                children.push(child_parsed.to_node(None, depth + 1, path.clone()));
            }
        }

        CborNode {
            key,
            value_type,
            value_preview,
            full_value,
            children,
            expanded,
            depth,
            path,
            range: self.range.clone(),
        }
    }
}

pub struct CborParser<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> CborParser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }

    pub fn parse(&mut self) -> Option<ParsedCbor> {
        if self.position >= self.input.len() {
            return None;
        }
        // Start with root, proceed recrusively with child nodes
        self.parse_item()
    }

    fn parse_item(&mut self) -> Option<ParsedCbor> {
        if self.position >= self.input.len() {
            return None;
        }

        let start = self.position;
        let head = self.input[self.position];
        self.position += 1;

        let major = (head & 0xE0) >> 5;
        let info = head & 0x1F;

        let (len, _is_indefinite) = self.read_len(info)?;

        let mut children = Vec::new();

        match major {
            0 | 1 => {}
            2 | 3 => {
                if len == u64::MAX {
                    while self.peek_byte() != Some(0xFF) {
                        if let Some(chunk) = self.parse_item() {
                            children.push(chunk);
                        } else {
                            return None;
                        }
                    }
                    self.position += 1; // Skip 0xFF
                } else {
                    self.position += len as usize;
                }
            }
            4 => {
                // Array
                if len == u64::MAX {
                    while self.peek_byte() != Some(0xFF) {
                        if let Some(child) = self.parse_item() {
                            children.push(child);
                        } else {
                            return None;
                        }
                    }
                    self.position += 1;
                } else {
                    for _ in 0..len {
                        if let Some(child) = self.parse_item() {
                            children.push(child);
                        } else {
                            return None;
                        }
                    }
                }
            }
            5 => {
                // Map
                if len == u64::MAX {
                    while self.peek_byte() != Some(0xFF) {
                        if let Some(key) = self.parse_item() {
                            children.push(key);
                        } else {
                            return None;
                        }
                        if let Some(val) = self.parse_item() {
                            children.push(val);
                        } else {
                            return None;
                        }
                    }
                    self.position += 1;
                } else {
                    for _ in 0..len {
                        if let Some(key) = self.parse_item() {
                            children.push(key);
                        } else {
                            return None;
                        }
                        if let Some(val) = self.parse_item() {
                            children.push(val);
                        } else {
                            return None;
                        }
                    }
                }
            }
            6 => {
                // Tag
                if let Some(child) = self.parse_item() {
                    children.push(child);
                }
            }
            7 => {}
            _ => return None,
        }

        let end = self.position;
        if end > self.input.len() {
            return None;
        }
        let range = start..end;

        let slice = &self.input[range.clone()];
        let value = match ciborium::from_reader(std::io::Cursor::new(slice)) {
            Ok(v) => v,
            Err(_) => return None,
        };

        Some(ParsedCbor {
            value,
            range,
            children,
        })
    }

    fn read_len(&mut self, info: u8) -> Option<(u64, bool)> {
        match info {
            0..=23 => Some((info as u64, false)),
            24 => self.read_u8().map(|v| (v as u64, false)),
            25 => self.read_u16().map(|v| (v as u64, false)),
            26 => self.read_u32().map(|v| (v as u64, false)),
            27 => self.read_u64().map(|v| (v, false)),
            31 => Some((u64::MAX, true)),
            _ => None,
        }
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.position < self.input.len() {
            let b = self.input[self.position];
            self.position += 1;
            Some(b)
        } else {
            None
        }
    }

    fn read_u16(&mut self) -> Option<u16> {
        if self.position + 2 <= self.input.len() {
            let bytes = &self.input[self.position..self.position + 2];
            self.position += 2;
            Some(u16::from_be_bytes([bytes[0], bytes[1]]))
        } else {
            None
        }
    }

    fn read_u32(&mut self) -> Option<u32> {
        if self.position + 4 <= self.input.len() {
            let bytes = &self.input[self.position..self.position + 4];
            self.position += 4;
            Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            None
        }
    }

    fn read_u64(&mut self) -> Option<u64> {
        if self.position + 8 <= self.input.len() {
            let bytes = &self.input[self.position..self.position + 8];
            self.position += 8;
            Some(u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        } else {
            None
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::CborScanner;
    use crate::scanner::ScanMode;
    use ciborium::value::Value;

    #[test]
    fn test_scan_embedded_cbor() {
        // Create a buffer with: [garbage 10 bytes] [map] [garbage 5 bytes] [array] [garbage]
        // Use 0xFF to represent true garbage (invalid start byte)
        let mut data = Vec::new();
        data.extend_from_slice(&[0xFF; 10]); // Garbage 10 bytes

        // Map: {"a": 1} -> A1 61 61 01
        let map_bytes = vec![0xA1, 0x61, 0x61, 0x01];
        data.extend_from_slice(&map_bytes);

        data.extend_from_slice(&[0xFF; 5]); // Garbage

        // Array: [1, 2, 3, 4, 5] -> 85 01 02 03 04 05
        // Score: 10(base) + 5 * (1(base) + 5(depth)) = 10 + 30 = 40. > 30.
        let array_bytes = vec![0x85, 0x01, 0x02, 0x03, 0x04, 0x05];
        data.extend_from_slice(&array_bytes);

        data.extend_from_slice(&[0xFF]); // Garbage

        let scanner = CborScanner::new(&data);
        let chunks = scanner.scan_for_cbor_sequences(ScanMode::Auto);

        assert_eq!(chunks.len(), 2, "Should find 2 CBOR chunks");

        // First chunk should be the Map (score > Array)
        // Map Score: 10 + 25 + 1+1+15+5 + 1+5 = 63.

        let chunk1 = &chunks[0];
        assert!(chunk1.score > 0);
        assert_eq!(chunk1.items.len(), 1);
        // Check if map of 1
        if let Value::Map(m) = &chunk1.items[0].value {
            assert_eq!(m.len(), 1);
        }

        let chunk2 = &chunks[1];
        assert!(chunk2.score > 0);
        assert_eq!(chunk2.items.len(), 1);
        // Check if array of 5
        if let Value::Array(a) = &chunk2.items[0].value {
            assert_eq!(a.len(), 5);
        }
    }

    #[test]
    fn test_truncated_data() {
        // String of length 5, but we only provide 1 byte
        // 0x65 = text string, length 5
        let data = vec![0x65, 0x41];
        let scanner = CborScanner::new(&data);

        // Should not panic, should just return empty or what it found
        let chunks = scanner.scan_for_cbor_sequences(ScanMode::Auto);

        assert_eq!(
            chunks.len(),
            0,
            "Should not find valid CBOR in truncated data"
        );
    }
}
