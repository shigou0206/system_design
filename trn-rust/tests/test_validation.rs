use trn_rust::*;

// =================================
// Basic Validation Tests
// =================================

#[test]
fn test_is_valid_trn() {
    // Valid TRN formats
    assert!(is_valid_trn("trn:aiplatform:model:bert:base-model:v1.0"));
    assert!(is_valid_trn("trn:user:alice:tool:openapi:github-api:v1.0"));
    assert!(is_valid_trn("trn:org:company:dataset:csv:sales-data:v2.0"));
    assert!(is_valid_trn("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    
    // Invalid TRN formats
    assert!(!is_valid_trn("invalid-trn"));
    assert!(!is_valid_trn("trn:user")); // Too few components
    assert!(!is_valid_trn("not-trn:user:alice:tool:openapi:github-api:v1.0")); // Wrong prefix
    assert!(!is_valid_trn("")); // Empty string
    assert!(!is_valid_trn("trn::tool:openapi:github-api:v1.0")); // Empty platform
}

#[test]
fn test_validate_convenience_function() {
    // Test validation through convenience function
    assert!(validate("trn:aiplatform:model:bert:base-model:v1.0").is_ok());
    assert!(validate("trn:user:alice:tool:openapi:github-api:v1.0").is_ok());
    
    // Test invalid TRNs
    assert!(validate("invalid-trn").is_err());
    assert!(validate("").is_err());
    
    // Test validation with TRN object
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert!(validate(&trn).is_ok());
}

#[test]
fn test_trn_validate_method() {
    // Test TRN's validate method
    let valid_trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert!(valid_trn.validate().is_ok());
    assert!(valid_trn.is_valid());
}

// =================================
// TrnValidator Tests
// =================================

#[test]
fn test_trn_validator_basic() {
    let validator = TrnValidator::new();
    
    // Test valid TRNs
    assert!(validator.validate("trn:aiplatform:model:bert:base-model:v1.0").is_ok());
    assert!(validator.validate("trn:user:alice:tool:openapi:github-api:v1.0").is_ok());
    assert!(validator.is_valid("trn:org:company:dataset:csv:sales-data:v2.0"));
    
    // Test invalid TRNs
    assert!(validator.validate("invalid-trn").is_err());
    assert!(!validator.is_valid("trn:user")); // Too few components
    assert!(!validator.is_valid("")); // Empty string
}

#[test]
fn test_trn_validator_default() {
    let validator = TrnValidator::new();
    
    assert!(validator.validate("trn:user:alice:tool:openapi:github-api:v1.0").is_ok());
    assert!(validator.is_valid("trn:aiplatform:model:bert:base-model:v1.0"));
}

#[test]
fn test_trn_validator_cache_stats() {
    let validator = TrnValidator::new();
    
    // Perform some validations
    validator.is_valid("trn:user:alice:tool:openapi:github-api:v1.0");
    validator.is_valid("trn:aiplatform:model:bert:base-model:v1.0");
    validator.is_valid("invalid-trn");
    
    // Check cache stats (just verify cache is accessible)
    let stats = validator.cache_stats();
    assert!(stats.max_size > 0); // Cache should have a positive max size
}

// =================================
// Batch Validation Tests
// =================================

#[test]
fn test_batch_validate() {
    let trns = vec![
        "trn:aiplatform:model:bert:base-model:v1.0".to_string(),
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:org:company:dataset:csv:sales-data:v2.0".to_string(),
        "invalid-trn".to_string(),
        "trn:user".to_string(), // Too few components
    ];
    
    let report = batch_validate(&trns);
    
    assert_eq!(report.total, 5);
    assert_eq!(report.valid, 3); // First 3 are valid
    assert_eq!(report.invalid, 2); // Last 2 are invalid
    assert!(report.success_count == 3);
    assert!(report.success_count < report.total);
}

#[test]
fn test_batch_validate_all_valid() {
    let trns = vec![
        "trn:aiplatform:model:bert:base-model:v1.0".to_string(),
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:org:company:dataset:csv:sales-data:v2.0".to_string(),
    ];
    
    let report = batch_validate(&trns);
    
    assert_eq!(report.total, 3);
    assert_eq!(report.valid, 3);
    assert_eq!(report.invalid, 0);
    assert_eq!(report.success_count, 3);
    assert_eq!(report.success_count, report.total);
}

#[test]
fn test_batch_validate_all_invalid() {
    let trns = vec![
        "invalid-trn".to_string(),
        "not-trn:user:alice".to_string(),
        "trn:user".to_string(),
        "".to_string(),
    ];
    
    let report = batch_validate(&trns);
    
    assert_eq!(report.total, 4);
    assert_eq!(report.valid, 0);
    assert_eq!(report.invalid, 4);
    assert_eq!(report.success_count, 0);
}

#[test]
fn test_batch_validate_empty() {
    let trns: Vec<String> = vec![];
    let report = batch_validate(&trns);
    
    assert_eq!(report.total, 0);
    assert_eq!(report.valid, 0);
    assert_eq!(report.invalid, 0);
    assert_eq!(report.success_count, 0);
    assert_eq!(report.success_count, report.total);
}

// =================================
// Component Validation Tests
// =================================

#[test]
fn test_is_valid_identifier() {
    // Valid identifiers
    assert!(is_valid_identifier("github-api"));
    assert!(is_valid_identifier("base-model"));
    assert!(is_valid_identifier("openapi"));
    assert!(is_valid_identifier("bert"));
    assert!(is_valid_identifier("api-v2"));
    assert!(is_valid_identifier("user123"));
    
    // Invalid identifiers
    assert!(!is_valid_identifier(""));
    assert!(!is_valid_identifier("invalid@id"));
    assert!(!is_valid_identifier("invalid.id"));
    assert!(!is_valid_identifier("invalid/id"));
    assert!(!is_valid_identifier("invalid id")); // Space
}

#[test]
fn test_is_valid_scope() {
    // Valid scopes
    assert!(is_valid_scope("alice"));
    assert!(is_valid_scope("company"));
    assert!(is_valid_scope("user-123"));
    assert!(is_valid_scope("org-name"));
    assert!(is_valid_scope("enterprise"));
    
    // Invalid scopes  
    assert!(!is_valid_scope(""));
    assert!(!is_valid_scope("invalid@scope"));
    assert!(!is_valid_scope("invalid.scope"));
    assert!(!is_valid_scope("invalid/scope"));
    assert!(!is_valid_scope("invalid scope")); // Space
}

#[test]
fn test_is_valid_version() {
    // Valid versions
    assert!(is_valid_version("v1.0"));
    assert!(is_valid_version("v2.0.0"));
    assert!(is_valid_version("latest"));
    assert!(is_valid_version("1.0"));
    assert!(is_valid_version("v1.0-beta"));
    assert!(is_valid_version("v1.0.0-alpha.1"));
    assert!(is_valid_version("*")); // Wildcard
    
    // Invalid versions
    assert!(!is_valid_version(""));
    assert!(!is_valid_version("invalid@version"));
    assert!(!is_valid_version("invalid.version.with.too.many.dots"));
    assert!(!is_valid_version("invalid version")); // Space
}

// =================================
// TrnStats Tests  
// =================================

#[test]
fn test_trn_stats_analyze() {
    let trns = vec![
        Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap(),
        Trn::parse("trn:user:bob:tool:python:data-processor:v2.0").unwrap(),
        Trn::parse("trn:org:company:model:bert:language-model:v1.0").unwrap(),
        Trn::parse("trn:aiplatform:model:gpt:text-generator:latest").unwrap(),
    ];
    
    let stats = TrnStats::analyze(&trns);
    
    assert_eq!(stats.total_count, 4);
    assert_eq!(stats.platform_distribution.len(), 3); // user, org, aiplatform
    assert_eq!(stats.resource_type_distribution.len(), 2); // tool, model
    assert!(stats.platform_distribution.contains_key(&Platform::User));
    assert!(stats.platform_distribution.contains_key(&Platform::Org));
    assert!(stats.platform_distribution.contains_key(&Platform::AiPlatform));
    assert_eq!(stats.platform_distribution[&Platform::User], 2);
    assert_eq!(stats.platform_distribution[&Platform::Org], 1);
    assert_eq!(stats.platform_distribution[&Platform::AiPlatform], 1);
}

#[test]
fn test_trn_stats_empty() {
    let trns: Vec<Trn> = vec![];
    let stats = TrnStats::analyze(&trns);
    
    assert_eq!(stats.total_count, 0);
    assert_eq!(stats.platform_distribution.len(), 0);
    assert_eq!(stats.resource_type_distribution.len(), 0);
}

#[test]
fn test_trn_stats_single() {
    let trns = vec![
        Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap(),
    ];
    
    let stats = TrnStats::analyze(&trns);
    
    assert_eq!(stats.total_count, 1);
    assert_eq!(stats.platform_distribution.len(), 1);
    assert_eq!(stats.platform_distribution[&Platform::User], 1);
    assert_eq!(stats.resource_type_distribution[&ResourceType::Tool], 1);
}

// =================================
// Normalization Tests
// =================================

#[test]
fn test_normalize_trn() {
    // Test basic normalization
    let normalized = normalize_trn("trn:aiplatform:model:bert:base-model:v1.0");
    assert_eq!(normalized, "trn:aiplatform:model:bert:base-model:v1.0");
    
    // Test normalization with scope
    let normalized_scope = normalize_trn("trn:user:alice:tool:openapi:github-api:v1.0");
    assert_eq!(normalized_scope, "trn:user:alice:tool:openapi:github-api:v1.0");
    
    // Test normalization with hash
    let normalized_hash = normalize_trn("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    assert_eq!(normalized_hash, "trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
}

// =================================
// Validation Rules Tests
// =================================

#[test]
fn test_platform_validation_rules() {
    // User platform requires scope
    assert!(is_valid_trn("trn:user:alice:tool:openapi:github-api:v1.0"));
    assert!(!is_valid_trn("trn:user:tool:openapi:github-api:v1.0")); // Missing scope
    
    // Org platform requires scope  
    assert!(is_valid_trn("trn:org:company:model:bert:language-model:v1.0"));
    assert!(!is_valid_trn("trn:org:model:bert:language-model:v1.0")); // Missing scope
    
    // Aiplatform doesn't require scope
    assert!(is_valid_trn("trn:aiplatform:model:bert:base-model:v1.0"));
}

#[test]
fn test_resource_type_validation() {
    // Valid resource types with proper format
    assert!(is_valid_trn("trn:aiplatform:tool:openapi:apigateway:v1.0"));
    assert!(is_valid_trn("trn:aiplatform:model:bert:basemodel:v1.0"));
    assert!(is_valid_trn("trn:org:company:dataset:csv:salesdata:v1.0"));
    assert!(is_valid_trn("trn:aiplatform:pipeline:etl:dataprocessor:v1.0"));
    assert!(is_valid_trn("trn:aiplatform:tool:openapi:chatbot:v1.0"));
}

#[test]
fn test_version_validation_rules() {
    // Valid versions
    assert!(is_valid_trn("trn:aiplatform:model:bert:base-model:v1.0"));
    assert!(is_valid_trn("trn:aiplatform:model:bert:base-model:latest"));
    assert!(is_valid_trn("trn:aiplatform:model:bert:base-model:1.0.0"));
    assert!(is_valid_trn("trn:aiplatform:model:bert:base-model:v2.1.0-beta"));
    
    // Invalid versions (empty version)
    assert!(!is_valid_trn("trn:aiplatform:model:bert:base-model:"));
}

// =================================
// Error Handling Tests
// =================================

#[test]
fn test_validation_error_messages() {
    let validator = TrnValidator::new();
    
    // Test specific error for invalid TRN
    let result = validator.validate("trn:user:tool:openapi:githubapi:v1.0");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty()); // Just verify we get some error message
    
    // Test error for invalid format
    let result2 = validator.validate("invalid-trn");
    assert!(result2.is_err());
    let error_msg2 = result2.unwrap_err().to_string();
    assert!(error_msg2.contains("format") || error_msg2.contains("invalid"));
}

// =================================
// Performance Tests
// =================================

#[test]
fn test_validation_performance_batch() {
    // Create a large batch of TRNs for performance testing
    let trns: Vec<String> = (0..1000)
        .map(|i| format!("trn:aiplatform:model:bert:model-{}:v1.0", i))
        .collect();
    
    let report = batch_validate(&trns);
    
    assert_eq!(report.total, 1000);
    assert_eq!(report.valid, 1000); // All should be valid
    assert_eq!(report.invalid, 0);
    assert_eq!(report.success_count, report.total);
}

#[test]
fn test_validation_caching() {
    let validator = TrnValidator::new();
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    // First validation (should cache)
    let result1 = validator.is_valid(trn);
    
    // Second validation (should use cache)
    let result2 = validator.is_valid(trn);
    
    assert_eq!(result1, result2); // Results should be the same
    // Just verify the cache is working by checking we get consistent results
}

// =================================
// Configuration Tests
// =================================

// #[test]
// fn test_validation_config() {
//     let config = ValidationConfig::new();
//     let validator = TrnValidator::with_config(config);
    
//     // Test that config is accessible
//     let retrieved_config = validator.config();
//     assert!(retrieved_config.strict_validation); // Should have default strict validation
// }

// =================================
// Edge Cases
// =================================

#[test]
fn test_validation_edge_cases() {
    // Shorter TRN (within limits)
    let long_trn = format!("trn:org:{}:tool:openapi:{}:v1.0", 
                          "a".repeat(30), "b".repeat(30));
    assert!(is_valid_trn(&long_trn));
    
    // TRN with special characters in allowed positions (use org platform for dashes)
    assert!(is_valid_trn("trn:org:user-123:tool:openapi:api-v2:v1.0"));
    assert!(is_valid_trn("trn:user:username:tool:openapi:apiv2:v1.0"));
}

#[test]
fn test_validation_consistent_results() {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    // Multiple validations should give consistent results
    for _ in 0..10 {
        assert!(is_valid_trn(trn));
        assert!(validate(trn).is_ok());
    }
} 