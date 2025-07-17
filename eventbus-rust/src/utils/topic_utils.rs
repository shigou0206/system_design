//! Topic utilities for event bus
//! 
//! This module provides utilities for working with event topics including
//! normalization, validation, and pattern matching.

use regex::Regex;
use once_cell::sync::Lazy;

use crate::core::{EventBusError, EventBusResult};

/// Regex for valid topic names
static TOPIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9._-]*[a-zA-Z0-9])?$").unwrap()
});

/// Maximum topic length
const MAX_TOPIC_LENGTH: usize = 256;

/// Minimum topic length
const MIN_TOPIC_LENGTH: usize = 1;

/// Normalize a topic name
/// 
/// This function:
/// - Trims whitespace
/// - Converts to lowercase
/// - Validates the format
/// - Ensures length constraints
pub fn normalize_topic(topic: &str) -> EventBusResult<String> {
    let normalized = topic.trim().to_lowercase();
    
    // Check length constraints
    if normalized.len() < MIN_TOPIC_LENGTH {
        return Err(EventBusError::validation(
            format!("Topic too short: '{}' (min: {} chars)", normalized, MIN_TOPIC_LENGTH)
        ));
    }
    
    if normalized.len() > MAX_TOPIC_LENGTH {
        return Err(EventBusError::validation(
            format!("Topic too long: '{}' (max: {} chars)", normalized, MAX_TOPIC_LENGTH)
        ));
    }
    
    // Validate format
    if !TOPIC_REGEX.is_match(&normalized) {
        return Err(EventBusError::validation(
            format!("Invalid topic format: '{}'. Must contain only alphanumeric characters, dots, underscores, and hyphens", normalized)
        ));
    }
    
    Ok(normalized)
}

/// Check if a topic name is valid
pub fn is_valid_topic(topic: &str) -> bool {
    normalize_topic(topic).is_ok()
}

/// Check if a topic matches a pattern with wildcards
/// 
/// Supports:
/// - * for single-level wildcards
/// - ** for multi-level wildcards
/// - . as level separator
pub fn topic_matches_pattern(topic: &str, pattern: &str) -> bool {
    // Simple wildcard matching
    if pattern == "*" || pattern == "**" {
        return true;
    }
    
    // Convert pattern to regex step by step
    let mut regex_pattern = pattern.to_string();
    
    // Handle ** first (multi-level wildcard)
    regex_pattern = regex_pattern.replace("**", "DOUBLE_WILDCARD");
    
    // Handle single * (single-level wildcard) 
    regex_pattern = regex_pattern.replace("*", "SINGLE_WILDCARD");
    
    // Escape literal dots
    regex_pattern = regex_pattern.replace(".", r"\.");
    
    // Replace wildcards with regex equivalents
    regex_pattern = regex_pattern.replace("DOUBLE_WILDCARD", ".*");
    regex_pattern = regex_pattern.replace("SINGLE_WILDCARD", ".*");
    
    let regex_pattern = format!("^{}$", regex_pattern);
    
    if let Ok(regex) = Regex::new(&regex_pattern) {
        regex.is_match(topic)
    } else {
        false
    }
}

/// Extract namespace from a hierarchical topic
/// 
/// For topic "workflow.execution.completed", returns "workflow"
/// For topic "system", returns "system"
pub fn extract_namespace(topic: &str) -> String {
    topic.split('.').next().unwrap_or(topic).to_string()
}

/// Extract all levels from a hierarchical topic
/// 
/// For topic "workflow.execution.completed", returns ["workflow", "execution", "completed"]
pub fn extract_topic_levels(topic: &str) -> Vec<String> {
    topic.split('.').map(|s| s.to_string()).collect()
}

/// Build a hierarchical topic from levels
pub fn build_topic_from_levels(levels: &[&str]) -> EventBusResult<String> {
    if levels.is_empty() {
        return Err(EventBusError::validation("Cannot build topic from empty levels".to_string()));
    }
    
    let topic = levels.join(".");
    normalize_topic(&topic)
}

/// Get parent topic
/// 
/// For "workflow.execution.completed", returns Some("workflow.execution")
/// For "workflow", returns None
pub fn get_parent_topic(topic: &str) -> Option<String> {
    let levels = extract_topic_levels(topic);
    if levels.len() <= 1 {
        None
    } else {
        Some(levels[..levels.len()-1].join("."))
    }
}

/// Get all parent topics in hierarchy
/// 
/// For "workflow.execution.completed", returns ["workflow", "workflow.execution"]
pub fn get_all_parent_topics(topic: &str) -> Vec<String> {
    let levels = extract_topic_levels(topic);
    let mut parents = Vec::new();
    
    for i in 1..levels.len() {
        let parent = levels[..i].join(".");
        parents.push(parent);
    }
    
    parents
}

/// Check if one topic is a child of another
/// 
/// "workflow.execution.completed" is a child of "workflow.execution"
pub fn is_child_topic(child: &str, parent: &str) -> bool {
    child.starts_with(parent) && child.len() > parent.len() && child.chars().nth(parent.len()) == Some('.')
}

/// Common topic patterns for event bus
pub mod patterns {
    /// System events pattern
    pub const SYSTEM: &str = "system.*";
    
    /// Workflow events pattern
    pub const WORKFLOW: &str = "workflow.*";
    
    /// User events pattern
    pub const USER: &str = "user.*";
    
    /// Error events pattern
    pub const ERROR: &str = "*.error";
    
    /// All events pattern
    pub const ALL: &str = "**";
    
    /// Status update pattern
    pub const STATUS: &str = "*.status.*";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_topic_normalization() {
        assert_eq!(normalize_topic("  Test.Topic  ").unwrap(), "test.topic");
        assert_eq!(normalize_topic("valid_topic-123").unwrap(), "valid_topic-123");
        
        assert!(normalize_topic("").is_err()); // Too short
        assert!(normalize_topic("invalid topic with spaces").is_err()); // Invalid characters
    }
    
    #[test]
    fn test_topic_validation() {
        assert!(is_valid_topic("valid.topic"));
        assert!(is_valid_topic("test_topic-123"));
        assert!(!is_valid_topic("invalid topic"));
        assert!(!is_valid_topic(""));
    }
    
    #[test]
    fn test_pattern_matching() {
        assert!(topic_matches_pattern("workflow.execution.completed", "workflow.*"));
        assert!(topic_matches_pattern("workflow.execution.completed", "workflow.execution.*"));
        assert!(topic_matches_pattern("workflow.execution.completed", "**"));
        assert!(!topic_matches_pattern("user.action", "workflow.*"));
    }
    
    #[test]
    fn test_topic_hierarchy() {
        let topic = "workflow.execution.completed";
        
        assert_eq!(extract_namespace(topic), "workflow");
        assert_eq!(extract_topic_levels(topic), vec!["workflow", "execution", "completed"]);
        assert_eq!(get_parent_topic(topic), Some("workflow.execution".to_string()));
        
        let parents = get_all_parent_topics(topic);
        assert_eq!(parents, vec!["workflow", "workflow.execution"]);
        
        assert!(is_child_topic("workflow.execution.completed", "workflow.execution"));
        assert!(!is_child_topic("workflow.execution", "workflow.execution.completed"));
    }
    
    #[test]
    fn test_topic_building() {
        let levels = vec!["workflow", "execution", "completed"];
        let topic = build_topic_from_levels(&levels).unwrap();
        assert_eq!(topic, "workflow.execution.completed");
        
        assert!(build_topic_from_levels(&[]).is_err());
    }
} 