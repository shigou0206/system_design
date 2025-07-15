//! TRN validation functionality
//!
//! This module provides comprehensive validation of TRN strings and structures,
//! including caching, business rules, and batch validation capabilities.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::constants::*;
use crate::error::{TrnError, TrnResult};
use crate::types::Trn;

/// Validation cache for performance optimization
#[derive(Debug, Clone)]
pub struct ValidationCache {
    cache: Arc<DashMap<String, CacheEntry>>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    result: bool,
    timestamp: Instant,
}

impl ValidationCache {
    /// Create a new validation cache
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_size,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get cached validation result
    pub fn get(&self, key: &str) -> Option<bool> {
        if let Some(entry) = self.cache.get(key) {
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.result);
            } else {
                // Entry expired, remove it
                self.cache.remove(key);
            }
        }
        None
    }

    /// Insert validation result into cache
    pub fn insert(&self, key: String, result: bool) {
        // If cache is full, remove oldest entries
        if self.cache.len() >= self.max_size {
            self.cleanup_expired();
            
            if self.cache.len() >= self.max_size {
                // Remove some entries to make space
                let keys_to_remove: Vec<String> = self.cache
                    .iter()
                    .take(self.max_size / 4)
                    .map(|entry| entry.key().clone())
                    .collect();
                
                for key in keys_to_remove {
                    self.cache.remove(&key);
                }
            }
        }

        self.cache.insert(key, CacheEntry {
            result,
            timestamp: Instant::now(),
        });
    }

    /// Remove expired entries
    fn cleanup_expired(&self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self.cache
            .iter()
            .filter(|entry| now.duration_since(entry.timestamp) >= self.ttl)
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_keys {
            self.cache.remove(&key);
        }
    }

    /// Clear all cached entries
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> ValidationCacheStats {
        ValidationCacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            ttl_seconds: self.ttl.as_secs(),
        }
    }
}

/// Validation cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

/// Global validation cache instance
static VALIDATION_CACHE: once_cell::sync::Lazy<ValidationCache> = 
    once_cell::sync::Lazy::new(|| {
        ValidationCache::new(1000, 300) // 1000 entries, 5 minutes TTL
    });

/// Validation report for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Total number of TRNs validated
    pub total: usize,
    /// Number of valid TRNs
    pub valid: usize,
    /// Number of valid TRNs (alias for compatibility)
    pub success_count: usize,
    /// Number of invalid TRNs
    pub invalid: usize,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// Validation statistics
    pub stats: ValidationStats,
}

/// Individual validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// The TRN that failed validation
    pub trn: String,
    /// Error message
    pub error: String,
    /// Error category
    pub category: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

/// Configuration for TRN validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable strict validation mode
    pub strict_mode: bool,
    /// Custom validation rules
    pub custom_rules: Vec<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            cache_ttl_seconds: 300,
            strict_mode: false,
            custom_rules: Vec::new(),
        }
    }
}

/// TRN validator with thread-safe caching
#[derive(Debug, Clone)]
pub struct TrnValidator {
    cache: ValidationCache,
    config: ValidationConfig,
}

impl TrnValidator {
    /// Create a new TRN validator with default configuration
    pub fn new() -> Self {
        let config = ValidationConfig::default();
        Self {
            cache: ValidationCache::new(config.max_cache_size, config.cache_ttl_seconds),
            config,
        }
    }

    /// Create a TRN validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            cache: ValidationCache::new(config.max_cache_size, config.cache_ttl_seconds),
            config,
        }
    }

    /// Validate a TRN string
    pub fn validate(&self, trn: &str) -> TrnResult<()> {
        validate_trn_string(trn)
    }

    /// Check if a TRN is valid
    pub fn is_valid(&self, trn: &str) -> bool {
        self.validate(trn).is_ok()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> ValidationCacheStats {
        self.cache.stats()
    }

    /// Get the current configuration
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }
}

impl Default for TrnValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// TRN statistics and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrnStats {
    /// Total number of TRNs analyzed
    pub total_count: usize,
    /// Platform distribution
    pub platform_distribution: std::collections::HashMap<crate::Platform, usize>,
    /// Resource type distribution
    pub resource_type_distribution: std::collections::HashMap<crate::ResourceType, usize>,
    /// Tool type distribution (for tool resources)
    pub tool_types: std::collections::HashMap<String, usize>,
    /// Version distribution
    pub versions: std::collections::HashMap<String, usize>,
}

