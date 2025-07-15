//! Constants and configuration for TRN operations
//!
//! This module defines all constants, patterns, and configuration values
//! used throughout the TRN library.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

// TRN Format Constants
/// TRN prefix that all TRNs must start with
pub const TRN_PREFIX: &str = "trn";

/// Separator character between TRN components
pub const TRN_SEPARATOR: char = ':';

/// Hash separator character
pub const TRN_HASH_SEPARATOR: char = '@';

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

/// Maximum type length
pub const TYPE_MAX_LENGTH: usize = 32;

/// Maximum subtype length
pub const SUBTYPE_MAX_LENGTH: usize = 32;

/// Maximum instance ID length
pub const INSTANCE_ID_MAX_LENGTH: usize = 64;

/// Maximum version length
pub const VERSION_MAX_LENGTH: usize = 32;

/// Maximum tag length
pub const TAG_MAX_LENGTH: usize = 16;

/// Maximum hash length
#[allow(dead_code)]
pub const HASH_MAX_LENGTH: usize = 71;

/// Fixed number of components in TRN (excluding hash)
pub const TRN_FIXED_COMPONENT_COUNT: usize = 9; // trn:platform:scope:resource_type:type:subtype:instance_id:version:tag

// Character Set Patterns (RFC3986 compatible)
/// Pattern for platform component
pub const PLATFORM_PATTERN: &str = r"[a-z][a-z0-9-]{1,31}";

/// Pattern for scope component (or empty string)
pub const SCOPE_PATTERN: &str = r"[a-z0-9][a-z0-9-]{0,31}|";

/// Pattern for resource type component
pub const RESOURCE_TYPE_PATTERN: &str = r"[a-z][a-z0-9-]{1,15}";

/// Pattern for type component
pub const TYPE_PATTERN: &str = r"[a-z][a-z0-9-]{1,31}";

/// Pattern for subtype component (or empty string)
pub const SUBTYPE_PATTERN: &str = r"[a-z][a-z0-9-]{1,31}|";

/// Pattern for instance ID component (actionable operations like getUserById, createUser, /users/{id}, POST_/users, etc.)
pub const INSTANCE_ID_PATTERN: &str = r"[a-z][a-zA-Z0-9_/-]{0,63}";

// Validation patterns for actionable instance IDs
/// Common actionable operation patterns (CRUD operations)
pub static ACTIONABLE_OPERATION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Method-style operations: getUserById, createUser, updateUser, deleteUser
        Regex::new(r"^(get|create|update|delete|list|find|search|fetch)[A-Z][a-zA-Z0-9]*$").unwrap(),
        // REST-style paths: /users/{id}, /users, /auth/login
        Regex::new(r"^/?[a-z][a-zA-Z0-9_/-]*$").unwrap(),
        // HTTP method + path: GET_/users, POST_/users
        Regex::new(r"^(GET|POST|PUT|DELETE|PATCH)_/?[a-z][a-zA-Z0-9_/-]*$").unwrap(),
        // RPC-style operations: user.get, auth.login, data.export
        Regex::new(r"^[a-z][a-zA-Z0-9]*\.[a-z][a-zA-Z0-9]*$").unwrap(),
    ]
});

/// Check if an instance ID represents an actionable operation
pub fn is_actionable_instance_id(instance_id: &str) -> bool {
    if instance_id.is_empty() {
        return false;
    }
    
    // Check against actionable patterns
    ACTIONABLE_OPERATION_PATTERNS.iter().any(|pattern| pattern.is_match(instance_id))
}

/// Pattern for version component
pub const VERSION_PATTERN: &str = r"[a-z0-9][a-z0-9.-]{0,31}";

/// Pattern for tag component (or empty string)
pub const TAG_PATTERN: &str = r"[a-z0-9][a-z0-9-]{0,15}|";

/// Pattern for hash component
pub const HASH_PATTERN: &str = r"[a-z0-9:]{8,71}";

// Complete TRN Regex Pattern - Fixed Structure Format
/// Compiled regex for complete TRN validation with fixed structure
pub static TRN_REGEX: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(
        r"^trn:({platform}):({scope}):({resource_type}):({type}):({subtype}):({instance_id}):({version}):({tag})(?:@({hash}))?$",
        platform = PLATFORM_PATTERN,
        scope = SCOPE_PATTERN,
        resource_type = RESOURCE_TYPE_PATTERN,
        type = TYPE_PATTERN,
        subtype = SUBTYPE_PATTERN,
        instance_id = INSTANCE_ID_PATTERN,
        version = VERSION_PATTERN,
        tag = TAG_PATTERN,
        hash = HASH_PATTERN,
    );
    Regex::new(&pattern).unwrap()
});

