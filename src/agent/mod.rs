//! Agent system for composable AI personalities
//!
//! Agents are named personas with trait compositions that affect:
//! - Prompt prefix (injected context)
//! - History routing (where outputs go)
//! - Communication style

pub mod loader;
pub mod traits;