impl TrnStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_count: 0,
            platform_distribution: std::collections::HashMap::new(),
            resource_type_distribution: std::collections::HashMap::new(),
            tool_types: std::collections::HashMap::new(),
            versions: std::collections::HashMap::new(),
        }
    }

    /// Analyze a collection of TRNs
    pub fn analyze(trns: &[Trn]) -> Self {
        let mut stats = Self::new();
        stats.total_count = trns.len();

        for trn in trns {
            // Count platforms
            let platform = match trn.platform() {
                "user" => crate::Platform::User,
                "org" => crate::Platform::Org,
                "aiplatform" => crate::Platform::AiPlatform,
                _ => continue,
            };
            *stats.platform_distribution.entry(platform).or_insert(0) += 1;
            
            // Count resource types
            let resource_type = match trn.resource_type() {
                "tool" => crate::ResourceType::Tool,
                "model" => crate::ResourceType::Model,
                "agent" => crate::ResourceType::Agent,
                "dataset" => crate::ResourceType::Dataset,
                "workflow" => crate::ResourceType::Workflow,
                _ => continue,
            };
            *stats.resource_type_distribution.entry(resource_type).or_insert(0) += 1;
            
            // Count tool types for tool resources
            if trn.resource_type() == "tool" {
                *stats.tool_types.entry(trn.type_().to_string()).or_insert(0) += 1;
            }
            
            // Count versions
            *stats.versions.entry(trn.version().to_string()).or_insert(0) += 1;
        }

        stats
    }
}

impl Default for TrnStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Time taken for validation (milliseconds)
    pub duration_ms: u64,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Validation rate (TRNs per second)
    pub rate_per_second: f64,
}

/// Validate a TRN string
pub fn is_valid_trn(input: &str) -> bool {
    validate_trn_string(input).is_ok()
}

/// Validate a TRN string with detailed error information
pub fn validate_trn_string(input: &str) -> TrnResult<()> {
    // Check cache first
    if let Some(cached_result) = VALIDATION_CACHE.get(input) {
        return if cached_result {
            Ok(())
        } else {
            Err(TrnError::validation("TRN is invalid (cached)", "cached", Some(input.to_string())))
        };
    }

    // Perform validation
    let result = validate_trn_string_impl(input);
    
    // Cache the result
    VALIDATION_CACHE.insert(input.to_string(), result.is_ok());
    
    result
}

/// Internal validation implementation
fn validate_trn_string_impl(input: &str) -> TrnResult<()> {
    // Basic format validation
    validate_basic_format(input)?;
    
    // Length validation
    validate_length(input)?;
    
    // Component validation
    validate_components(input)?;
    
    // Business rules validation
    validate_business_rules(input)?;
    
    Ok(())
}

/// Validate TRN structure (for already parsed TRN objects)
pub fn validate_trn_struct(trn: &Trn) -> TrnResult<()> {
    let trn_string = trn.to_string();
    validate_trn_string(&trn_string)
}

/// Validate basic TRN format
fn validate_basic_format(input: &str) -> TrnResult<()> {
    if input.is_empty() {
        return Err(TrnError::format("TRN cannot be empty", None));
    }

    if !input.starts_with("trn:") {
        return Err(TrnError::format(
            "TRN must start with 'trn:' prefix",
            Some(input.to_string()),
        ));
    }

    // Use regex for comprehensive validation
    if !TRN_REGEX.is_match(input) {
        return Err(TrnError::format(
            "TRN does not match the required format",
            Some(input.to_string()),
        ));
    }

    Ok(())
}

/// Validate TRN length constraints
fn validate_length(input: &str) -> TrnResult<()> {
    let len = input.len();
    
    if len < TRN_MIN_LENGTH {
        return Err(TrnError::length(
            "TRN is too short",
            len,
            TRN_MIN_LENGTH,
            Some(input.to_string()),
        ));
    }
    
    if len > TRN_MAX_LENGTH {
        return Err(TrnError::length(
            "TRN is too long",
            len,
            TRN_MAX_LENGTH,
            Some(input.to_string()),
        ));
    }
    
    Ok(())
}

