use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub custom: HashMap<String, String>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            name: None,
            description: None,
            tags: Vec::new(),
            created_at: now,
            modified_at: now,
            custom: HashMap::new(),
        }
    }
}

impl Metadata {
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
        self.touch();
    }

    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }
}
