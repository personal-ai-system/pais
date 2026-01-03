//! Hook event dispatching

use super::{HookEvent, HookHandler, HookResult};

/// Dispatch a hook event to all registered handlers
#[allow(dead_code)] // Utility function for future handler composition
pub fn dispatch(event: HookEvent, payload: &serde_json::Value, handlers: &[Box<dyn HookHandler>]) -> HookResult {
    for handler in handlers {
        if handler.handles(event) {
            let result = handler.handle(event, payload);
            match &result {
                HookResult::Block { message } => {
                    log::info!("Hook blocked: {}", message);
                    return result;
                }
                HookResult::Error { message } => {
                    log::error!("Hook error: {}", message);
                    // Continue to next handler
                }
                HookResult::Allow => {
                    // Continue to next handler
                }
            }
        }
    }

    HookResult::Allow
}