/// Validate individual TRN components
fn validate_components(input: &str) -> TrnResult<()> {
    let components = crate::parsing::parse_trn_components(input)?;
    
    // Validate platform
    validate_component(components.platform, "platform", &PLATFORM_REGEX, PLATFORM_MAX_LENGTH)?;
    
    // Validate scope (optional)
    if let Some(scope) = components.scope {
        validate_component(scope, "scope", &SCOPE_REGEX, SCOPE_MAX_LENGTH)?;
    }
    
    // Validate resource type
    validate_component(components.resource_type, "resource_type", &RESOURCE_TYPE_REGEX, RESOURCE_TYPE_MAX_LENGTH)?;
    
    // Validate type
    validate_component(components.type_, "type", &TYPE_REGEX, TYPE_MAX_LENGTH)?;
    
    // Validate subtype (optional)
    if let Some(subtype) = components.subtype {
        validate_component(subtype, "subtype", &SUBTYPE_REGEX, SUBTYPE_MAX_LENGTH)?;
    }
    
    // Validate instance ID
    validate_component(components.instance_id, "instance_id", &INSTANCE_ID_REGEX, INSTANCE_ID_MAX_LENGTH)?;
    
    // Validate instance ID represents an actionable operation
    if !crate::constants::is_actionable_instance_id(components.instance_id) {
        return Err(TrnError::validation(
            format!(
                "Instance ID '{}' must represent an actionable operation (e.g., getUserById, createUser, /users/{{id}}, POST_/users)",
                components.instance_id
            ),
            "instance_id_not_actionable".to_string(),
            Some(input.to_string()),
        ));
    }
    
    // Validate version
    validate_component(components.version, "version", &VERSION_REGEX, VERSION_MAX_LENGTH)?;
    
    // Validate tag (optional)
    if let Some(tag) = components.tag {
        validate_component(tag, "tag", &TAG_REGEX, TAG_MAX_LENGTH)?;
    }
    
    // Validate hash (optional)
    if let Some(hash) = components.hash {
        validate_hash_component(hash)?;
    }
    
    Ok(())
}

/// Validate a single component
fn validate_component(
    value: &str,
    component_name: &str,
    regex: &regex::Regex,
    max_length: usize,
) -> TrnResult<()> {
    if value.is_empty() {
        return Err(TrnError::component(
            format!("{} cannot be empty", component_name),
            component_name.to_string(),
            None,
        ));
    }
    
    if value.len() > max_length {
        return Err(TrnError::length(
            format!("{} is too long", component_name),
            value.len(),
            max_length,
            None,
        ));
    }
    
    if !regex.is_match(value) {
        return Err(TrnError::component(
            format!("{} contains invalid characters or format", component_name),
            component_name.to_string(),
            None,
        ));
    }
    
    // Check for reserved words
    if is_reserved_word(value) {
        return Err(TrnError::reserved_word(value, component_name, None));
    }
    
    Ok(())
}

/// Validate hash component
fn validate_hash_component(hash: &str) -> TrnResult<()> {
    if !HASH_FORMAT_REGEX.is_match(hash) {
        return Err(TrnError::hash(
            "Hash must be in format 'algorithm:hexvalue'",
            None,
            Some(hash.to_string()),
            None,
        ));
    }
    
    // Extract algorithm and hash value
    let parts: Vec<&str> = hash.split(':').collect();
    if parts.len() != 2 {
        return Err(TrnError::hash(
            "Hash must contain exactly one colon",
            None,
            Some(hash.to_string()),
            None,
        ));
    }
    
    let algorithm = parts[0];
    let hash_value = parts[1];
    
    // Validate algorithm
    if let Some(expected_length) = get_hash_length(algorithm) {
        if hash_value.len() != expected_length {
            return Err(TrnError::hash(
                format!("Hash value length {} does not match expected length {} for algorithm {}", 
                    hash_value.len(), expected_length, algorithm),
                Some(format!("{}", expected_length)),
                Some(format!("{}", hash_value.len())),
                None,
            ));
        }
    } else {
        return Err(TrnError::hash(
            format!("Unsupported hash algorithm: {}", algorithm),
            None,
            None,
            None,
        ));
    }
    
    // Validate hex characters
    if !hash_value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(TrnError::hash(
            "Hash value contains non-hexadecimal characters",
            None,
            Some(hash_value.to_string()),
            None,
        ));
    }
    
    Ok(())
}

/// Validate business rules
fn validate_business_rules(input: &str) -> TrnResult<()> {
    let components = crate::parsing::parse_trn_components(input)?;
    
    // Validate platform support
    if !is_platform_supported(components.platform) {
        return Err(TrnError::validation(
            format!("Platform '{}' is not supported", components.platform),
            "platform_support".to_string(),
            Some(input.to_string()),
        ));
    }
    
    // Validate resource type support
    if !is_resource_type_supported(components.resource_type) {
        return Err(TrnError::validation(
            format!("Resource type '{}' is not supported", components.resource_type),
            "resource_type_support".to_string(),
            Some(input.to_string()),
        ));
    }
    
    // For tools, validate tool type
    if components.resource_type == "tool" && !is_tool_type_supported(components.type_) {
        return Err(TrnError::validation(
            format!("Tool type '{}' is not supported", components.type_),
            "tool_type_support".to_string(),
            Some(input.to_string()),
        ));
    }
    
    // Validate scope requirements
    validate_scope_requirements(&components, input)?;
    
    // Validate version format
    validate_version_format(components.version, input)?;
    
    Ok(())
}

