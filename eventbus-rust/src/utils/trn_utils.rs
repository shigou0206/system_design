//! TRN (Tool Resource Name) utilities for event bus integration
//! 
//! This module provides comprehensive TRN handling capabilities including:
//! - Parsing and validation of TRN strings
//! - Comparison and matching logic
//! - AWS-style empty field handling (::)
//! - TRN-based routing and filtering

use trn_rust::{Trn, TrnBuilder};
use regex::Regex;
use once_cell::sync::Lazy;

use crate::core::{EventEnvelope, EventTriggerRule, EventBusError, EventBusResult};

/// TRN pattern matcher for event routing
#[derive(Debug, Clone)]
pub struct TrnMatcher {
    /// Compiled patterns for efficient matching
    patterns: Vec<CompiledTrnPattern>,
}

/// Compiled TRN pattern for efficient matching
#[derive(Debug, Clone)]
struct CompiledTrnPattern {
    /// Original pattern string
    pattern: String,
    
    /// Compiled regex for matching
    regex: Regex,
    
    /// Component patterns
    components: TrnPatternComponents,
}

/// TRN pattern components with wildcard support
#[derive(Debug, Clone)]
struct TrnPatternComponents {
    platform: TrnPattern,
    scope: TrnPattern,
    resource_type: TrnPattern,
    resource_id: TrnPattern,
    version: TrnPattern,
}

/// Individual component pattern
#[derive(Debug, Clone)]
enum TrnPattern {
    /// Exact match
    Exact(String),
    
    /// Wildcard match (*)
    Wildcard,
    
    /// Empty field (::)
    Empty,
    
    /// Prefix match (ends with *)
    Prefix(String),
    
    /// Suffix match (starts with *)
    Suffix(String),
    
    /// Contains match (*substring*)
    Contains(String),
}

/// TRN validation cache for performance
static TRN_VALIDATION_CACHE: Lazy<dashmap::DashMap<String, bool>> = 
    Lazy::new(|| dashmap::DashMap::new());

/// Maximum cache size to prevent memory leaks
const MAX_CACHE_SIZE: usize = 10000;

impl TrnMatcher {
    /// Create a new TRN matcher with patterns
    pub fn new(patterns: Vec<String>) -> EventBusResult<Self> {
        let mut compiled_patterns = Vec::new();
        
        for pattern in patterns {
            let compiled = Self::compile_pattern(&pattern)?;
            compiled_patterns.push(compiled);
        }
        
        Ok(Self {
            patterns: compiled_patterns,
        })
    }
    
    /// Create a single-pattern matcher
    pub fn single(pattern: &str) -> EventBusResult<Self> {
        Self::new(vec![pattern.to_string()])
    }
    
