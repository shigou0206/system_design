use trn_rust::{
    Trn, is_valid_trn, validate_trn_string, validate_trn_struct,
    validate_multiple_trns, generate_validation_report, check_component_format,
    validate_naming_conventions, ValidationCache
};

#[test]
fn test_valid_trns() {
    let valid_cases = vec![
        "trn:user:alice:tool:myapi:v1.0",
        "trn:org:company:model:bert:v2.1",
        "trn:aiplatform:system:dataset:training:latest",
        "trn:user:bob:pipeline:etl:main",
        "trn:org:startup:tool:analyzer:dev",
    ];
    
    for trn_str in valid_cases {
        assert!(is_valid_trn(trn_str), "Should be valid: {}", trn_str);
        assert!(validate_trn_string(trn_str).is_ok(), "Should validate: {}", trn_str);
    }
}

#[test]
fn test_invalid_format() {
    let invalid_cases = vec![
        ("", "empty string"),
        ("invalid", "not trn format"),
        ("trn:user:alice", "too few components"),
        ("trn:user:alice:tool:myapi:v1.0:extra", "too many components"),
        ("nottrn:user:alice:tool:myapi:v1.0", "wrong prefix"),
    ];
    
    for (trn_str, reason) in invalid_cases {
        assert!(!is_valid_trn(trn_str), "Should be invalid ({}): {}", reason, trn_str);
        assert!(validate_trn_string(trn_str).is_err(), "Should fail validation ({}): {}", reason, trn_str);
    }
}

#[test]
fn test_empty_components() {
    let invalid_cases = vec![
        ("trn::alice:tool:myapi:v1.0", "empty platform"),
        ("trn:user::tool:myapi:v1.0", "empty scope"),
        ("trn:user:alice::myapi:v1.0", "empty resource type"),
        ("trn:user:alice:tool::v1.0", "empty resource id"),
        ("trn:user:alice:tool:myapi:", "empty version"),
    ];
    
    for (trn_str, reason) in invalid_cases {
        assert!(!is_valid_trn(trn_str), "Should be invalid ({}): {}", reason, trn_str);
    }
}

#[test]
fn test_component_length_validation() {
    // Test very long components
    let long_platform = "a".repeat(100);
    let long_scope = "b".repeat(100);
    let long_resource_type = "c".repeat(100);
    let long_resource_id = "d".repeat(200);
    let long_version = "e".repeat(100);
    
    let test_cases = vec![
        (format!("trn:{}:alice:tool:myapi:v1.0", long_platform), "platform too long"),
        (format!("trn:user:{}:tool:myapi:v1.0", long_scope), "scope too long"),
        (format!("trn:user:alice:{}:myapi:v1.0", long_resource_type), "resource type too long"),
        (format!("trn:user:alice:tool:{}:v1.0", long_resource_id), "resource id too long"),
        (format!("trn:user:alice:tool:myapi:{}", long_version), "version too long"),
    ];
    
    for (trn_str, reason) in test_cases {
        let _result = validate_trn_string(&trn_str);
        // Should either fail parsing or validation
        if let Ok(trn) = Trn::parse(&trn_str) {
            assert!(trn.validate().is_err(), "Should fail validation ({}): {}", reason, trn_str);
        }
    }
}

#[test]
fn test_naming_conventions() {
    let test_cases = vec![
        ("trn:USER:alice:tool:myapi:v1.0", true, "uppercase platform allowed"),
        ("trn:user:ALICE:tool:myapi:v1.0", true, "uppercase scope allowed"),
        ("trn:user:alice:tool:myapi:v1.0", true, "lowercase resource type required"),
        ("trn:user:alice:tool:MYAPI:v1.0", true, "uppercase resource id allowed"),
        ("trn:user:alice:tool:myapi:V1.0", true, "uppercase version allowed"),
    ];
    
    for (trn_str, should_pass, reason) in test_cases {
        let trn = Trn::parse(trn_str).unwrap();
        let result = validate_naming_conventions(&trn);
        
        if should_pass {
            assert!(result.is_ok(), "Should pass {}: {}", reason, trn_str);
        } else {
            assert!(result.is_err(), "Should fail {}: {}", reason, trn_str);
        }
    }
}

#[test]
fn test_reserved_words() {
    // Test reserved words (using actual reserved words)
    let reserved_cases = vec![
        "trn:trn:alice:tool:myapi:v1.0",        // reserved platform
        "trn:null:alice:tool:myapi:v1.0",       // reserved platform
        "trn:void:alice:tool:myapi:v1.0",       // reserved platform
        "trn:user:null:tool:myapi:v1.0",        // reserved scope
        "trn:user:undefined:tool:myapi:v1.0",   // reserved scope
    ];
    
    for trn_str in reserved_cases {
        assert!(!is_valid_trn(trn_str), "Should reject reserved word: {}", trn_str);
    }
}

