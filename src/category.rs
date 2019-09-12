use serde::{Deserialize, Serialize};
use std::str;

pub const UNKNOWN_CATEGORY: &str = "Unknown";

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Category {
    pub name: String,
    pub patterns: Vec<String>,
}

impl Category {
    pub fn is_match(&self, input: &str) -> bool {
        self.patterns.iter().any(|s| {
            input
                .to_ascii_lowercase()
                .contains(String::from(s).to_ascii_lowercase().as_str())
        })
    }
}
