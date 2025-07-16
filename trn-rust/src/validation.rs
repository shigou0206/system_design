//! TRN validation functionality
//!
//! This module provides comprehensive validation of TRN strings and structures
//! for the simplified 6-component format, including caching and business rules.

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
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> ValidationCacheStats {
        ValidationCacheStats {
            total_entries: self.cache.len(),
            hit_rate: 0.0, // Would need additional tracking
        }
    }
}

/// Statistics for validation cache performance
pub struct ValidationCacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Cache hit rate as a percentage (0.0 to 1.0)
    pub hit_rate: f64,
}

/// Global validation cache instance
static VALIDATION_CACHE: once_cell::sync::Lazy<ValidationCache> = once_cell::sync::Lazy::new(|| {
    ValidationCache::new(VALIDATION_CACHE_SIZE, VALIDATION_CACHE_TTL_SECONDS)
});

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
            Err(TrnError::validation("TRN is invalid (cached)".to_string(), "cached".to_string(), Some(input.to_string())))
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
        return Err(TrnError::format(
            "TRN component cannot be empty".to_string(),
            Some(input.to_string()),
        ));
    }

    if !input.starts_with("trn:") {
        return Err(TrnError::format(
            "TRN must start with 'trn:'".to_string(),
            Some(input.to_string()),
        ));
    }

    // Check component count
    let component_count = input.matches(':').count();
    if component_count != 5 {
        return Err(TrnError::format(
            "TRN must have exactly 6 components".to_string(),
            Some(input.to_string()),
        ));
    }

    // Basic regex validation
    if !TRN_REGEX.is_match(input) {
        return Err(TrnError::format(
            "Invalid TRN format. Expected: trn:platform:scope:resource_type:resource_id:version".to_string(),
            Some(input.to_string()),
        ));
    }

    Ok(())
}

/// Validate TRN length
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
    
    // Validate scope (required in simplified format)
    validate_component(components.scope, "scope", &SCOPE_REGEX, SCOPE_MAX_LENGTH)?;
    
    // Validate resource type
    validate_component(components.resource_type, "resource_type", &RESOURCE_TYPE_REGEX, RESOURCE_TYPE_MAX_LENGTH)?;
    
    // Validate resource ID
    validate_component(components.resource_id, "resource_id", &RESOURCE_ID_REGEX, RESOURCE_ID_MAX_LENGTH)?;
    
    // Validate version
    validate_component(components.version, "version", &VERSION_REGEX, VERSION_MAX_LENGTH)?;
    
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
    if RESERVED_PLATFORMS.contains(value) || 
       RESERVED_SCOPES.contains(value) || 
       RESERVED_RESOURCE_TYPES.contains(value) ||
       RESERVED_RESOURCE_IDS.contains(value) ||
       RESERVED_VERSIONS.contains(value) {
        return Err(TrnError::validation(
            format!("{} '{}' is a reserved word", component_name, value),
            "reserved_word".to_string(),
            None,
        ));
    }
    
    Ok(())
}

/// Validate business rules for simplified format
fn validate_business_rules(input: &str) -> TrnResult<()> {
    let components = crate::parsing::parse_trn_components(input)?;
    
    // Validate resource type support
    if !VALID_RESOURCE_TYPES.contains(components.resource_type) {
        return Err(TrnError::validation(
            format!("Resource type '{}' is not supported", components.resource_type),
            "resource_type_support".to_string(),
            Some(input.to_string()),
        ));
    }
    
    // Validate scope requirements based on platform
    validate_scope_requirements(&components, input)?;
    
    // Validate version format
    validate_version_format(components.version, input)?;
    
    Ok(())
}