    /// Check if a TRN matches any of the patterns
    pub fn matches(&self, trn: &str) -> EventBusResult<bool> {
        // Validate TRN first
        if !is_valid_trn(trn) {
            return Err(EventBusError::validation(
                format!("Invalid TRN: {}", trn)
            ));
        }
        
        for pattern in &self.patterns {
            if pattern.regex.is_match(trn) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Get all patterns that match a TRN
    pub fn matching_patterns(&self, trn: &str) -> EventBusResult<Vec<String>> {
        let mut matches = Vec::new();
        
        if !is_valid_trn(trn) {
            return Err(EventBusError::validation(
                format!("Invalid TRN: {}", trn)
            ));
        }
        
        for pattern in &self.patterns {
            if pattern.regex.is_match(trn) {
                matches.push(pattern.pattern.clone());
            }
        }
        
        Ok(matches)
    }
    
    /// Compile a TRN pattern into a regex
    fn compile_pattern(pattern: &str) -> EventBusResult<CompiledTrnPattern> {
        let normalized = normalize_trn_pattern(pattern)?;
        let components = parse_trn_pattern(&normalized)?;
        
        // Build regex from components
        let regex_pattern = build_regex_from_components(&components)?;
        let regex = Regex::new(&regex_pattern)
            .map_err(|e| EventBusError::validation(
                format!("Invalid TRN pattern regex: {}", e)
            ))?;
        
        Ok(CompiledTrnPattern {
            pattern: pattern.to_string(),
            regex,
            components,
        })
    }
}

/// Check if an event matches a TRN-based rule
pub fn trn_matches(event: &EventEnvelope, rule: &EventTriggerRule) -> EventBusResult<bool> {
    // Check source TRN pattern if specified in match_fields
    if let Some(source_pattern_value) = rule.match_fields.get("source_trn") {
        if let Some(source_pattern) = source_pattern_value.as_str() {
            if let Some(ref source_trn) = event.source_trn {
                let matcher = TrnMatcher::single(source_pattern)?;
                if !matcher.matches(source_trn)? {
                    return Ok(false);
                }
            } else if source_pattern != "*" {
                return Ok(false); // Rule requires source TRN but event has none
            }
        }
    }
    
    // Check target TRN pattern if specified in match_fields
    if let Some(target_pattern_value) = rule.match_fields.get("target_trn") {
        if let Some(target_pattern) = target_pattern_value.as_str() {
            if let Some(ref target_trn) = event.target_trn {
                let matcher = TrnMatcher::single(target_pattern)?;
                if !matcher.matches(target_trn)? {
                    return Ok(false);
                }
            } else if target_pattern != "*" {
                return Ok(false); // Rule requires target TRN but event has none
            }
        }
    }
    
    Ok(true)
}

/// Validate a TRN string with caching
pub fn validate_trn(trn: &str) -> EventBusResult<()> {
    // Check cache first
    if let Some(is_valid) = TRN_VALIDATION_CACHE.get(trn) {
        return if *is_valid {
            Ok(())
        } else {
            Err(EventBusError::validation(format!("Invalid TRN: {}", trn)))
        };
    }
    
    // Validate using trn-rust library
    let result = Trn::parse(trn);
    let is_valid = result.is_ok();
    
    // Cache result (with size limit)
    if TRN_VALIDATION_CACHE.len() < MAX_CACHE_SIZE {
        TRN_VALIDATION_CACHE.insert(trn.to_string(), is_valid);
    }
    
    result.map(|_| ()).map_err(|e| EventBusError::validation(
        format!("Invalid TRN '{}': {}", trn, e)
    ))
}

/// Check if a TRN string is valid (returns boolean)
pub fn is_valid_trn(trn: &str) -> bool {
    validate_trn(trn).is_ok()
}

/// Normalize a TRN to standard format
pub fn normalize_trn(trn: &str) -> EventBusResult<String> {
    let parsed = Trn::parse(trn)
        .map_err(|e| EventBusError::validation(
            format!("Cannot normalize invalid TRN '{}': {}", trn, e)
        ))?;
    
    Ok(parsed.to_string())
}

/// Normalize a TRN pattern (handle empty fields with ::)
pub fn normalize_trn_pattern(pattern: &str) -> EventBusResult<String> {
    if !pattern.starts_with("trn:") {
        return Err(EventBusError::validation(
            format!("TRN pattern must start with 'trn:': {}", pattern)
        ));
    }
    
    let parts: Vec<&str> = pattern.split(':').collect();
    if parts.len() != 6 {
        return Err(EventBusError::validation(
            format!("TRN pattern must have exactly 6 parts: {}", pattern)
        ));
    }
    
    // Preserve empty fields as empty strings but ensure structure
    let normalized_parts: Vec<String> = parts.iter().enumerate().map(|(i, part)| {
        if i == 0 {
            "trn".to_string() // Always 'trn'
        } else if part.is_empty() {
            "".to_string() // Preserve empty fields
        } else {
            part.to_string()
        }
    }).collect();
    
    Ok(normalized_parts.join(":"))
}

/// Parse TRN pattern components
fn parse_trn_pattern(pattern: &str) -> EventBusResult<TrnPatternComponents> {
    let parts: Vec<&str> = pattern.split(':').collect();
    if parts.len() != 6 {
        return Err(EventBusError::validation(
            format!("Invalid TRN pattern format: {}", pattern)
        ));
    }
    
    Ok(TrnPatternComponents {
        platform: parse_component_pattern(parts[1])?,
        scope: parse_component_pattern(parts[2])?,
        resource_type: parse_component_pattern(parts[3])?,
        resource_id: parse_component_pattern(parts[4])?,
        version: parse_component_pattern(parts[5])?,
    })
}

/// Parse individual component pattern
fn parse_component_pattern(component: &str) -> EventBusResult<TrnPattern> {
    if component.is_empty() {
        Ok(TrnPattern::Empty)
    } else if component == "*" {
        Ok(TrnPattern::Wildcard)
    } else if component.starts_with('*') && component.ends_with('*') && component.len() > 2 {
        Ok(TrnPattern::Contains(component[1..component.len()-1].to_string()))
    } else if component.starts_with('*') {
        Ok(TrnPattern::Suffix(component[1..].to_string()))
    } else if component.ends_with('*') {
        Ok(TrnPattern::Prefix(component[..component.len()-1].to_string()))
    } else {
        Ok(TrnPattern::Exact(component.to_string()))
    }
}

/// Build regex pattern from TRN components
fn build_regex_from_components(components: &TrnPatternComponents) -> EventBusResult<String> {
    let platform_regex = component_to_regex(&components.platform)?;
    let scope_regex = component_to_regex(&components.scope)?;
    let resource_type_regex = component_to_regex(&components.resource_type)?;
    let resource_id_regex = component_to_regex(&components.resource_id)?;
    let version_regex = component_to_regex(&components.version)?;
    
    Ok(format!(
        "^trn:{}:{}:{}:{}:{}$",
        platform_regex, scope_regex, resource_type_regex, resource_id_regex, version_regex
    ))
}

/// Convert component pattern to regex
fn component_to_regex(pattern: &TrnPattern) -> EventBusResult<String> {
    match pattern {
        TrnPattern::Exact(s) => Ok(regex::escape(s)),
        TrnPattern::Wildcard => Ok("[^:]*".to_string()),
        TrnPattern::Empty => Ok("".to_string()),
        TrnPattern::Prefix(s) => Ok(format!("{}[^:]*", regex::escape(s))),
        TrnPattern::Suffix(s) => Ok(format!("[^:]*{}", regex::escape(s))),
        TrnPattern::Contains(s) => Ok(format!("[^:]*{}[^:]*", regex::escape(s))),
    }
}

/// Extract run ID from event correlation ID or generate one
pub fn extract_run_id(event: &EventEnvelope) -> String {
    event.correlation_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

/// Compare two TRNs for compatibility (same base, different version)
pub fn trns_compatible(trn1: &str, trn2: &str) -> EventBusResult<bool> {
    let parsed1 = Trn::parse(trn1)
        .map_err(|e| EventBusError::validation(format!("Invalid TRN1: {}", e)))?;
    let parsed2 = Trn::parse(trn2)
        .map_err(|e| EventBusError::validation(format!("Invalid TRN2: {}", e)))?;
    
    Ok(parsed1.platform() == parsed2.platform() &&
       parsed1.scope() == parsed2.scope() &&
       parsed1.resource_type() == parsed2.resource_type() &&
       parsed1.resource_id() == parsed2.resource_id())
}

/// Build a TRN from components with validation
pub fn build_trn(
    platform: &str,
    scope: &str,
    resource_type: &str,
    resource_id: &str,
    version: &str,
) -> EventBusResult<String> {
    let trn = TrnBuilder::new()
        .platform(platform)
        .scope(scope)
        .resource_type(resource_type)
        .resource_id(resource_id)
        .version(version)
        .build()
        .map_err(|e| EventBusError::validation(format!("Failed to build TRN: {}", e)))?;
    
    Ok(trn.to_string())
}

/// Parse TRN components from string
pub fn parse_trn_components(trn: &str) -> EventBusResult<TrnComponents> {
    let parsed = Trn::parse(trn)
        .map_err(|e| EventBusError::validation(format!("Invalid TRN: {}", e)))?;
    
    Ok(TrnComponents {
        platform: parsed.platform().to_string(),
        scope: parsed.scope().to_string(),
        resource_type: parsed.resource_type().to_string(),
        resource_id: parsed.resource_id().to_string(),
        version: parsed.version().to_string(),
    })
}

/// TRN components structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrnComponents {
    pub platform: String,
    pub scope: String,
    pub resource_type: String,
    pub resource_id: String,
    pub version: String,
}

/// Clear TRN validation cache
pub fn clear_trn_cache() {
    TRN_VALIDATION_CACHE.clear();
}

/// Get TRN cache statistics
pub fn get_cache_stats() -> (usize, usize) {
    (TRN_VALIDATION_CACHE.len(), MAX_CACHE_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trn_validation() {
        assert!(is_valid_trn("trn:user:alice:tool:api:v1.0"));
        assert!(!is_valid_trn("invalid-trn"));
        assert!(!is_valid_trn("trn:user:alice")); // Too few components
    }
    
    #[test]
    fn test_trn_pattern_matching() {
        let matcher = TrnMatcher::single("trn:user:*:tool:*:v1.0").unwrap();
        
        assert!(matcher.matches("trn:user:alice:tool:api:v1.0").unwrap());
        assert!(matcher.matches("trn:user:bob:tool:database:v1.0").unwrap());
        assert!(!matcher.matches("trn:org:company:tool:api:v1.0").unwrap());
        assert!(!matcher.matches("trn:user:alice:model:bert:v1.0").unwrap());
    }
    
    #[test]
    fn test_empty_field_handling() {
        // Test pattern normalization with empty fields
        let pattern = "trn:user::tool::v1.0";
        let normalized = normalize_trn_pattern(pattern).unwrap();
        assert_eq!(normalized, "trn:user::tool::v1.0");
        
        // Note: trn-rust doesn't support actual empty fields in TRNs,
        // but we can still test the pattern matching logic
        let matcher = TrnMatcher::single("trn:user:*:tool:*:v1.0").unwrap();
        assert!(matcher.matches("trn:user:alice:tool:api:v1.0").unwrap());
    }
    
    #[test]
    fn test_trn_compatibility() {
        assert!(trns_compatible(
            "trn:user:alice:tool:api:v1.0",
            "trn:user:alice:tool:api:v2.0"
        ).unwrap());
        
        assert!(!trns_compatible(
            "trn:user:alice:tool:api:v1.0",
            "trn:user:bob:tool:api:v1.0"
        ).unwrap());
    }
    
    #[test]
    fn test_component_parsing() {
        let components = parse_trn_components("trn:user:alice:tool:api:v1.0").unwrap();
        assert_eq!(components.platform, "user");
        assert_eq!(components.scope, "alice");
        assert_eq!(components.resource_type, "tool");
        assert_eq!(components.resource_id, "api");
        assert_eq!(components.version, "v1.0");
    }
} 