// URL Pattern for TRN URLs - Fixed Structure Format
/// Compiled regex for TRN URL validation with fixed structure
#[allow(dead_code)]
pub static TRN_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(
        r"^trn://({platform})/({scope})/({resource_type})/({type})/({subtype})/({instance_id})/({version})/({tag})(?:\?hash=({hash}))?$",
        platform = PLATFORM_PATTERN,
        scope = SCOPE_PATTERN,
        resource_type = RESOURCE_TYPE_PATTERN,
        type = TYPE_PATTERN,
        subtype = SUBTYPE_PATTERN,
        instance_id = INSTANCE_ID_PATTERN,
        version = VERSION_PATTERN,
        tag = TAG_PATTERN,
        hash = HASH_PATTERN,
    );
    Regex::new(&pattern).unwrap()
});

// Component validation regexes
/// Platform validation regex
pub static PLATFORM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", PLATFORM_PATTERN)).unwrap()
});

/// Scope validation regex
pub static SCOPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", SCOPE_PATTERN)).unwrap()
});

/// Resource type validation regex
pub static RESOURCE_TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", RESOURCE_TYPE_PATTERN)).unwrap()
});

/// Type validation regex
pub static TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", TYPE_PATTERN)).unwrap()
});

/// Subtype validation regex
pub static SUBTYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", SUBTYPE_PATTERN)).unwrap()
});

/// Instance ID validation regex
pub static INSTANCE_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", INSTANCE_ID_PATTERN)).unwrap()
});

/// Version validation regex
pub static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", VERSION_PATTERN)).unwrap()
});

/// Tag validation regex
#[allow(dead_code)]
pub static TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", TAG_PATTERN)).unwrap()
});

/// Hash validation regex
#[allow(dead_code)]
pub static HASH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!("^{}$", HASH_PATTERN)).unwrap()
});

// Supported Platforms
/// Set of supported platform identifiers
pub static SUPPORTED_PLATFORMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("aiplatform"); // System platform
    set.insert("user");       // User platform
    set.insert("org");        // Organization platform
    set
});

// Supported Resource Types
/// Set of supported resource types
pub static SUPPORTED_RESOURCE_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("tool");     // Executable tools
    set.insert("dataset");  // Data resources
    set.insert("pipeline"); // Workflow templates
    set.insert("model");    // AI model resources
    set
});

// Supported Tool Types
/// Set of supported tool types
pub static SUPPORTED_TOOL_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("openapi");   // RESTful API tools
    set.insert("workflow");  // Business process tools
    set.insert("python");    // Python execution tools
    set.insert("shell");     // Shell command tools
    set.insert("system");    // System operation tools
    set.insert("async_api"); // Async/Event-driven API tools
    set
});

// Supported Tool Subtypes
/// Set of supported tool subtypes
#[allow(dead_code)]
pub static SUPPORTED_TOOL_SUBTYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("async");     // Asynchronous execution
    set.insert("streaming"); // Streaming data processing
    set.insert("batch");     // Batch processing
    set.insert("sync");      // Synchronous execution
    set.insert("realtime");  // Real-time processing
    set
});

// Reserved Words (cannot be used in any component)
/// Set of reserved words that cannot be used in TRN components
pub static RESERVED_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("__internal__");
    set.insert("__system__");
    set.insert("__admin__");
    set.insert("__test__");
    set.insert("system");
    set.insert("internal");
    set.insert("admin");
    set.insert("root");
    set.insert("super");
    set.insert("null");
    set.insert("undefined");
    set.insert("reserved");
    set
});

// Common Version Aliases
/// Set of common version aliases
pub static COMMON_ALIASES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("latest");
    set.insert("stable");
    set.insert("beta");
    set.insert("alpha");
    set.insert("dev");
    set.insert("experimental");
    set.insert("lts"); // Long Term Support
    set.insert("rc");  // Release Candidate
    set
});

