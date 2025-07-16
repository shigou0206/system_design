//! Constants and configuration for TRN operations
//!
//! This module defines all constants, patterns, and configuration values
//! used throughout the TRN library for the simplified 6-component format.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

// TRN Format Constants
/// TRN prefix that all TRNs must start with
#[allow(dead_code)]
pub const TRN_PREFIX: &str = "trn";

/// Separator character between TRN components
#[allow(dead_code)]
pub const TRN_SEPARATOR: char = ':';

/// Maximum total TRN length
pub const TRN_MAX_LENGTH: usize = 256;

/// Minimum total TRN length
pub const TRN_MIN_LENGTH: usize = 10;

// Component Length Limits
/// Maximum platform name length
pub const PLATFORM_MAX_LENGTH: usize = 32;

/// Maximum scope length
pub const SCOPE_MAX_LENGTH: usize = 32;

/// Maximum resource type length
pub const RESOURCE_TYPE_MAX_LENGTH: usize = 16;

/// Maximum resource ID length
pub const RESOURCE_ID_MAX_LENGTH: usize = 64;

/// Maximum version length
pub const VERSION_MAX_LENGTH: usize = 32;

/// Fixed number of components in TRN
pub const TRN_FIXED_COMPONENT_COUNT: usize = 6; // trn:platform:scope:resource_type:resource_id:version

// Character Set Patterns (RFC3986 compatible)
/// Pattern for platform component
pub const PLATFORM_PATTERN: &str = r"[a-zA-Z][a-zA-Z0-9-]{1,31}";

/// Pattern for scope component (required)
pub const SCOPE_PATTERN: &str = r"[a-zA-Z0-9][a-zA-Z0-9_-]{0,31}";

/// Pattern for resource type component
pub const RESOURCE_TYPE_PATTERN: &str = r"[a-zA-Z][a-zA-Z0-9_-]{1,15}";

/// Pattern for resource ID component
pub const RESOURCE_ID_PATTERN: &str = r"[a-zA-Z0-9][a-zA-Z0-9_.-]{0,63}";

/// Pattern for version component
pub const VERSION_PATTERN: &str = r"[a-zA-Z0-9][a-zA-Z0-9.-]{0,31}";

// Complete TRN Regex Pattern - Simplified 6-Component Format
/// Compiled regex for complete TRN validation with simplified structure
pub static TRN_REGEX: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(
        r"^trn:({platform}):({scope}):({resource_type}):({resource_id}):({version})$",
        platform = PLATFORM_PATTERN,
        scope = SCOPE_PATTERN,
        resource_type = RESOURCE_TYPE_PATTERN,
        resource_id = RESOURCE_ID_PATTERN,
        version = VERSION_PATTERN,
    );
    Regex::new(&pattern).unwrap()
});

// Individual component regex patterns
/// Compiled regex for platform validation
pub static PLATFORM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"^{}$", PLATFORM_PATTERN)).unwrap()
});

/// Compiled regex for scope validation
pub static SCOPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"^{}$", SCOPE_PATTERN)).unwrap()
});

/// Compiled regex for resource type validation
pub static RESOURCE_TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"^{}$", RESOURCE_TYPE_PATTERN)).unwrap()
});

/// Compiled regex for resource ID validation
pub static RESOURCE_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"^{}$", RESOURCE_ID_PATTERN)).unwrap()
});

/// Compiled regex for version validation
pub static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(r"^{}$", VERSION_PATTERN)).unwrap()
});

// Reserved words that cannot be used in components
/// Set of reserved words that cannot be used as platform names
pub static RESERVED_PLATFORMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "trn", "null", "undefined", "void",
    ].iter().copied().collect()
});

/// Set of reserved words that cannot be used as scope names
pub static RESERVED_SCOPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "trn", "null", "undefined", "void",
    ].iter().copied().collect()
});

/// Set of reserved words that cannot be used as resource type names
pub static RESERVED_RESOURCE_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "trn", "null", "undefined", "void",
    ].iter().copied().collect()
});

/// Set of reserved words that cannot be used as resource ID names
pub static RESERVED_RESOURCE_IDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "trn", "null", "undefined", "void",
    ].iter().copied().collect()
});

/// Set of reserved words that cannot be used as version names
pub static RESERVED_VERSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "trn", "null", "undefined", "void",
    ].iter().copied().collect()
});

// Valid platform values
/// Set of valid platform identifiers
#[allow(dead_code)]
pub static VALID_PLATFORMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "user", "org", "aiplatform"
    ].iter().copied().collect()
});

// Valid resource types
/// Set of valid resource type identifiers
#[allow(dead_code)]
pub static VALID_RESOURCE_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "tool", "model", "dataset", "pipeline", "workflow", "service",
        "api", "schema", "template", "config", "plugin", "extension",
        "library", "framework", "runtime", "environment", "container",
        "image", "script", "function", "lambda", "microservice",
        "component", "module", "package", "bundle", "archive",
        "custom-type", "other"
    ].iter().copied().collect()
});

// Semantic versioning pattern
/// Regex pattern for semantic version validation
#[allow(dead_code)]
pub static SEMANTIC_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap()
});

// Performance and operational constants
/// Cache size for validation operations
#[allow(dead_code)]
pub const VALIDATION_CACHE_SIZE: usize = 1000;

/// Cache TTL in seconds
#[allow(dead_code)]
pub const VALIDATION_CACHE_TTL_SECONDS: u64 = 300;

/// Library version
#[allow(dead_code)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// TRN specification version
#[allow(dead_code)]
pub const TRN_SPEC_VERSION: &str = "2.0";

// Test constants
#[cfg(test)]
#[allow(dead_code)]
pub const TEST_PLATFORM: &str = "user";

#[cfg(test)]
#[allow(dead_code)]
pub const TEST_SCOPE: &str = "testuser";

#[cfg(test)]
#[allow(dead_code)]
pub const TEST_RESOURCE_TYPE: &str = "tool";

#[cfg(test)]
#[allow(dead_code)]
pub const TEST_RESOURCE_ID: &str = "testresource";

#[cfg(test)]
#[allow(dead_code)]
pub const TEST_VERSION: &str = "v1.0";

#[cfg(test)]
#[allow(dead_code)]
pub const SAMPLE_TRN: &str = "trn:user:testuser:tool:testresource:v1.0"; 