/// Validate scope requirements based on platform
fn validate_scope_requirements(components: &crate::types::TrnComponents<'_>, input: &str) -> TrnResult<()> {
    match components.platform {
        "user" => {
            if components.scope.is_none() {
                return Err(TrnError::validation(
                    "User platform requires a scope (username)",
                    "scope_required",
                    Some(input.to_string()),
                ));
            }
        }
        "org" => {
            if components.scope.is_none() {
                return Err(TrnError::validation(
                    "Organization platform requires a scope (organization name)",
                    "scope_required",
                    Some(input.to_string()),
                ));
            }
        }
        "aiplatform" => {
            if components.scope.is_some() {
                return Err(TrnError::validation(
                    "AI platform should not have a scope",
                    "scope_forbidden",
                    Some(input.to_string()),
                ));
            }
        }
        _ => {
            // Custom platforms may or may not require scope
        }
    }
    
    Ok(())
}

/// Validate version format
fn validate_version_format(version: &str, input: &str) -> TrnResult<()> {
    // Check if it's a common alias
    if is_common_alias(version) {
        return Ok(());
    }
    
    // Check semantic versioning
    if SEMANTIC_VERSION_REGEX.is_match(version) {
        return Ok(());
    }
    
    // Check major.minor versioning
    if MAJOR_MINOR_VERSION_REGEX.is_match(version) {
        return Ok(());
    }
    
    // Check major version only
    if MAJOR_VERSION_REGEX.is_match(version) {
        return Ok(());
    }
    
    // Generic version pattern
    if VERSION_REGEX.is_match(version) {
        return Ok(());
    }
    
    Err(TrnError::validation(
        format!("Version '{}' does not match any supported format", version),
        "version_format".to_string(),
        Some(input.to_string()),
    ))
}

/// Batch validate multiple TRNs
pub fn batch_validate(trns: &[String]) -> ValidationReport {
    let start_time = Instant::now();
    let mut valid = 0;
    let mut invalid = 0;
    let mut errors = Vec::new();
    let mut cache_hits = 0;
    let mut cache_misses = 0;

    for trn in trns {
        // Check cache first
        let was_cached = VALIDATION_CACHE.get(trn).is_some();
        
        match validate_trn_string(trn) {
            Ok(()) => {
                valid += 1;
                if was_cached {
                    cache_hits += 1;
                } else {
                    cache_misses += 1;
                }
            }
            Err(e) => {
                invalid += 1;
                cache_misses += 1;
                
                let suggestion = suggest_fix(trn, &e);
                errors.push(ValidationError {
                    trn: trn.clone(),
                    error: e.to_string(),
                    category: get_error_category(&e),
                    suggestion,
                });
            }
        }
    }

    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let rate_per_second = if duration.as_secs_f64() > 0.0 {
        trns.len() as f64 / duration.as_secs_f64()
    } else {
        0.0
    };

    ValidationReport {
        total: trns.len(),
        valid,
        success_count: valid,
        invalid,
        errors,
        stats: ValidationStats {
            duration_ms,
            cache_hits,
            cache_misses,
            rate_per_second,
        },
    }
}

/// Get error category for reporting
fn get_error_category(error: &TrnError) -> String {
    match error {
        TrnError::Format { .. } => "Format".to_string(),
        TrnError::Validation { .. } => "Validation".to_string(),
        TrnError::Component { .. } => "Component".to_string(),
        TrnError::Length { .. } => "Length".to_string(),
        TrnError::Character { .. } => "Character".to_string(),
        TrnError::ReservedWord { .. } => "ReservedWord".to_string(),
        TrnError::Hash { .. } => "Hash".to_string(),
        _ => "Other".to_string(),
    }
}

/// Suggest a fix for a validation error
fn suggest_fix(trn: &str, error: &TrnError) -> Option<String> {
    match error {
        TrnError::Character { .. } => {
            Some("Remove or replace invalid characters with alphanumeric characters or hyphens".to_string())
        }
        TrnError::Length { max_length, .. } => {
            Some(format!("Reduce length to {} characters or less", max_length))
        }
        TrnError::Format { .. } => {
            if !trn.starts_with("trn:") {
                Some("Add 'trn:' prefix".to_string())
            } else {
                Some("Check TRN format: trn:platform[:scope]:resource_type:type[:subtype]:instance_id:version[:tag][@hash]".to_string())
            }
        }
        TrnError::ReservedWord { reserved_word, .. } => {
            Some(format!("Replace reserved word '{}' with a different value", reserved_word))
        }
        _ => None,
    }
}

