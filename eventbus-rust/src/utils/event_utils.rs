//! Event processing utilities

use crate::core::EventEnvelope;

/// Utility functions for event processing
pub struct EventUtils;

impl EventUtils {
    /// Generate a unique event ID
    pub fn generate_event_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    /// Validate event data
    pub fn validate_event(event: &EventEnvelope) -> bool {
        !event.topic.is_empty() && !event.event_id.is_empty()
    }
} 