/// Validate scope requirements based on platform (simplified for new format)
fn validate_scope_requirements(components: &crate::types::TrnComponents<'_>, input: &str) -> TrnResult<()> {
    // In the simplified format, scope is always required but check platform-specific rules
    if components.scope.is_empty() {
        return Err(TrnError::validation(
            "Scope cannot be empty in simplified TRN format".to_string(),
            "scope_required".to_string(),
            Some(input.to_string()),
        ));
    }

    // Platform-specific scope validation
    match components.platform {
        "user" => {
            // User scope should be a valid username
            if components.scope.len() < 2 || components.scope.len() > 32 {
                                 return Err(TrnError::validation(
                     "User scope must be between 2 and 32 characters".to_string(),
                     "scope_length".to_string(),
                     Some(input.to_string()),
                 ));
            }
        }
        "org" => {
            // Organization scope should be a valid organization name
            if components.scope.len() < 2 || components.scope.len() > 32 {
                                 return Err(TrnError::validation(
                     "Organization scope must be between 2 and 32 characters".to_string(),
                     "scope_length".to_string(),
                     Some(input.to_string()),
                 ));
            }
        }
        "aiplatform" => {
            // AI platform typically uses system-level scopes
            if components.scope.len() > 32 {
                                 return Err(TrnError::validation(
                     "AI platform scope must not exceed 32 characters".to_string(),
                     "scope_length".to_string(),
                     Some(input.to_string()),
                 ));
            }
        }
        _ => {
            // Custom platforms have flexible scope requirements
        }
    }
    
    Ok(())
}

/// Validate version format
fn validate_version_format(version: &str, input: &str) -> TrnResult<()> {
    // Check if it's a common version alias
    let common_aliases = ["latest", "stable", "beta", "alpha", "dev", "main", "master"];
    if common_aliases.contains(&version) {
        return Ok(());
    }
    
    // Check basic version pattern (already validated by regex, but add semantic checks)
    if version.is_empty() {
                 return Err(TrnError::validation(
             "Version cannot be empty".to_string(),
             "version_empty".to_string(),
             Some(input.to_string()),
         ));
    }

    // Additional version format validation can be added here
    Ok(())
}

/// Batch validation of multiple TRNs
pub fn validate_multiple_trns(trns: &[String]) -> Vec<TrnResult<()>> {
    trns.iter()
        .map(|trn| validate_trn_string(trn))
        .collect()
}

/// Validation report for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Total number of TRNs validated
    pub total: usize,
    /// Number of valid TRNs
    pub valid: usize,
    /// Number of invalid TRNs
    pub invalid: usize,
    /// Validation errors
    pub errors: Vec<String>,
    /// Performance statistics
    pub stats: ValidationStats,
}

/// Generate a validation report for multiple TRNs
pub fn generate_validation_report(trns: &[String]) -> ValidationReport {
    let start_time = std::time::Instant::now();
    let results = validate_multiple_trns(trns);
    let duration = start_time.elapsed();
    
    let total = results.len();
    let valid = results.iter().filter(|r| r.is_ok()).count();
    let invalid = total - valid;
    
    let errors: Vec<String> = results
        .iter()
        .filter_map(|r| r.as_ref().err().map(|e| e.to_string()))
        .collect();
    
    let stats = ValidationStats {
        duration_ms: duration.as_millis() as u64,
        cache_hits: 0, // Would need additional tracking
        cache_misses: 0, // Would need additional tracking
        rate_per_second: if duration.as_secs_f64() > 0.0 {
            total as f64 / duration.as_secs_f64()
        } else {
            0.0
        },
    };
    
    ValidationReport {
        total,
        valid,
        invalid,
        errors,
        stats,
    }
}

/// Check if TRN components are well-formed
pub fn check_component_format(components: &crate::types::TrnComponents<'_>) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Check platform format
    if !PLATFORM_REGEX.is_match(components.platform) {
        issues.push(format!("Platform '{}' has invalid format", components.platform));
    }
    
    // Check scope format
    if !SCOPE_REGEX.is_match(components.scope) {
        issues.push(format!("Scope '{}' has invalid format", components.scope));
    }
    
    // Check resource type format
    if !RESOURCE_TYPE_REGEX.is_match(components.resource_type) {
        issues.push(format!("Resource type '{}' has invalid format", components.resource_type));
    }
    
    // Check resource ID format
    if !RESOURCE_ID_REGEX.is_match(components.resource_id) {
        issues.push(format!("Resource ID '{}' has invalid format", components.resource_id));
    }
    
    // Check version format
    if !VERSION_REGEX.is_match(components.version) {
        issues.push(format!("Version '{}' has invalid format", components.version));
    }
    
    issues
}