// Hash Algorithm Mapping
/// Hash algorithm name to hex length mapping
pub static HASH_ALGORITHMS: Lazy<std::collections::HashMap<&'static str, usize>> =
    Lazy::new(|| {
        let mut map = std::collections::HashMap::new();
        map.insert("md5", 32);    // MD5 hex length
        map.insert("sha1", 40);   // SHA1 hex length
        map.insert("sha256", 64); // SHA256 hex length
        map.insert("sha512", 128); // SHA512 hex length
        map.insert("crc32", 8);   // CRC32 hex length
        map
    });

// Supported Hash Format Pattern
/// Regex for validating hash format
pub static HASH_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(md5|sha1|sha256|sha512|crc32):[a-f0-9]+$").unwrap()
});

// Version Pattern for semantic versioning
/// Regex for semantic version validation
pub static SEMANTIC_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^v?(\d+)\.(\d+)\.(\d+)(?:-([a-z0-9\-\.]+))?(?:\+([a-z0-9\-\.]+))?$").unwrap()
});

/// Regex for major.minor version
pub static MAJOR_MINOR_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^v?(\d+)\.(\d+)(?:-([a-z0-9\-]+))?$").unwrap()
});

/// Regex for major version only
pub static MAJOR_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^v?(\d+)(?:-([a-z0-9\-]+))?$").unwrap()
});

// URL Encoding Characters
/// Characters that need URL encoding in TRN components
#[allow(dead_code)]
pub static URL_ENCODING_CHARS: Lazy<std::collections::HashMap<char, &'static str>> =
    Lazy::new(|| {
        let mut map = std::collections::HashMap::new();
        map.insert(':', "%3A");
        map.insert('/', "%2F");
        map.insert('.', "%2E");
        map.insert('@', "%40");
        map.insert('#', "%23");
        map.insert('?', "%3F");
        map.insert('&', "%26");
        map.insert('=', "%3D");
        map.insert('+', "%2B");
        map.insert(' ', "%20");
        map
    });

// Default Configuration
/// Default configuration values
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrnConfig {
    /// Whether to validate TRNs on creation
    pub validate_on_create: bool,
    /// Whether to use strict validation rules
    pub strict_validation: bool,
    /// Whether to allow uppercase characters (converts to lowercase)
    pub allow_uppercase: bool,
    /// Whether to normalize TRNs on parsing
    pub normalize_on_parse: bool,
    /// Whether to cache validation results
    pub cache_validation_results: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Whether to enable deprecation warnings
    pub enable_deprecation_warnings: bool,
    /// Whether to verify hashes
    pub hash_verification: bool,
    /// Whether to resolve aliases
    pub alias_resolution: bool,
    /// Whether to check permissions
    pub permission_check: bool,
}

impl Default for TrnConfig {
    fn default() -> Self {
        Self {
            validate_on_create: true,
            strict_validation: true,
            allow_uppercase: false,
            normalize_on_parse: true,
            cache_validation_results: true,
            max_cache_size: 1000,
            enable_deprecation_warnings: true,
            hash_verification: true,
            alias_resolution: true,
            permission_check: false, // Disabled by default
        }
    }
}

// Component Display Names for error messages
/// Human-readable component names for error messages
#[allow(dead_code)]
pub static COMPONENT_NAMES: Lazy<std::collections::HashMap<&'static str, &'static str>> =
    Lazy::new(|| {
        let mut map = std::collections::HashMap::new();
        map.insert("platform", "Platform");
        map.insert("scope", "Scope");
        map.insert("resource_type", "Resource Type");
        map.insert("type", "Type");
        map.insert("subtype", "Subtype");
        map.insert("instance_id", "Instance ID");
        map.insert("version", "Version");
        map.insert("tag", "Tag");
        map.insert("hash", "Hash");
        map
    });

// Permission Actions
/// Set of valid permission actions
#[allow(dead_code)]
pub static PERMISSION_ACTIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("READ");
    set.insert("WRITE");
    set.insert("EXECUTE");
    set.insert("DELETE");
    set.insert("ADMIN");
    set.insert("ALL");
    set
});

// Version Comparison Operators
/// Set of valid version comparison operators
#[allow(dead_code)]
pub static VERSION_OPERATORS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("=="); // Equal
    set.insert("!="); // Not equal
    set.insert(">");  // Greater than
    set.insert(">="); // Greater than or equal
    set.insert("<");  // Less than
    set.insert("<="); // Less than or equal
    set.insert("~");  // Compatible version
    set.insert("^");  // Compatible within major version
    set
});

