use trn_rust::*;

// =================================
// Basic Parsing Tests (using public API)
// =================================

#[test]
fn test_trn_parse_basic() {
    // Test basic TRN parsing (6 components)
    let trn = Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").unwrap();
    assert_eq!(trn.platform(), "aiplatform");
    assert_eq!(trn.scope(), None);
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.type_(), "bert");
    assert_eq!(trn.subtype(), None);
    assert_eq!(trn.instance_id(), "base-model");
    assert_eq!(trn.version(), "v1.0");
    assert_eq!(trn.tag(), None);
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_trn_parse_with_scope() {
    // Test TRN with scope (7 components)
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.subtype(), None);
    assert_eq!(trn.instance_id(), "github-api");
    assert_eq!(trn.version(), "v1.0");
    assert_eq!(trn.tag(), None);
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_trn_parse_with_organization() {
    // Test organization TRN
    let trn = Trn::parse("trn:org:company:model:bert:language-model:v2.0").unwrap();
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("company"));
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.type_(), "bert");
    assert_eq!(trn.instance_id(), "language-model");
    assert_eq!(trn.version(), "v2.0");
}

#[test]
fn test_trn_parse_with_hash() {
    // Test TRN with hash
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.hash(), Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_trn_parse_invalid_format() {
    // Test invalid TRN formats
    assert!(Trn::parse("invalid-trn").is_err());
    assert!(Trn::parse("trn:user").is_err()); // Too few components
    assert!(Trn::parse("not-trn:user:alice:tool:openapi:github-api:v1.0").is_err()); // Wrong prefix
    assert!(Trn::parse("").is_err()); // Empty string
    assert!(Trn::parse("trn::tool:openapi:github-api:v1.0").is_err()); // Empty platform
}

#[test]
fn test_parse_different_platforms() {
    // Test different platform types
    let trn1 = Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").unwrap();
    assert_eq!(trn1.platform(), "aiplatform");
    
    let trn2 = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert_eq!(trn2.platform(), "user");
    
    let trn3 = Trn::parse("trn:org:company:tool:python:data-processor:v1.0").unwrap();
    assert_eq!(trn3.platform(), "org");
}

#[test]
fn test_parse_different_resource_types() {
    // Test different resource types
    let datasets = Trn::parse("trn:org:company:dataset:csv:sales-data:v1.0").unwrap();
    assert_eq!(datasets.resource_type(), "dataset");
    
    let models = Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").unwrap();
    assert_eq!(models.resource_type(), "model");
    
    let tools = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert_eq!(tools.resource_type(), "tool");
}

// =================================
// TrnComponents Tests (using public methods)
// =================================

#[test]
fn test_parse_trn_components_basic() {
    // Test components through parse and then components() method
    let trn = Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").unwrap();
    let components = trn.components();
    assert_eq!(components.platform, "aiplatform");
    assert_eq!(components.scope, None);
    assert_eq!(components.resource_type, "model");
    assert_eq!(components.type_, "bert");
    assert_eq!(components.subtype, None);
    assert_eq!(components.instance_id, "base-model");
    assert_eq!(components.version, "v1.0");
    assert_eq!(components.tag, None);
    assert_eq!(components.hash, None);
}

#[test]
fn test_parse_trn_components_with_scope() {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    let components = trn.components();
    assert_eq!(components.platform, "user");
    assert_eq!(components.scope, Some("alice"));
    assert_eq!(components.resource_type, "tool");
    assert_eq!(components.type_, "openapi");
    assert_eq!(components.instance_id, "github-api");
    assert_eq!(components.version, "v1.0");
}

#[test]
fn test_parse_trn_components_with_hash() {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let components = trn.components();
    assert_eq!(components.hash, Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    assert_eq!(components.version, "v1.0");
}

// =================================
// Format Validation Tests
// =================================

#[test]
fn test_trn_format_validation() {
    // Test through parsing - valid TRNs should parse successfully
    assert!(Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").is_ok());
    assert!(Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").is_ok());
    assert!(Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").is_ok());
    
    // Invalid formats should fail
    assert!(Trn::parse("invalid-trn").is_err());
    assert!(Trn::parse("not-trn:user:alice:tool:openapi:github-api:v1.0").is_err());
    assert!(Trn::parse("trn:user").is_err());
    assert!(Trn::parse("").is_err());
}

// =================================
// Normalization Tests
// =================================

#[test]
fn test_normalize_trn() {
    // Test basic normalization - normalize_trn from validation module returns String
    let normalized = normalize_trn("trn:aiplatform:model:bert:base-model:v1.0");
    assert_eq!(normalized, "trn:aiplatform:model:bert:base-model:v1.0");
    
    // Test normalization with scope
    let normalized_scope = normalize_trn("trn:user:alice:tool:openapi:github-api:v1.0");
    assert_eq!(normalized_scope, "trn:user:alice:tool:openapi:github-api:v1.0");
}

#[test]
fn test_normalize_trn_invalid() {
    // normalize_trn from validation returns original string on error
    let result = normalize_trn("invalid-trn");
    assert_eq!(result, "invalid-trn"); // Returns original string when invalid
    
    let empty_result = normalize_trn("");
    assert_eq!(empty_result, ""); // Returns original string when empty
}

// =================================
// Base TRN Tests
// =================================

#[test]
fn test_base_trn_functionality() {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    
    // Test base_trn method
    let base_trn = trn.base_trn();
    assert_eq!(base_trn.platform(), "user");
    assert_eq!(base_trn.scope(), Some("alice"));
    assert_eq!(base_trn.resource_type(), "tool");
    assert_eq!(base_trn.type_(), "openapi");
    assert_eq!(base_trn.instance_id(), "github-api");
    assert_eq!(base_trn.version(), "*");
    assert_eq!(base_trn.tag(), None);
    assert_eq!(base_trn.hash(), None);
}

// =================================
// URL Parsing Tests (through public API)
// =================================

#[test]
fn test_parse_trn_from_url() {
    // Test TRN URL parsing through public url_to_trn function
    let trn = url_to_trn("trn://user/alice/tool/openapi/github-api/v1.0").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "github-api");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_parse_trn_from_url_invalid() {
    // Test invalid URL formats
    assert!(url_to_trn("http://example.com").is_err());
    assert!(url_to_trn("invalid-url").is_err());
    assert!(url_to_trn("").is_err());
}

// =================================
// Edge Cases and Error Handling
// =================================

#[test]
fn test_parse_edge_cases() {
    // Test TRN with special characters in components (use org platform to allow dashes in scope)
    let trn = Trn::parse("trn:org:user-123:tool:openapi:api-v2:v1.0").unwrap();
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("user-123"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "api-v2");
}

#[test]
fn test_parse_version_formats() {
    // Test different version formats
    let trn1 = Trn::parse("trn:aiplatform:model:bert:base-model:v1.0.0").unwrap();
    assert_eq!(trn1.version(), "v1.0.0");
    
    let trn2 = Trn::parse("trn:aiplatform:model:bert:base-model:latest").unwrap();
    assert_eq!(trn2.version(), "latest");
    
    let trn3 = Trn::parse("trn:aiplatform:model:bert:base-model:1.0").unwrap();
    assert_eq!(trn3.version(), "1.0");
}

#[test]
fn test_parse_long_components() {
    // Test TRN with longer component names (7 components with scope, keeping under length limits)
    let trn = Trn::parse("trn:org:organization-name:model:transformer:large-language-model:v2.1.0").unwrap();
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("organization-name"));
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.type_(), "transformer");
    assert_eq!(trn.instance_id(), "large-language-model");
    assert_eq!(trn.version(), "v2.1.0");
}

// =================================
// Performance and Stress Tests
// =================================

#[test]
fn test_parse_large_batch() {
    // Test parsing a large number of TRNs
    let trns: Vec<String> = (0..100)
        .map(|i| format!("trn:aiplatform:model:bert:model-{}:v1.0", i))
        .collect();
    
    for trn_str in &trns {
        let trn = Trn::parse(trn_str).unwrap();
        assert_eq!(trn.platform(), "aiplatform");
        assert_eq!(trn.resource_type(), "model");
        assert_eq!(trn.type_(), "bert");
    }
}

#[test]
fn test_parse_consistent_results() {
    // Test that parsing the same TRN multiple times gives consistent results
    let trn_str = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    for _ in 0..10 {
        let trn = Trn::parse(trn_str).unwrap();
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.type_(), "openapi");
        assert_eq!(trn.instance_id(), "github-api");
        assert_eq!(trn.version(), "v1.0");
    }
}

// =================================
// Roundtrip Tests
// =================================

#[test]
fn test_parse_and_serialize_roundtrip() {
    let original = "trn:user:alice:tool:openapi:github-api:v1.0";
    let trn = Trn::parse(original).unwrap();
    let serialized = trn.to_string();
    assert_eq!(original, serialized);
}

#[test]
fn test_parse_components_roundtrip() {
    let original = "trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    let trn = Trn::parse(original).unwrap();
    let components = trn.components();
    let new_trn = components.to_owned();
    let serialized = new_trn.to_string();
    assert_eq!(original, serialized);
}

// =================================
// Convenience Function Tests
// =================================

#[test]
fn test_parse_convenience_function() {
    // Test the convenience parse function from lib.rs
    let trn = parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.instance_id(), "github-api");
}

#[test]
fn test_parse_convenience_function_invalid() {
    assert!(parse("invalid-trn").is_err());
    assert!(parse("").is_err());
} 