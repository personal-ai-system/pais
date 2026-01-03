//! Content categorization for history entries
//!
//! Analyzes content to determine if it should be categorized as:
//! - `learnings`: Problem-solving narratives, debugging discoveries
//! - `sessions`: Regular work sessions
//! - `research`: Investigation reports
//! - `decisions`: Architectural/design decisions

/// Content category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Category {
    Sessions,
    Learnings,
    Research,
    Decisions,
    Execution,
}

impl Category {
    /// Get the directory name for this category
    pub fn dir_name(&self) -> &'static str {
        match self {
            Category::Sessions => "sessions",
            Category::Learnings => "learnings",
            Category::Research => "research",
            Category::Decisions => "decisions",
            Category::Execution => "execution",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dir_name())
    }
}

/// Learning indicator patterns
const LEARNING_INDICATORS: &[&str] = &[
    "problem",
    "solved",
    "discovered",
    "fixed",
    "learned",
    "realized",
    "figured out",
    "root cause",
    "debugging",
    "issue was",
    "turned out",
    "the fix",
    "solution was",
    "bug was",
    "finally got",
    "breakthrough",
    "key insight",
    "mistake was",
    "error was",
    "found that",
    "root of the issue",
];

/// Research indicator patterns
const RESEARCH_INDICATORS: &[&str] = &[
    "investigating",
    "research",
    "analysis",
    "comparing",
    "evaluation",
    "looking into",
    "exploring",
    "options are",
    "alternatives",
    "trade-offs",
    "pros and cons",
    "benchmark",
];

/// Decision indicator patterns
const DECISION_INDICATORS: &[&str] = &[
    "decided to",
    "decision",
    "architecture",
    "design choice",
    "going with",
    "choosing",
    "will use",
    "approach is",
    "strategy is",
    "pattern we",
    "adopting",
];

/// Analyze content and determine its category
pub fn categorize_content(content: &str) -> Category {
    let content_lower = content.to_lowercase();

    // Count indicator matches
    let learning_score = count_matches(&content_lower, LEARNING_INDICATORS);
    let research_score = count_matches(&content_lower, RESEARCH_INDICATORS);
    let decision_score = count_matches(&content_lower, DECISION_INDICATORS);

    // Require at least 2 matches for specialized categories
    if learning_score >= 2 {
        return Category::Learnings;
    }

    if decision_score >= 2 {
        return Category::Decisions;
    }

    if research_score >= 2 {
        return Category::Research;
    }

    // Default to sessions
    Category::Sessions
}

/// Count how many indicator patterns match in the content
fn count_matches(content: &str, indicators: &[&str]) -> usize {
    indicators.iter().filter(|&&ind| content.contains(ind)).count()
}

/// Extract a summary from content (first meaningful paragraph or heading)
pub fn extract_summary(content: &str, max_len: usize) -> String {
    // Try to find a title (# heading)
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ")
            && !title.is_empty()
        {
            return truncate(title, max_len);
        }
    }

    // Fall back to first non-empty paragraph
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('-') {
            return truncate(trimmed, max_len);
        }
    }

    "Untitled".to_string()
}

/// Extract key topics/tags from content
pub fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();

    // Known technical terms to look for
    let tech_terms = [
        "rust",
        "python",
        "typescript",
        "javascript",
        "docker",
        "kubernetes",
        "aws",
        "gcp",
        "azure",
        "api",
        "cli",
        "database",
        "sql",
        "git",
        "ci",
        "cd",
        "test",
        "deploy",
        "build",
        "config",
        "yaml",
        "json",
        "toml",
        "http",
        "grpc",
        "websocket",
    ];

    // Look for common technical terms
    let content_lower = content.to_lowercase();
    for word in content_lower.split_whitespace() {
        // Strip common punctuation from word boundaries
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if tech_terms.contains(&clean) && !tags.contains(&clean.to_string()) {
            tags.push(clean.to_string());
        }
    }

    // Limit to first 5 tags
    tags.truncate(5);
    tags
}

/// Truncate a string to max length, adding ellipsis if needed
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_learning() {
        let content = "After debugging for an hour, I finally figured out the root cause. The problem was in the async handling. Fixed it by adding proper await.";
        assert_eq!(categorize_content(content), Category::Learnings);
    }

    #[test]
    fn test_categorize_research() {
        let content = "Investigating different approaches. Comparing options between A and B. The trade-offs are complex. Analysis shows option A is better.";
        assert_eq!(categorize_content(content), Category::Research);
    }

    #[test]
    fn test_categorize_decision() {
        let content = "Architecture decision: we're going with approach A. The design choice was influenced by scalability. Decided to use the pattern we discussed.";
        assert_eq!(categorize_content(content), Category::Decisions);
    }

    #[test]
    fn test_categorize_session() {
        let content = "Implemented the new feature. Added tests. Updated documentation.";
        assert_eq!(categorize_content(content), Category::Sessions);
    }

    #[test]
    fn test_extract_summary_with_title() {
        let content = "# My Great Title\n\nSome content here.";
        assert_eq!(extract_summary(content, 100), "My Great Title");
    }

    #[test]
    fn test_extract_summary_no_title() {
        let content = "This is the first paragraph.\n\nMore content.";
        assert_eq!(extract_summary(content, 100), "This is the first paragraph.");
    }

    #[test]
    fn test_extract_tags() {
        let content = "Working with Rust and Python. Deployed to AWS using Docker.";
        let tags = extract_tags(content);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"python".to_string()));
        assert!(tags.contains(&"aws".to_string()));
        assert!(tags.contains(&"docker".to_string()));
    }
}
