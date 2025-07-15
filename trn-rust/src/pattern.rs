//! TRN pattern matching and filtering
//!
//! This module provides pattern matching capabilities for TRN strings,
//! including wildcard matching, filtering, and advanced pattern operations.

use regex::Regex;
use std::collections::HashMap;

use crate::constants::*;
use crate::error::{TrnError, TrnResult};

/// Pattern matcher for TRN strings
#[derive(Debug, Clone)]
pub struct TrnMatcher {
    patterns: Vec<CompiledPattern>,
}

/// Compiled pattern for efficient matching
#[derive(Debug, Clone)]
struct CompiledPattern {
    original: String,
    regex: Regex,
    #[allow(dead_code)]
    components: PatternComponents,
}

/// Pattern components with wildcards
#[derive(Debug, Clone)]
struct PatternComponents {
    platform: Option<String>,
    scope: Option<String>,
    resource_type: Option<String>,
    type_: Option<String>,
    subtype: Option<String>,
    instance_id: Option<String>,
    version: Option<String>,
    tag: Option<String>,
    hash: Option<String>,
}

impl TrnMatcher {
    /// Create a new TRN matcher with a pattern
    pub fn new(pattern: &str) -> TrnResult<Self> {
        let mut matcher = Self {
            patterns: Vec::new(),
        };
        matcher.add_pattern(pattern)?;
        Ok(matcher)
    }

    /// Create an empty TRN matcher
    pub fn empty() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Create a new TRN matcher with a pattern
    pub fn with_pattern(pattern: &str) -> TrnResult<Self> {
        Self::new(pattern)
    }

    /// Add a pattern to the matcher
    pub fn add_pattern(&mut self, pattern: &str) -> TrnResult<()> {
        let compiled = compile_pattern(pattern)?;
        self.patterns.push(compiled);
        Ok(())
    }

    /// Check if a TRN matches any pattern
    pub fn matches(&self, trn: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.regex.is_match(trn))
    }

    /// Check if a TRN matches a specific pattern by index
    pub fn matches_pattern(&self, trn: &str, pattern_index: usize) -> bool {
        if let Some(pattern) = self.patterns.get(pattern_index) {
            pattern.regex.is_match(trn)
        } else {
            false
        }
    }

    /// Get all patterns that match a TRN
    pub fn matching_patterns(&self, trn: &str) -> Vec<&str> {
        self.patterns
            .iter()
            .filter(|pattern| pattern.regex.is_match(trn))
            .map(|pattern| pattern.original.as_str())
            .collect()
    }

    /// Filter TRNs by patterns
    pub fn filter_trns<'a>(&self, trns: &'a [String]) -> Vec<&'a String> {
        trns.iter()
            .filter(|trn| self.matches(trn))
            .collect()
    }

    /// Get pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Clear all patterns
    pub fn clear(&mut self) {
        self.patterns.clear();
    }
}

impl Default for TrnMatcher {
    fn default() -> Self {
        Self::empty()
    }
}

/// Check if a TRN matches a pattern
pub fn matches_pattern(trn: &str, pattern: &str) -> bool {
    match compile_pattern(pattern) {
        Ok(compiled) => compiled.regex.is_match(trn),
        Err(_) => false,
    }
}