/// Validate TRN compliance with naming conventions
pub fn validate_naming_conventions(_trn: &Trn) -> TrnResult<()> {
    // We now allow both uppercase and lowercase in platform, scope, and resource type
    // Only enforce basic format validation through regex patterns
    
    // Platforms, scopes, and resource types can be uppercase or lowercase
    // The regex validation handles character restrictions
    
    Ok(())
}

/// Performance validation for high-throughput scenarios
pub fn validate_performance_batch(trns: &[String], max_duration_ms: u64) -> ValidationReport {
    let mut report = generate_validation_report(trns);
    
    if report.stats.duration_ms > max_duration_ms {
        report.errors.push(format!(
            "Validation took {}ms, exceeding limit of {}ms",
            report.stats.duration_ms, max_duration_ms
        ));
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_trn() {
        assert!(validate_trn_string("trn:user:alice:tool:myapi:v1.0").is_ok());
        assert!(validate_trn_string("trn:org:company:model:bert:v2.1").is_ok());
        assert!(validate_trn_string("trn:aiplatform:system:dataset:training:latest").is_ok());
    }

    #[test]
    fn test_invalid_trn_format() {
        assert!(validate_trn_string("invalid:format").is_err());
        assert!(validate_trn_string("trn:too:few").is_err());
        assert!(validate_trn_string("trn:too:many:components:here:now:extra").is_err());
    }

    #[test]
    fn test_empty_components() {
        assert!(validate_trn_string("trn::alice:tool:myapi:v1.0").is_err()); // Empty platform
        assert!(validate_trn_string("trn:user::tool:myapi:v1.0").is_err()); // Empty scope
        assert!(validate_trn_string("trn:user:alice::myapi:v1.0").is_err()); // Empty resource type
        assert!(validate_trn_string("trn:user:alice:tool::v1.0").is_err()); // Empty resource ID
        assert!(validate_trn_string("trn:user:alice:tool:myapi:").is_err()); // Empty version
    }

    #[test]
    fn test_reserved_words() {
        // Test reserved platforms
        assert!(validate_trn_string("trn:trn:alice:tool:myapi:v1.0").is_err());
        assert!(validate_trn_string("trn:null:alice:tool:myapi:v1.0").is_err());
        assert!(validate_trn_string("trn:void:alice:tool:myapi:v1.0").is_err());
        
        // Test reserved scopes
        assert!(validate_trn_string("trn:user:null:tool:myapi:v1.0").is_err());
        assert!(validate_trn_string("trn:user:undefined:tool:myapi:v1.0").is_err());
    }

    #[test]
    fn test_validation_cache() {
        let cache = ValidationCache::new(100, 60);
        
        cache.insert("test_trn".to_string(), true);
        assert_eq!(cache.get("test_trn"), Some(true));
        
        cache.insert("invalid_trn".to_string(), false);
        assert_eq!(cache.get("invalid_trn"), Some(false));
        
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_batch_validation() {
        let trns = vec![
            "trn:user:alice:tool:myapi:v1.0".to_string(),
            "invalid:format".to_string(),
            "trn:org:company:model:bert:v2.1".to_string(),
        ];
        
        let report = generate_validation_report(&trns);
        assert_eq!(report.total, 3);
        assert_eq!(report.valid, 2);
        assert_eq!(report.invalid, 1);
    }

    #[test]
    fn test_naming_conventions() {
        let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
        assert!(validate_naming_conventions(&trn).is_ok());
        
        let trn = Trn::new("USER", "alice", "tool", "myapi", "v1.0").unwrap();
        assert!(validate_naming_conventions(&trn).is_ok());
    }
} 