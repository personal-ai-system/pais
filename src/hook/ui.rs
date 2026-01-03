//! UI-related hooks
//!
//! Handles terminal UI updates like tab titles.

use super::{HookEvent, HookHandler, HookResult};

/// UI hook handler - updates terminal tab titles
pub struct UiHandler {
    enabled: bool,
}

impl UiHandler {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Update terminal tab title based on user prompt
    fn on_user_prompt_submit(&self, payload: &serde_json::Value) -> HookResult {
        let prompt = payload
            .get("prompt")
            .or_else(|| payload.get("message"))
            .or_else(|| payload.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if prompt.is_empty() {
            return HookResult::Allow;
        }

        // Extract a short summary from the prompt
        let summary = extract_task_summary(prompt);

        // Update terminal tab title using OSC escape sequence
        // OSC 0 sets both title and icon name
        // Format: \x1b]0;TITLE\x07
        print!("\x1b]0;🤖 {}\x07", summary);

        // Also set just the title (OSC 2)
        print!("\x1b]2;🤖 {}\x07", summary);

        log::debug!("Updated tab title to: 🤖 {}", summary);

        HookResult::Allow
    }
}

impl HookHandler for UiHandler {
    fn handles(&self, event: HookEvent) -> bool {
        self.enabled && event == HookEvent::UserPromptSubmit
    }

    fn handle(&self, event: HookEvent, payload: &serde_json::Value) -> HookResult {
        match event {
            HookEvent::UserPromptSubmit => self.on_user_prompt_submit(payload),
            _ => HookResult::Allow,
        }
    }
}

/// Extract a short task summary from user prompt
fn extract_task_summary(prompt: &str) -> String {
    const MAX_LEN: usize = 40;

    // Clean up the prompt
    let cleaned = prompt
        .lines()
        .next()
        .unwrap_or(prompt)
        .trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric());

    // Common task prefixes to strip
    let prefixes = [
        "please ",
        "can you ",
        "could you ",
        "i need you to ",
        "i want you to ",
        "help me ",
        "let's ",
        "now ",
    ];

    let mut summary = cleaned.to_lowercase();
    for prefix in prefixes {
        if let Some(rest) = summary.strip_prefix(prefix) {
            summary = rest.to_string();
            break;
        }
    }

    // Capitalize first letter
    let summary = if let Some(first) = summary.chars().next() {
        format!("{}{}", first.to_uppercase(), &summary[first.len_utf8()..])
    } else {
        summary
    };

    // Truncate if too long
    if summary.len() > MAX_LEN {
        format!("{}...", &summary[..MAX_LEN - 3])
    } else {
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_task_summary_simple() {
        assert_eq!(extract_task_summary("Fix the bug"), "Fix the bug");
    }

    #[test]
    fn test_extract_task_summary_strips_prefix() {
        assert_eq!(extract_task_summary("Please fix the bug"), "Fix the bug");
        assert_eq!(extract_task_summary("Can you fix this?"), "Fix this?");
        assert_eq!(extract_task_summary("I need you to refactor"), "Refactor");
    }

    #[test]
    fn test_extract_task_summary_truncates() {
        let long_prompt = "Implement a very long feature that does many things and has lots of requirements";
        let summary = extract_task_summary(long_prompt);
        assert!(summary.len() <= 40);
        assert!(summary.ends_with("..."));
    }

    #[test]
    fn test_extract_task_summary_multiline() {
        let prompt = "Fix the bug\n\nHere are the details:\n- error 1\n- error 2";
        assert_eq!(extract_task_summary(prompt), "Fix the bug");
    }

    #[test]
    fn test_extract_task_summary_capitalizes() {
        assert_eq!(extract_task_summary("fix the bug"), "Fix the bug");
    }

    #[test]
    fn test_handles_user_prompt_submit() {
        let handler = UiHandler::new(true);
        assert!(handler.handles(HookEvent::UserPromptSubmit));
        assert!(!handler.handles(HookEvent::PreToolUse));
    }

    #[test]
    fn test_disabled_handler() {
        let handler = UiHandler::new(false);
        assert!(!handler.handles(HookEvent::UserPromptSubmit));
    }
}
