//! Observability module for real-time event streaming
//!
//! Provides event emission to multiple sinks:
//! - File (JSONL) - writes to history/raw-events/
//! - Stdout - prints formatted events
//! - HTTP - POSTs events to configured endpoint

pub mod emitter;

pub use emitter::{Event, EventEmitter};