#[test]
fn test_platform_specific_validation() {
    // Test user platform validation
    let user_cases = vec![
        ("trn:user:a:tool:myapi:v1.0", false, "scope too short"),
        ("trn:user:alice:tool:myapi:v1.0", true, "valid user scope"),
        ("trn:user:a_very_long_username_that_exceeds_limit:tool:myapi:v1.0", false, "scope too long"),
    ];
    
    for (trn_str, should_pass, reason) in user_cases {
        let result = is_valid_trn(trn_str);
        if should_pass {
            assert!(result, "Should pass ({}): {}", reason, trn_str);
        } else {
            assert!(!result, "Should fail ({}): {}", reason, trn_str);
        }
    }
}

#[test]
fn test_version_format_validation() {
    let version_cases = vec![
        ("trn:user:alice:tool:myapi:v1.0", true, "semantic version"),
        ("trn:user:alice:tool:myapi:latest", true, "latest alias"),
        ("trn:user:alice:tool:myapi:dev", true, "dev alias"),
        ("trn:user:alice:tool:myapi:main", true, "main alias"),
        ("trn:user:alice:tool:myapi:1.2.3", true, "numeric version"),
        ("trn:user:alice:tool:myapi:v2.0-beta", true, "pre-release version"),
        ("trn:user:alice:tool:myapi:feature-branch", true, "feature branch"),
        ("trn:user:alice:tool:myapi:", false, "empty version"),
    ];
    
    for (trn_str, should_pass, reason) in version_cases {
        let result = is_valid_trn(trn_str);
        if should_pass {
            assert!(result, "Should pass ({}): {}", reason, trn_str);
        } else {
            assert!(!result, "Should fail ({}): {}", reason, trn_str);
        }
    }
}

#[test]
fn test_batch_validation() {
    let trns = vec![
        "trn:user:alice:tool:myapi:v1.0".to_string(),
        "invalid:format".to_string(),
        "trn:org:company:model:bert:v2.1".to_string(),
        "trn:user::tool:myapi:v1.0".to_string(),  // Empty scope
        "trn:aiplatform:system:dataset:training:latest".to_string(),
    ];
    
    let results = validate_multiple_trns(&trns);
    assert_eq!(results.len(), 5);
    
    // Check individual results
    assert!(results[0].is_ok()); // Valid TRN
    assert!(results[1].is_err()); // Invalid format
    assert!(results[2].is_ok()); // Valid TRN
    assert!(results[3].is_err()); // Empty scope
    assert!(results[4].is_ok()); // Valid TRN
    
    // Generate report
    let report = generate_validation_report(&trns);
    assert_eq!(report.total, 5);
    assert_eq!(report.valid, 3);
    assert_eq!(report.invalid, 2);
    assert_eq!(report.errors.len(), 2);
}

#[test]
fn test_validation_cache() {
    let cache = ValidationCache::new(100, 60);
    
    // Test cache miss
    assert_eq!(cache.get("test_trn"), None);
    
    // Insert and test cache hit
    cache.insert("test_trn".to_string(), true);
    assert_eq!(cache.get("test_trn"), Some(true));
    
    // Insert invalid result
    cache.insert("invalid_trn".to_string(), false);
    assert_eq!(cache.get("invalid_trn"), Some(false));
    
    // Test cache stats
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 2);
}

#[test]
fn test_trn_struct_validation() {
    // Valid TRN
    let valid_trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    assert!(validate_trn_struct(&valid_trn).is_ok());
}

#[test]
fn test_component_format_check() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let components = trn.components();
    
    let issues = check_component_format(&components);
    assert!(issues.is_empty(), "Valid TRN should have no format issues");
}

#[test]
fn test_validation_error_messages() {
    let test_cases = vec![
        ("", "empty string"),
        ("not_a_trn", "wrong format"),
        ("trn:user", "too few components"),
        ("prefix:user:alice:tool:myapi:v1.0", "wrong prefix"),
    ];
    
    for (invalid_trn, expected_reason) in test_cases {
        let result = validate_trn_string(invalid_trn);
        assert!(result.is_err(), "Should be invalid: {}", invalid_trn);
        
        let error_msg = result.unwrap_err().to_string().to_lowercase();
        match expected_reason {
            "empty string" => assert!(error_msg.contains("empty") || error_msg.contains("invalid")),
            "too few components" => assert!(error_msg.contains("component") || error_msg.contains("format") || error_msg.contains("expected")),
            "empty platform" => assert!(error_msg.contains("empty") || error_msg.contains("platform")),
            "wrong prefix" => assert!(error_msg.contains("trn") || error_msg.contains("format") || error_msg.contains("expected")),
            "wrong format" => assert!(error_msg.contains("format") || error_msg.contains("expected")),
            _ => {}
        }
    }
}

#[test]
fn test_validation_performance() {
    use std::time::Instant;
    
    let trns: Vec<String> = (0..1000)
        .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i, i))
        .collect();
    
    let start = Instant::now();
    let report = generate_validation_report(&trns);
    let duration = start.elapsed();
    
    assert_eq!(report.total, 1000);
    assert_eq!(report.valid, 1000);
    assert_eq!(report.invalid, 0);
    
    // Should validate 1000 TRNs in less than 1 second
    assert!(duration.as_millis() < 1000, "Validation too slow: {:?}", duration);
} 