/// Find TRNs matching a pattern
pub fn find_matching_trns<'a>(trns: &'a [String], pattern: &str) -> Vec<&'a String> {
    match compile_pattern(pattern) {
        Ok(compiled) => trns
            .iter()
            .filter(|trn| compiled.regex.is_match(trn))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Compile a pattern into a regex
fn compile_pattern(pattern: &str) -> TrnResult<CompiledPattern> {
    // Parse pattern components
    let components = parse_pattern_components(pattern)?;
    
    // Build regex pattern
    let regex_pattern = build_regex_pattern(&components)?;
    
    // Compile regex
    let regex = Regex::new(&regex_pattern)
        .map_err(|e| TrnError::pattern(
            format!("Failed to compile pattern regex: {}", e),
            pattern.to_string(),
        ))?;
    
    Ok(CompiledPattern {
        original: pattern.to_string(),
        regex,
        components,
    })
}

/// Parse pattern into components
fn parse_pattern_components(pattern: &str) -> TrnResult<PatternComponents> {
    if !pattern.starts_with("trn:") {
        return Err(TrnError::pattern(
            "Pattern must start with 'trn:'",
            pattern,
        ));
    }
    
    // Split by colons, handling hash separator
    let (main_part, hash_part) = if let Some(hash_pos) = pattern.find('@') {
        (&pattern[..hash_pos], Some(&pattern[hash_pos + 1..]))
    } else {
        (pattern, None)
    };
    
    let parts: Vec<&str> = main_part.split(':').collect();
    
    if parts.len() < 6 {
        return Err(TrnError::pattern(
            "Pattern must have at least 6 components",
            pattern,
        ));
    }
    
    // Helper function to convert component to pattern
    let to_pattern = |s: &str| -> Option<String> {
        if s == "*" || s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    };
    
    let mut components = PatternComponents {
        platform: to_pattern(parts[1]),
        scope: None,
        resource_type: None,
        type_: None,
        subtype: None,
        instance_id: None,
        version: None,
        tag: None,
        hash: hash_part.and_then(|h| to_pattern(h)),
    };
    
    // Parse based on number of components
    match parts.len() {
        6 => {
            // trn:platform:resource_type:type:instance_id:version
            components.resource_type = to_pattern(parts[2]);
            components.type_ = to_pattern(parts[3]);
            components.instance_id = to_pattern(parts[4]);
            components.version = to_pattern(parts[5]);
        }
        7 => {
            // Could be with scope or subtype - use heuristics
            if is_scope_pattern(parts[2], parts[1]) {
                // trn:platform:scope:resource_type:type:instance_id:version
                components.scope = to_pattern(parts[2]);
                components.resource_type = to_pattern(parts[3]);
                components.type_ = to_pattern(parts[4]);
                components.instance_id = to_pattern(parts[5]);
                components.version = to_pattern(parts[6]);
            } else {
                // trn:platform:resource_type:type:subtype:instance_id:version
                components.resource_type = to_pattern(parts[2]);
                components.type_ = to_pattern(parts[3]);
                components.subtype = to_pattern(parts[4]);
                components.instance_id = to_pattern(parts[5]);
                components.version = to_pattern(parts[6]);
            }
        }
        8 => {
            // trn:platform:scope:resource_type:type:subtype:instance_id:version
            components.scope = to_pattern(parts[2]);
            components.resource_type = to_pattern(parts[3]);
            components.type_ = to_pattern(parts[4]);
            components.subtype = to_pattern(parts[5]);
            components.instance_id = to_pattern(parts[6]);
            components.version = to_pattern(parts[7]);
        }
        9 => {
            // trn:platform:scope:resource_type:type:subtype:instance_id:version:tag
            components.scope = to_pattern(parts[2]);
            components.resource_type = to_pattern(parts[3]);
            components.type_ = to_pattern(parts[4]);
            components.subtype = to_pattern(parts[5]);
            components.instance_id = to_pattern(parts[6]);
            components.version = to_pattern(parts[7]);
            components.tag = to_pattern(parts[8]);
        }
        _ => {
            return Err(TrnError::pattern(
                "Pattern has too many components",
                pattern,
            ));
        }
    }
    
    Ok(components)
}

/// Check if a component looks like a scope pattern
fn is_scope_pattern(component: &str, platform: &str) -> bool {
    if component == "*" {
        return true;
    }
    
    match platform {
        "user" | "org" => true,
        "aiplatform" => false,
        "*" => component.len() <= 32,
        _ => component.len() <= 32,
    }
}

/// Build regex pattern from components
fn build_regex_pattern(components: &PatternComponents) -> TrnResult<String> {
    let mut pattern = String::from("^trn:");
    
    // Platform
    if let Some(platform) = &components.platform {
        pattern.push_str(&escape_pattern_component(platform));
    } else {
        pattern.push_str(PLATFORM_PATTERN);
    }
    
    // Scope (optional)
    if let Some(scope) = &components.scope {
        pattern.push(':');
        pattern.push_str(&escape_pattern_component(scope));
    } else {
        pattern.push_str("(?:");
        pattern.push(':');
        pattern.push_str(SCOPE_PATTERN);
        pattern.push_str(")?");
    }
    
    // Resource type
    pattern.push(':');
    if let Some(resource_type) = &components.resource_type {
        pattern.push_str(&escape_pattern_component(resource_type));
    } else {
        pattern.push_str(RESOURCE_TYPE_PATTERN);
    }
    
    // Type
    pattern.push(':');
    if let Some(type_) = &components.type_ {
        pattern.push_str(&escape_pattern_component(type_));
    } else {
        pattern.push_str(TYPE_PATTERN);
    }
    
    // Subtype (optional)
    if let Some(subtype) = &components.subtype {
        pattern.push(':');
        pattern.push_str(&escape_pattern_component(subtype));
    } else {
        pattern.push_str("(?:");
        pattern.push(':');
        pattern.push_str(SUBTYPE_PATTERN);
        pattern.push_str(")?");
    }
    
    // Instance ID
    pattern.push(':');
    if let Some(instance_id) = &components.instance_id {
        pattern.push_str(&escape_pattern_component(instance_id));
    } else {
        pattern.push_str(INSTANCE_ID_PATTERN);
    }
    
    // Version
    pattern.push(':');
    if let Some(version) = &components.version {
        pattern.push_str(&escape_pattern_component(version));
    } else {
        pattern.push_str(VERSION_PATTERN);
    }
    
    // Tag (optional)
    if let Some(tag) = &components.tag {
        pattern.push(':');
        pattern.push_str(&escape_pattern_component(tag));
    } else {
        pattern.push_str("(?:");
        pattern.push(':');
        pattern.push_str(TAG_PATTERN);
        pattern.push_str(")?");
    }
    
    // Hash (optional)
    if let Some(hash) = &components.hash {
        pattern.push('@');
        pattern.push_str(&escape_pattern_component(hash));
    } else {
        pattern.push_str("(?:");
        pattern.push('@');
        pattern.push_str(HASH_PATTERN);
        pattern.push_str(")?");
    }
    
    pattern.push('$');
    
    Ok(pattern)
}

/// Escape special regex characters in pattern components
fn escape_pattern_component(component: &str) -> String {
    // Handle wildcards
    if component == "*" {
        return "[^:@]+".to_string();
    }
    
    if component.contains('*') {
        // Replace * with regex pattern
        let escaped = regex::escape(component);
        return escaped.replace(r"\*", "[^:@]*");
    }
    
    regex::escape(component)
}

/// Advanced pattern matching with multiple conditions
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdvancedMatcher {
    conditions: Vec<MatchCondition>,
}

/// Match condition for advanced matching
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MatchCondition {
    /// Exact match
    Exact(String),
    /// Pattern match with wildcards
    Pattern(String),
    /// Platform filter
    Platform(Vec<String>),
    /// Resource type filter
    ResourceType(Vec<String>),
    /// Version range
    VersionRange { min: Option<String>, max: Option<String> },
    /// Has tag
    HasTag,
    /// Has hash
    HasHash,
    /// Custom function
    Custom(fn(&str) -> bool),
}

#[allow(dead_code)]
impl AdvancedMatcher {
    /// Create a new advanced matcher
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    /// Add a condition
    pub fn add_condition(mut self, condition: MatchCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Check if a TRN matches all conditions
    pub fn matches(&self, trn: &str) -> bool {
        // Try to parse TRN first
        let trn_obj = match crate::types::Trn::parse(trn) {
            Ok(t) => t,
            Err(_) => return false,
        };

        self.conditions.iter().all(|condition| {
            match condition {
                MatchCondition::Exact(pattern) => trn == pattern,
                MatchCondition::Pattern(pattern) => matches_pattern(trn, pattern),
                MatchCondition::Platform(platforms) => {
                    platforms.contains(&trn_obj.platform().to_string())
                }
                MatchCondition::ResourceType(types) => {
                    types.contains(&trn_obj.resource_type().to_string())
                }
                MatchCondition::VersionRange { min, max } => {
                    let version = trn_obj.version();
                    let mut matches = true;
                    
                    if let Some(min_ver) = min {
                        matches = matches && compare_versions(version, min_ver, ">=");
                    }
                    
                    if let Some(max_ver) = max {
                        matches = matches && compare_versions(version, max_ver, "<=");
                    }
                    
                    matches
                }
                MatchCondition::HasTag => trn_obj.tag().is_some(),
                MatchCondition::HasHash => trn_obj.hash().is_some(),
                MatchCondition::Custom(func) => func(trn),
            }
        })
    }

    /// Filter TRNs by all conditions
    pub fn filter_trns<'a>(&self, trns: &'a [String]) -> Vec<&'a String> {
        trns.iter()
            .filter(|trn| self.matches(trn))
            .collect()
    }
}

