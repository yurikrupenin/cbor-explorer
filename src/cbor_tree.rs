use ciborium::Value;
use std::fmt;

/// Represents a node in the CBOR tree structure
#[derive(Debug, Clone)]
pub struct CborNode {
    pub key: Option<String>,
    pub value_type: CborType,
    pub value_preview: String,
    pub full_value: String,
    pub children: Vec<CborNode>,
    pub expanded: bool,
    pub depth: usize,
    pub path: Vec<PathSegment>,
    pub range: std::ops::Range<usize>,
}

/// A segment in the breadcrumb path
#[derive(Debug, Clone)]
pub struct PathSegment {
    pub name: String,
    pub depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CborType {
    Null,
    Bool,
    Integer,
    Float,
    ByteString,
    TextString,
    Array,
    Map,
    Tag,
}

impl fmt::Display for CborType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CborType::Null => write!(f, "null"),
            CborType::Bool => write!(f, "bool"),
            CborType::Integer => write!(f, "int"),
            CborType::Float => write!(f, "float"),
            CborType::ByteString => write!(f, "bytes"),
            CborType::TextString => write!(f, "text"),
            CborType::Array => write!(f, "array"),
            CborType::Map => write!(f, "map"),
            CborType::Tag => write!(f, "tag"),
        }
    }
}

impl CborNode {


    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get all visible nodes as a flat list for rendering
    pub fn flatten(&self) -> Vec<&CborNode> {
        let mut result = vec![self];
        if self.expanded {
            for child in &self.children {
                result.extend(child.flatten());
            }
        }
        result
    }

    /// Get mutable reference to node at given flat index
    pub fn get_node_at_index_mut(&mut self, target_index: usize) -> Option<&mut CborNode> {
        let mut current_index = 0;
        self.find_node_at_index_mut(target_index, &mut current_index)
    }

    fn find_node_at_index_mut(
        &mut self,
        target_index: usize,
        current_index: &mut usize,
    ) -> Option<&mut CborNode> {
        if *current_index == target_index {
            return Some(self);
        }
        *current_index += 1;

        if self.expanded {
            for child in &mut self.children {
                if let Some(node) = child.find_node_at_index_mut(target_index, current_index) {
                    return Some(node);
                }
            }
        }
        None
    }

    pub fn expand_all(&mut self) {
        self.expanded = true;
        for child in &mut self.children {
            child.expand_all();
        }
    }

    pub fn collapse_all(&mut self) {
        self.expanded = false;
        for child in &mut self.children {
            child.collapse_all();
        }
    }
    
    /// Helper to find the path of nodes containing the given offset.
    /// Returns a vector from Root to the deepest node containing the offset.
    pub fn get_path_to_offset(&self, offset: usize) -> Vec<&CborNode> {
        let mut path = Vec::new();
        if self.range.contains(&offset) {
            path.push(self);
            for child in &self.children {
                let sub = child.get_path_to_offset(offset);
                if !sub.is_empty() {
                    path.extend(sub);
                    break;
                }
            }
        }
        path
    }
    /// Helper to expand all nodes in the path to a given offset
    pub fn expand_path_to_offset(&mut self, offset: usize) -> bool {
        if self.range.contains(&offset) {
            // If this node contains the offset, it's part of the path.
            // We should expand it so its children become visible candidates.
            self.expanded = true;
            
            // Check children recursively
            for child in &mut self.children {
                if child.expand_path_to_offset(offset) {
                    return true;
                }
            }
            return true;
        }
        false
    }
}

pub fn value_to_key_string(value: &Value) -> String {
    match value {
        Value::Text(s) => s.clone(),
        Value::Integer(i) => {
            let val: i128 = (*i).into();
            val.to_string()
        }
        Value::Bytes(b) => format!(
            "0x{}",
            b.iter().map(|x| format!("{:02x}", x)).collect::<String>()
        ),
        _ => "?".to_string(),
    }
}