// HTTP Status Code Mapping for TRN Errors
/// HTTP status codes for TRN error types
#[allow(dead_code)]
pub static HTTP_STATUS_MAP: Lazy<std::collections::HashMap<i32, u16>> = Lazy::new(|| {
    let mut map = std::collections::HashMap::new();
    map.insert(-32000, 400); // Bad Request - Format Error
    map.insert(-32001, 400); // Bad Request - Validation Error
    map.insert(-32002, 400); // Bad Request - Length/Character Error
    map.insert(-32003, 400); // Bad Request - Hash/Alias Error
    map.insert(-32020, 403); // Forbidden - Permission Error
    map.insert(-32030, 404); // Not Found - Resource Not Found
    map.insert(-32031, 409); // Conflict - Resource Conflict
    map
});

// TTL values in seconds
/// Time-to-live values for various caches
#[allow(dead_code)]
pub static TTL_VALUES: Lazy<std::collections::HashMap<&'static str, u64>> = Lazy::new(|| {
    let mut map = std::collections::HashMap::new();
    map.insert("validation_cache", 300);   // 5 minutes
    map.insert("alias_resolution", 600);   // 10 minutes
    map.insert("permission_cache", 1800);  // 30 minutes
    map.insert("hash_verification", 3600); // 1 hour
    map
});

/// Check if a platform is supported
pub fn is_platform_supported(platform: &str) -> bool {
    SUPPORTED_PLATFORMS.contains(platform)
}

/// Check if a resource type is supported
pub fn is_resource_type_supported(resource_type: &str) -> bool {
    SUPPORTED_RESOURCE_TYPES.contains(resource_type)
}

/// Check if a tool type is supported
pub fn is_tool_type_supported(tool_type: &str) -> bool {
    SUPPORTED_TOOL_TYPES.contains(tool_type)
}

/// Check if a word is reserved
pub fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.contains(word) || word.starts_with("__")
}

/// Check if a version is a common alias
pub fn is_common_alias(version: &str) -> bool {
    COMMON_ALIASES.contains(version)
}

/// Get the expected hash length for an algorithm
pub fn get_hash_length(algorithm: &str) -> Option<usize> {
    HASH_ALGORITHMS.get(algorithm).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trn_regex() {
        let trn = "trn:user:alice:tool:openapi::getUserById:v1.0:";
        assert!(TRN_REGEX.is_match(trn));
        
        let invalid_trn = "invalid:trn:format";
        assert!(!TRN_REGEX.is_match(invalid_trn));
    }

    #[test]
    fn test_platform_support() {
        assert!(is_platform_supported("user"));
        assert!(is_platform_supported("org"));
        assert!(is_platform_supported("aiplatform"));
        assert!(!is_platform_supported("invalid"));
    }

    #[test]
    fn test_reserved_words() {
        assert!(is_reserved_word("system"));
        assert!(is_reserved_word("__internal__"));
        assert!(!is_reserved_word("valid-name"));
    }

    #[test]
    fn test_version_aliases() {
        assert!(is_common_alias("latest"));
        assert!(is_common_alias("stable"));
        assert!(!is_common_alias("v1.0"));
    }

    #[test]
    fn test_hash_algorithms() {
        assert_eq!(get_hash_length("sha256"), Some(64));
        assert_eq!(get_hash_length("md5"), Some(32));
        assert_eq!(get_hash_length("invalid"), None);
    }

    #[test]
    fn test_semantic_version_regex() {
        assert!(SEMANTIC_VERSION_REGEX.is_match("1.2.3"));
        assert!(SEMANTIC_VERSION_REGEX.is_match("v1.2.3"));
        assert!(SEMANTIC_VERSION_REGEX.is_match("1.2.3-beta"));
        assert!(SEMANTIC_VERSION_REGEX.is_match("1.2.3+build"));
        assert!(!SEMANTIC_VERSION_REGEX.is_match("1.2"));
    }

    #[test]
    fn test_default_config() {
        let config = TrnConfig::default();
        assert!(config.validate_on_create);
        assert!(config.strict_validation);
        assert_eq!(config.max_cache_size, 1000);
    }
} 