impl Default for AdvancedMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple version comparison (basic semantic versioning)
fn compare_versions(v1: &str, v2: &str, op: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    
    let ver1 = parse_version(v1);
    let ver2 = parse_version(v2);
    
    let cmp = ver1.cmp(&ver2);
    
    match op {
        "==" => cmp == std::cmp::Ordering::Equal,
        "!=" => cmp != std::cmp::Ordering::Equal,
        ">" => cmp == std::cmp::Ordering::Greater,
        ">=" => cmp != std::cmp::Ordering::Less,
        "<" => cmp == std::cmp::Ordering::Less,
        "<=" => cmp != std::cmp::Ordering::Greater,
        _ => false,
    }
}

/// Pattern statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PatternStats {
    /// Total patterns
    pub total_patterns: usize,
    /// Patterns with wildcards
    pub wildcard_patterns: usize,
    /// Most common platforms
    pub common_platforms: HashMap<String, usize>,
    /// Most common resource types
    pub common_resource_types: HashMap<String, usize>,
}

/// Analyze patterns for statistics
#[allow(dead_code)]
pub fn analyze_patterns(patterns: &[String]) -> PatternStats {
    let mut stats = PatternStats {
        total_patterns: patterns.len(),
        wildcard_patterns: 0,
        common_platforms: HashMap::new(),
        common_resource_types: HashMap::new(),
    };
    
    for pattern in patterns {
        if pattern.contains('*') {
            stats.wildcard_patterns += 1;
        }
        
        // Extract platform and resource type for statistics
        if let Ok(components) = parse_pattern_components(pattern) {
            if let Some(platform) = &components.platform {
                *stats.common_platforms.entry(platform.clone()).or_insert(0) += 1;
            }
            
            if let Some(resource_type) = &components.resource_type {
                *stats.common_resource_types.entry(resource_type.clone()).or_insert(0) += 1;
            }
        }
    }
    
    stats
}

