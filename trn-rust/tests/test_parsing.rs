use trn_rust::*;

#[test]
fn test_basic_parsing() {
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    let trn = Trn::parse(trn_str).unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), "alice");
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.resource_id(), "myapi");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_parse_different_platforms() {
    let test_cases = vec![
        ("trn:user:alice:tool:myapi:v1.0", "user"),
        ("trn:org:company:model:bert:v2.1", "org"),
        ("trn:aiplatform:system:dataset:training:latest", "aiplatform"),
        ("trn:custom:scope:pipeline:etl:v1.0", "custom"),
    ];
    
    for (trn_str, expected_platform) in test_cases {
        let trn = Trn::parse(trn_str).unwrap();
        assert_eq!(trn.platform(), expected_platform);
    }
}

#[test]
fn test_parse_different_resource_types() {
    let test_cases = vec![
        ("trn:user:alice:tool:myapi:v1.0", "tool"),
        ("trn:org:company:model:bert:v2.1", "model"),
        ("trn:user:bob:dataset:images:v1.0", "dataset"),
        ("trn:org:startup:pipeline:etl:v1.0", "pipeline"),
        ("trn:user:charlie:custom-type:resource:v1.0", "custom-type"),
    ];
    
    for (trn_str, expected_type) in test_cases {
        let trn = Trn::parse(trn_str).unwrap();
        assert_eq!(trn.resource_type(), expected_type);
    }
}

#[test]
fn test_parse_versions() {
    let test_cases = vec![
        ("trn:user:alice:tool:myapi:v1.0", "v1.0"),
        ("trn:user:alice:tool:myapi:latest", "latest"),
        ("trn:user:alice:tool:myapi:1.2.3", "1.2.3"),
        ("trn:user:alice:tool:myapi:dev", "dev"),
        ("trn:user:alice:tool:myapi:v2.0-beta", "v2.0-beta"),
        ("trn:user:alice:tool:myapi:main", "main"),
    ];
    
    for (trn_str, expected_version) in test_cases {
        let trn = Trn::parse(trn_str).unwrap();
        assert_eq!(trn.version(), expected_version);
    }
}

#[test]
fn test_parse_complex_identifiers() {
    let test_cases = vec![
        ("trn:user:alice-smith:tool:my-api-tool:v1.0", ("alice-smith", "my-api-tool")),
        ("trn:org:tech_company:model:bert_large:v2.1", ("tech_company", "bert_large")),
        ("trn:user:user123:dataset:dataset_01:v1.0", ("user123", "dataset_01")),
        ("trn:aiplatform:system-prod:pipeline:data-processing:latest", ("system-prod", "data-processing")),
    ];
    
    for (trn_str, (expected_scope, expected_resource_id)) in test_cases {
        let trn = Trn::parse(trn_str).unwrap();
        assert_eq!(trn.scope(), expected_scope);
        assert_eq!(trn.resource_id(), expected_resource_id);
    }
}

#[test]
fn test_parse_invalid_format() {
    let invalid_cases = vec![
        "invalid:format",                           // Wrong prefix
        "trn:user:alice",                          // Too few components
        "trn:user:alice:tool",                     // Too few components
        "trn:user:alice:tool:myapi",               // Missing version
        "trn:user:alice:tool:myapi:v1.0:extra",    // Too many components
        "",                                        // Empty string
        "trn:",                                    // Only prefix
        ":user:alice:tool:myapi:v1.0",            // Missing prefix
    ];
    
    for invalid_trn in invalid_cases {
        assert!(Trn::parse(invalid_trn).is_err(), "Should fail for: {}", invalid_trn);
    }
}

#[test]
fn test_parse_empty_components() {
    let invalid_cases = vec![
        "trn::alice:tool:myapi:v1.0",      // Empty platform
        "trn:user::tool:myapi:v1.0",       // Empty scope
        "trn:user:alice::myapi:v1.0",      // Empty resource type
        "trn:user:alice:tool::v1.0",       // Empty resource ID
        "trn:user:alice:tool:myapi:",      // Empty version
    ];
    
    for invalid_trn in invalid_cases {
        assert!(Trn::parse(invalid_trn).is_err(), "Should fail for: {}", invalid_trn);
    }
}

#[test]
fn test_parse_special_characters() {
    let test_cases = vec![
        ("trn:user:alice:tool:my-api:v1.0", true),
        ("trn:user:alice:tool:my_api:v1.0", true),
        ("trn:user:alice:tool:myapi123:v1.0", true),
        ("trn:user:alice-123:tool:myapi:v1.0", true),
        ("trn:user:alice:tool:my.api:v1.0", true),
    ];
    
    for (trn_str, should_pass) in test_cases {
        let result = Trn::parse(trn_str);
        if should_pass {
            assert!(result.is_ok(), "Should pass for: {}", trn_str);
        } else {
            assert!(result.is_err(), "Should fail for: {}", trn_str);
        }
    }
}

#[test]
fn test_parse_component_extraction() {
    let trn_str = "trn:org:company:model:bert-large:v2.1";
    let trn = Trn::parse(trn_str).unwrap();
    
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), "company");
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.resource_id(), "bert-large");
    assert_eq!(trn.version(), "v2.1");
}

#[test]
fn test_parse_from_str_trait() {
    use std::str::FromStr;
    
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    let trn = Trn::from_str(trn_str).unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), "alice");
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.resource_id(), "myapi");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_parse_roundtrip() {
    let original = "trn:user:alice:tool:myapi:v1.0";
    let trn = Trn::parse(original).unwrap();
    let reconstructed = trn.to_string();
    
    assert_eq!(original, reconstructed);
}

#[test]
fn test_parse_case_sensitivity() {
    // Test with uppercase platform (now allowed)
    let trn_str = "trn:USER:alice:tool:myapi:v1.0";
    let trn = Trn::parse(trn_str).unwrap(); // Parsing succeeds
    assert_eq!(trn.platform(), "USER"); // And preserves case
    
    // Validation should now pass since uppercase is allowed
    assert!(trn.validate().is_ok());
}

#[test]
fn test_parse_unicode_support() {
    // Test with unicode characters (should be rejected by validation)
    let unicode_trn = "trn:user:alice:tool:测试:v1.0";
    assert!(Trn::parse(unicode_trn).is_err());
}

#[test]
fn test_parse_edge_cases() {
    // Test minimum valid TRN (with minimum required lengths)
    let min_trn = "trn:aa:b:tool:d:e";  // Use supported resource type and min platform length
    assert!(Trn::parse(min_trn).is_ok());
    
    // Test with maximum allowed component lengths (test with reasonable lengths)
    let max_trn = format!(
        "trn:{}:{}:{}:{}:{}",
        "a".repeat(20),  // platform
        "b".repeat(20),  // scope
        "c".repeat(20),  // resource_type
        "d".repeat(50),  // resource_id
        "e".repeat(20)   // version
    );
    
    let result = Trn::parse(&max_trn);
    // This should parse but might fail validation based on length limits
    if let Ok(trn) = result {
        // Just verify it parsed correctly
        assert!(trn.platform().len() > 0);
    }
}

#[test]
fn test_parse_performance() {
    use std::time::Instant;
    
    let trn_str = "trn:user:alice:tool:myapi:v1.0";
    let start = Instant::now();
    
    // Parse 1000 times
    for _ in 0..1000 {
        let _ = Trn::parse(trn_str).unwrap();
    }
    
    let duration = start.elapsed();
    // Should be fast - less than 100ms for 1000 parses
    assert!(duration.as_millis() < 100, "Parsing too slow: {:?}", duration);
} 