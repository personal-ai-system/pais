//! History storage and retrieval
//!
//! Stores session data, learnings, decisions, and insights as markdown files.
//! Structure: ~/.config/pais/history/<category>/<date>/<id>.md

use chrono::{DateTime, Local, NaiveDate};
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub category: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Local>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(category: &str, title: &str, content: &str) -> Self {
        Self {
            id: generate_id(),
            category: category.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
            created_at: Local::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Convert to markdown format
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Frontmatter
        md.push_str("---\n");
        md.push_str(&format!("id: {}\n", self.id));
        md.push_str(&format!("title: {}\n", self.title));
        md.push_str(&format!("category: {}\n", self.category));
        md.push_str(&format!(
            "created_at: {}\n",
            self.created_at.format("%Y-%m-%dT%H:%M:%S%z")
        ));
        if !self.tags.is_empty() {
            md.push_str(&format!("tags: [{}]\n", self.tags.join(", ")));
        }
        for (key, value) in &self.metadata {
            md.push_str(&format!("{}: {}\n", key, value));
        }
        md.push_str("---\n\n");

        // Title
        md.push_str(&format!("# {}\n\n", self.title));

        // Content
        md.push_str(&self.content);
        md.push('\n');

        md
    }

    /// Parse from markdown format
    pub fn from_markdown(content: &str, path: &Path) -> Result<Self> {
        let mut lines = content.lines();
        let mut in_frontmatter = false;
        let mut frontmatter = String::new();
        let mut body = String::new();

        for line in lines.by_ref() {
            if line == "---" {
                if in_frontmatter {
                    break;
                } else {
                    in_frontmatter = true;
                    continue;
                }
            }
            if in_frontmatter {
                frontmatter.push_str(line);
                frontmatter.push('\n');
            }
        }

        // Rest is body
        for line in lines {
            body.push_str(line);
            body.push('\n');
        }

        // Parse frontmatter (simple key: value parsing)
        let mut id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let mut title = String::new();
        let mut category = String::new();
        let mut created_at = Local::now();
        let mut tags = Vec::new();
        let mut metadata = std::collections::HashMap::new();

        for line in frontmatter.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "id" => id = value.to_string(),
                    "title" => title = value.to_string(),
                    "category" => category = value.to_string(),
                    "created_at" => {
                        if let Ok(dt) = DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%z") {
                            created_at = dt.with_timezone(&Local);
                        }
                    }
                    "tags" => {
                        let tag_str = value.trim_start_matches('[').trim_end_matches(']');
                        tags = tag_str.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    _ => {
                        metadata.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        // Extract title from body if not in frontmatter
        if title.is_empty() {
            for line in body.lines() {
                if let Some(stripped) = line.strip_prefix("# ") {
                    title = stripped.to_string();
                    break;
                }
            }
        }

        Ok(Self {
            id,
            category,
            title,
            content: body.trim().to_string(),
            tags,
            created_at,
            metadata,
        })
    }
}

/// History storage
pub struct HistoryStore {
    base_path: PathBuf,
}

impl HistoryStore {
    /// Create a new history store
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Store an entry
    pub fn store(&self, entry: &HistoryEntry) -> Result<PathBuf> {
        let date = entry.created_at.format("%Y-%m-%d").to_string();
        let dir = self.base_path.join(&entry.category).join(&date);
        fs::create_dir_all(&dir).context("Failed to create history directory")?;

        let filename = format!("{}.md", entry.id);
        let path = dir.join(&filename);

        fs::write(&path, entry.to_markdown()).context("Failed to write history entry")?;

        log::info!("Stored history entry: {}", path.display());
        Ok(path)
    }

    /// List categories
    pub fn categories(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut categories = Vec::new();
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                categories.push(name.to_string());
            }
        }
        categories.sort();
        Ok(categories)
    }

    /// Count entries in a category
    pub fn count(&self, category: &str) -> Result<usize> {
        let cat_path = self.base_path.join(category);
        if !cat_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for date_entry in fs::read_dir(&cat_path)? {
            let date_entry = date_entry?;
            if date_entry.path().is_dir() {
                for file_entry in fs::read_dir(date_entry.path())? {
                    let file_entry = file_entry?;
                    if file_entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Get recent entries
    pub fn recent(&self, category: Option<&str>, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut entries = Vec::new();
        let mut paths = Vec::new();

        // Collect all entry paths
        let categories: Vec<String> = match category {
            Some(c) => vec![c.to_string()],
            None => self.categories()?,
        };

        for cat in categories {
            let cat_path = self.base_path.join(&cat);
            if !cat_path.exists() {
                continue;
            }

            for date_entry in fs::read_dir(&cat_path)? {
                let date_entry = date_entry?;
                if date_entry.path().is_dir() {
                    for file_entry in fs::read_dir(date_entry.path())? {
                        let file_entry = file_entry?;
                        let path = file_entry.path();
                        if path.extension().map(|e| e == "md").unwrap_or(false) {
                            paths.push(path);
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        paths.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        // Load entries up to limit
        for path in paths.into_iter().take(limit) {
            let content = fs::read_to_string(&path)?;
            if let Ok(entry) = HistoryEntry::from_markdown(&content, &path) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Query entries with regex
    pub fn query(
        &self,
        pattern: &str,
        category: Option<&str>,
        since: Option<NaiveDate>,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>> {
        let regex = regex::Regex::new(pattern).context("Invalid regex pattern")?;
        let mut entries = Vec::new();

        let categories: Vec<String> = match category {
            Some(c) => vec![c.to_string()],
            None => self.categories()?,
        };

        for cat in categories {
            let cat_path = self.base_path.join(&cat);
            if !cat_path.exists() {
                continue;
            }

            for date_entry in fs::read_dir(&cat_path)? {
                let date_entry = date_entry?;
                let date_path = date_entry.path();

                if !date_path.is_dir() {
                    continue;
                }

                // Check date filter
                if let Some(since_date) = since
                    && let Some(date_str) = date_path.file_name().and_then(|s| s.to_str())
                    && let Ok(entry_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    && entry_date < since_date
                {
                    continue;
                }

                for file_entry in fs::read_dir(&date_path)? {
                    let file_entry = file_entry?;
                    let path = file_entry.path();

                    if !path.extension().map(|e| e == "md").unwrap_or(false) {
                        continue;
                    }

                    let content = fs::read_to_string(&path)?;

                    if !regex.is_match(&content) {
                        continue;
                    }

                    if let Ok(entry) = HistoryEntry::from_markdown(&content, &path) {
                        entries.push(entry);
                        if entries.len() >= limit {
                            return Ok(entries);
                        }
                    }
                }
            }
        }

        // Sort by date (newest first)
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(entries)
    }
}

/// Generate a unique ID for an entry
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    format!("{:x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_to_markdown() {
        let entry = HistoryEntry::new("sessions", "Test Session", "This is a test")
            .with_tag("test")
            .with_metadata("project", "pais");

        let md = entry.to_markdown();
        assert!(md.contains("title: Test Session"));
        assert!(md.contains("category: sessions"));
        assert!(md.contains("# Test Session"));
        assert!(md.contains("This is a test"));
    }

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = generate_id();
        assert!(!id1.is_empty());
        assert_ne!(id1, id2);
    }
}