/// Common pattern templates
#[allow(dead_code)]
pub struct PatternTemplates;

#[allow(dead_code)]
impl PatternTemplates {
    /// All user tools
    pub fn user_tools() -> &'static str {
        "trn:user:*:tool:*:*:*"
    }
    
    /// All organization tools
    pub fn org_tools() -> &'static str {
        "trn:org:*:tool:*:*:*"
    }
    
    /// All system tools
    pub fn system_tools() -> &'static str {
        "trn:aiplatform:tool:*:*:*"
    }
    
    /// All OpenAPI tools
    pub fn openapi_tools() -> &'static str {
        "trn:*:tool:openapi:*:*"
    }
    
    /// All Python tools
    pub fn python_tools() -> &'static str {
        "trn:*:tool:python:*:*"
    }
    
    /// All datasets
    pub fn datasets() -> &'static str {
        "trn:*:dataset:*:*:*"
    }
    
    /// All models
    pub fn models() -> &'static str {
        "trn:*:model:*:*:*"
    }
    
    /// Latest versions only
    pub fn latest_versions() -> &'static str {
        "trn:*:*:*:*:latest"
    }
    
    /// Stable versions only
    pub fn stable_versions() -> &'static str {
        "trn:*:*:*:*:stable"
    }
    
    /// Tools with hashes
    pub fn tools_with_hash() -> &'static str {
        "trn:*:tool:*:*:*@*"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern_matching() {
        assert!(matches_pattern(
            "trn:user:alice:tool:openapi:github-api:v1.0",
            "trn:user:alice:tool:openapi:*:*"
        ));
        
        assert!(matches_pattern(
            "trn:user:alice:tool:openapi:github-api:v1.0",
            "trn:user:*:tool:*:*:*"
        ));
        
        assert!(!matches_pattern(
            "trn:user:alice:tool:openapi:github-api:v1.0",
            "trn:org:*:tool:*:*:*"
        ));
    }

    #[test]
    fn test_find_matching_trns() {
        let trns = vec![
            "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
            "trn:user:alice:tool:python:script:v2.0".to_string(),
            "trn:user:bob:tool:openapi:slack-api:v1.5".to_string(),
            "trn:org:company:tool:workflow:pipeline:latest".to_string(),
        ];

        let alice_tools = find_matching_trns(&trns, "trn:user:alice:tool:*:*:*");
        assert_eq!(alice_tools.len(), 2);

        let openapi_tools = find_matching_trns(&trns, "trn:*:*:tool:openapi:*:*");
        assert_eq!(openapi_tools.len(), 2);
    }

    #[test]
    fn test_trn_matcher() {
        let mut matcher = TrnMatcher::empty();
        matcher.add_pattern("trn:user:*:tool:*:*:*").unwrap();
        matcher.add_pattern("trn:org:*:tool:*:*:*").unwrap();

        assert!(matcher.matches("trn:user:alice:tool:openapi:api:v1.0"));
        assert!(matcher.matches("trn:org:company:tool:workflow:pipeline:latest"));
        assert!(!matcher.matches("trn:aiplatform:tool:system:backup:v1.0"));

        assert_eq!(matcher.pattern_count(), 2);
    }

    #[test]
    fn test_advanced_matcher() {
        let matcher = AdvancedMatcher::new()
            .add_condition(MatchCondition::Platform(vec!["user".to_string()]))
            .add_condition(MatchCondition::ResourceType(vec!["tool".to_string()]))
            .add_condition(MatchCondition::HasHash);

        assert!(!matcher.matches("trn:user:alice:tool:openapi:api:v1.0"));
        assert!(matcher.matches("trn:user:alice:tool:openapi:api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    }

    #[test]
    fn test_version_comparison() {
        assert!(compare_versions("v1.2.3", "v1.2.0", ">"));
        assert!(compare_versions("v1.2.3", "v1.2.3", "=="));
        assert!(compare_versions("v1.2.0", "v1.2.3", "<"));
        assert!(compare_versions("v2.0.0", "v1.9.9", ">"));
    }

    #[test]
    fn test_pattern_templates() {
        let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
        
        assert!(matches_pattern(trn, PatternTemplates::user_tools()));
        assert!(matches_pattern(trn, PatternTemplates::openapi_tools()));
        assert!(!matches_pattern(trn, PatternTemplates::python_tools()));
    }

    #[test]
    fn test_pattern_with_hash() {
        let trn = "trn:user:alice:tool:openapi:api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(matches_pattern(trn, "trn:*:*:*:*:*:*@*"));
        assert!(matches_pattern(trn, "trn:user:alice:tool:*:*:*@sha256:*"));
    }

    #[test]
    fn test_pattern_analysis() {
        let patterns = vec![
            "trn:user:*:tool:*:*:*".to_string(),
            "trn:user:alice:tool:openapi:*:*".to_string(),
            "trn:org:*:tool:*:*:*".to_string(),
        ];

        let stats = analyze_patterns(&patterns);
        assert_eq!(stats.total_patterns, 3);
        assert_eq!(stats.wildcard_patterns, 3);
        assert_eq!(stats.common_platforms.get("user"), Some(&2));
        assert_eq!(stats.common_resource_types.get("tool"), Some(&3));
    }
} 

/// Validate a pattern string
#[allow(dead_code)]
pub fn validate_pattern(pattern: &str) -> TrnResult<()> {
    // Check basic pattern format
    if !pattern.starts_with("trn:") {
        return Err(TrnError::format("Pattern must start with 'trn:'", Some(pattern.to_string())));
    }
    
    let parts: Vec<&str> = pattern.split(':').collect();
    if parts.len() < 7 {
        return Err(TrnError::format("Pattern must have at least 7 components", Some(pattern.to_string())));
    }
    
    // Validate platform
    let platform = parts[1];
    if platform != "*" && !["user", "org", "aiplatform"].contains(&platform) {
        return Err(TrnError::format("Invalid platform in pattern", Some(pattern.to_string())));
    }
    
    Ok(())
}



/// TRN filter for advanced filtering operations
#[allow(dead_code)]
pub struct TrnFilter {
    platform: Option<String>,
    resource_type: Option<String>,
    scope: Option<String>,
    tool_type: Option<String>,
    version_pattern: Option<String>,
}

#[allow(dead_code)]
impl TrnFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self {
            platform: None,
            resource_type: None,
            scope: None,
            tool_type: None,
            version_pattern: None,
        }
    }
    
    /// Filter by platform
    pub fn platform(mut self, platform: &str) -> Self {
        self.platform = Some(platform.to_string());
        self
    }
    
    /// Filter by resource type
    pub fn resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }
    
    /// Filter by scope
    pub fn scope(mut self, scope: &str) -> Self {
        self.scope = Some(scope.to_string());
        self
    }
    
    /// Filter by tool type
    pub fn tool_type(mut self, tool_type: &str) -> Self {
        self.tool_type = Some(tool_type.to_string());
        self
    }
    
    /// Filter by version pattern
    pub fn version_pattern(mut self, pattern: &str) -> Self {
        self.version_pattern = Some(pattern.to_string());
        self
    }
    
    /// Apply filter to a TRN
    pub fn matches(&self, trn: &crate::types::Trn) -> bool {
        if let Some(ref platform) = self.platform {
            if trn.platform() != platform {
                return false;
            }
        }
        
        if let Some(ref resource_type) = self.resource_type {
            if trn.resource_type() != resource_type {
                return false;
            }
        }
        
        if let Some(ref scope) = self.scope {
            if let Some(trn_scope) = trn.scope() {
                if trn_scope != scope {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        if let Some(ref tool_type) = self.tool_type {
            if trn.type_() != tool_type {
                return false;
            }
        }
        
        if let Some(ref version_pattern) = self.version_pattern {
            let version_regex = regex::Regex::new(version_pattern).unwrap_or_else(|_| {
                regex::Regex::new(&regex::escape(version_pattern)).unwrap()
            });
            if !version_regex.is_match(trn.version()) {
                return false;
            }
        }
        
        true
    }
} 