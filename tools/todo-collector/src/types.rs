//! Type definitions for TODO Collector

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of TODO comment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TodoType {
    Todo,
    Fixme,
    Hack,
    Xxx,
    Note,
}

impl TodoType {
    /// Get the priority of the TODO type (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            TodoType::Fixme => 1,
            TodoType::Todo => 2,
            TodoType::Hack => 3,
            TodoType::Xxx => 4,
            TodoType::Note => 5,
        }
    }

    /// Parse a string into a TodoType
    pub fn from_str(s: &str) -> Option<TodoType> {
        match s.to_uppercase().as_str() {
            "TODO" => Some(TodoType::Todo),
            "FIXME" => Some(TodoType::Fixme),
            "HACK" => Some(TodoType::Hack),
            "XXX" => Some(TodoType::Xxx),
            "NOTE" => Some(TodoType::Note),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            TodoType::Todo => "TODO",
            TodoType::Fixme => "FIXME",
            TodoType::Hack => "HACK",
            TodoType::Xxx => "XXX",
            TodoType::Note => "NOTE",
        }
    }
}

/// A single TODO item found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// Type of the TODO (TODO, FIXME, HACK, etc.)
    #[serde(rename = "type")]
    pub todo_type: TodoType,
    /// File path where the TODO was found
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// The content/message of the TODO
    pub content: String,
}

/// Summary of TODO counts by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoSummary {
    /// Total count of all TODOs
    pub total: usize,
    /// Count by type
    pub by_type: HashMap<String, usize>,
}

/// The complete scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Summary statistics
    pub summary: TodoSummary,
    /// All found TODO items
    pub items: Vec<TodoItem>,
}

impl ScanResult {
    /// Create a new scan result from a list of TODO items
    pub fn from_items(mut items: Vec<TodoItem>) -> Self {
        let total = items.len();
        
        // Count by type
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for item in &items {
            *by_type.entry(item.todo_type.as_str().to_string()).or_insert(0) += 1;
        }

        // Sort by file path and line number
        items.sort_by(|a, b| {
            a.file.cmp(&b.file).then(a.line.cmp(&b.line))
        });

        ScanResult {
            summary: TodoSummary { total, by_type },
            items,
        }
    }

    /// Sort items by priority (FIXME > TODO > HACK > XXX > NOTE)
    pub fn sort_by_priority(&mut self) {
        self.items.sort_by(|a, b| {
            a.todo_type.priority()
                .cmp(&b.todo_type.priority())
                .then(a.file.cmp(&b.file))
                .then(a.line.cmp(&b.line))
        });
    }
}