/// Normalize a TRN string for validation
pub fn normalize_trn(input: &str) -> String {
    crate::parsing::normalize_trn(input).unwrap_or_else(|_| input.to_string())
}

/// Clear validation cache
#[allow(dead_code)]
pub fn clear_validation_cache() {
    VALIDATION_CACHE.clear();
}

/// Get validation cache statistics
#[allow(dead_code)]
pub fn get_validation_cache_stats() -> ValidationCacheStats {
    VALIDATION_CACHE.stats()
}

/// Validate identifier format (alphanumeric, hyphen, underscore)
pub fn is_valid_identifier(identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }
    
    INSTANCE_ID_REGEX.is_match(identifier)
}

/// Validate scope format
pub fn is_valid_scope(scope: &str) -> bool {
    if scope.is_empty() {
        return false;
    }
    
    SCOPE_REGEX.is_match(scope)
}

/// Validate version format
pub fn is_valid_version(version: &str) -> bool {
    if version.is_empty() {
        return false;
    }
    
    // Check semantic version, simple version, or special versions
    VERSION_REGEX.is_match(version) || 
    version == "latest" || 
    version == "*" ||
    SEMANTIC_VERSION_REGEX.is_match(version)
}

/// Validate instance ID format and ensure it represents an actionable operation
pub fn is_valid_instance_id(instance_id: &str) -> bool {
    if instance_id.is_empty() {
        return false;
    }
    
    // Check basic pattern match
    if !INSTANCE_ID_REGEX.is_match(instance_id) {
        return false;
    }
    
    // Check if it represents an actionable operation
    crate::constants::is_actionable_instance_id(instance_id)
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn test_valid_trn() {
        assert!(is_valid_trn("trn:user:alice:tool:openapi:github-api:v1.0"));
        assert!(is_valid_trn("trn:aiplatform:tool:workflow:data-pipeline:latest"));
        assert!(is_valid_trn("trn:org:company:tool:openapi:async:api:v2.0:stable@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    }

    #[test]
    fn test_invalid_trn() {
        assert!(!is_valid_trn(""));
        assert!(!is_valid_trn("invalid"));
        assert!(!is_valid_trn("trn:user:tool"));
        assert!(!is_valid_trn("trn:INVALID:tool:openapi:api:v1.0"));
    }

    #[test]
    fn test_component_validation() {
        // Too long platform
        assert!(!is_valid_trn(&format!("trn:{}:tool:openapi:api:v1.0", "a".repeat(50))));
        
        // Reserved word
        assert!(!is_valid_trn("trn:system:tool:openapi:api:v1.0"));
        
        // Invalid characters
        assert!(!is_valid_trn("trn:user@invalid:tool:openapi:api:v1.0"));
    }

    #[test]
    fn test_business_rules() {
        // User platform without scope
        assert!(!is_valid_trn("trn:user:tool:openapi:api:v1.0"));
        
        // AI platform with scope
        assert!(!is_valid_trn("trn:aiplatform:scope:tool:openapi:api:v1.0"));
    }

    #[test]
    fn test_hash_validation() {
        assert!(is_valid_trn("trn:user:alice:tool:openapi:api:v1.0@sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"));
        assert!(!is_valid_trn("trn:user:alice:tool:openapi:api:v1.0@invalid:hash"));
        assert!(!is_valid_trn("trn:user:alice:tool:openapi:api:v1.0@sha256:xyz"));
    }

    #[test]
    fn test_cache() {
        let cache = ValidationCache::new(10, 60);
        
        // Miss
        assert_eq!(cache.get("test"), None);
        
        // Insert and hit
        cache.insert("test".to_string(), true);
        assert_eq!(cache.get("test"), Some(true));
        
        // Stats
        let stats = cache.stats();
        assert_eq!(stats.max_size, 10);
        assert_eq!(stats.ttl_seconds, 60);
    }

    #[test]
    fn test_batch_validation() {
        let trns = vec![
            "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
            "trn:invalid:format".to_string(),
            "trn:user:bob:tool:python:processor:v2.0".to_string(),
        ];
        
        let report = batch_validate(&trns);
        
        assert_eq!(report.total, 3);
        assert_eq!(report.valid, 2);
        assert_eq!(report.invalid, 1);
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_normalization() {
        let input = "TRN:USER:Alice:TOOL:OpenAPI:API:V1.0";
        let normalized = normalize_trn(input);
        assert_eq!(normalized, "trn:user:alice:tool:openapi:api:v1.0");
    }
} 