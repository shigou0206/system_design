use trn_rust::{Trn, TrnBuilder, is_valid_trn, url_to_trn};

#[test]
fn test_complete_trn_workflow() {
    // Create TRN using constructor
    let trn1 = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    // Create TRN using builder
    let trn2 = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build()
        .unwrap();
    
    // Create TRN from string
    let trn3 = Trn::parse("trn:user:alice:tool:myapi:v1.0").unwrap();
    
    // All should be equal
    assert_eq!(trn1.to_string(), trn2.to_string());
    assert_eq!(trn2.to_string(), trn3.to_string());
    
    // All should be valid
    assert!(trn1.is_valid());
    assert!(trn2.is_valid());
    assert!(trn3.is_valid());
    assert!(is_valid_trn(&trn1.to_string()));
}

#[test]
fn test_trn_url_roundtrip_workflow() {
    let original = Trn::new("org", "company", "model", "bert-large", "v2.1").unwrap();
    
    // Convert to TRN URL
    let trn_url = original.to_url().unwrap();
    assert!(trn_url.starts_with("trn://"));
    
    // Convert back from URL
    let from_url = url_to_trn(&trn_url).unwrap();
    
    // Should be identical
    assert_eq!(original.platform(), from_url.platform());
    assert_eq!(original.scope(), from_url.scope());
    assert_eq!(original.resource_type(), from_url.resource_type());
    assert_eq!(original.resource_id(), from_url.resource_id());
    assert_eq!(original.version(), from_url.version());
    
    // Convert to HTTP URL
    let http_url = original.to_http_url("https://registry.example.com").unwrap();
    assert!(http_url.starts_with("https://registry.example.com"));
    assert!(http_url.contains("org"));
    assert!(http_url.contains("company"));
    assert!(http_url.contains("model"));
    assert!(http_url.contains("bert-large"));
    assert!(http_url.contains("v2.1"));
}

#[test]
fn test_different_platforms_integration() {
    let platforms = vec![
        ("user", "alice", "Personal user platform"),
        ("org", "company", "Organization platform"),
        ("aiplatform", "system", "AI platform"),
        ("custom", "scope", "Custom platform"),
    ];
    
    for (platform, scope, description) in platforms {
        // Create TRN
        let trn = Trn::new(platform, scope, "tool", "myapi", "v1.0").unwrap();
        
        // Validate
        assert!(trn.is_valid(), "TRN should be valid for {}", description);
        
        // Test string conversion
        let trn_str = trn.to_string();
        assert!(trn_str.starts_with("trn:"));
        assert!(trn_str.contains(platform));
        assert!(trn_str.contains(scope));
        
        // Test parsing back
        let parsed = Trn::parse(&trn_str).unwrap();
        assert_eq!(trn.platform(), parsed.platform());
        assert_eq!(trn.scope(), parsed.scope());
        
        // Test URL conversion
        let url = trn.to_url().unwrap();
        let from_url = url_to_trn(&url).unwrap();
        assert_eq!(trn.to_string(), from_url.to_string());
    }
}

#[test]
fn test_different_resource_types_integration() {
    let resource_types = vec!["tool", "model", "dataset", "pipeline", "custom-type"];
    
    for resource_type in resource_types {
        let trn = Trn::new("user", "alice", resource_type, "resource", "v1.0").unwrap();
        
        // Test all conversions
        let trn_str = trn.to_string();
        let parsed = Trn::parse(&trn_str).unwrap();
        let url = trn.to_url().unwrap();
        let from_url = url_to_trn(&url).unwrap();
        
        // All should match
        assert_eq!(trn.resource_type(), parsed.resource_type());
        assert_eq!(parsed.resource_type(), from_url.resource_type());
        assert!(trn.is_valid());
        assert!(parsed.is_valid());
        assert!(from_url.is_valid());
    }
}

#[test]
fn test_version_handling_integration() {
    let versions = vec![
        "v1.0",
        "latest",
        "dev",
        "main",
        "1.2.3",
        "v2.0-beta",
        "feature-branch",
        "v1.0.0-rc.1",
    ];
    
    for version in versions {
        let trn = Trn::new("user", "alice", "tool", "myapi", version).unwrap();
        
        // Test version handling
        assert_eq!(trn.version(), version);
        assert!(trn.is_valid());
        
        // Test base TRN (version becomes wildcard)
        let base = trn.base_trn();
        assert_eq!(base.version(), "*");
        assert_eq!(base.platform(), trn.platform());
        assert_eq!(base.scope(), trn.scope());
        assert_eq!(base.resource_type(), trn.resource_type());
        assert_eq!(base.resource_id(), trn.resource_id());
        
        // Test compatibility
        let other_version = Trn::new("user", "alice", "tool", "myapi", "v2.0").unwrap();
        assert!(trn.is_compatible_with(&other_version));
    }
}

#[test]
fn test_pattern_matching_integration() {
    let trns = vec![
        Trn::new("user", "alice", "tool", "api1", "v1.0").unwrap(),
        Trn::new("user", "alice", "tool", "api2", "v1.0").unwrap(),
        Trn::new("user", "bob", "tool", "api1", "v1.0").unwrap(),
        Trn::new("org", "company", "model", "bert", "v2.0").unwrap(),
        Trn::new("user", "alice", "model", "gpt", "v1.5").unwrap(),
    ];
    
    // Test various pattern combinations
    let patterns = vec![
        ("trn:user:alice:*:*:*", 3), // Alice's resources
        ("trn:*:*:tool:*:*", 3),     // All tools
        ("trn:*:*:model:*:*", 2),    // All models
        ("trn:user:*:*:*:v1.0", 3),  // User v1.0 resources
        ("trn:*:*:*:*:*", 5),        // Everything
    ];
    
    for (pattern, expected_count) in patterns {
        let matches = trns.iter()
            .filter(|trn| trn.matches_pattern(pattern))
            .count();
        assert_eq!(matches, expected_count, "Pattern {} should match {} TRNs", pattern, expected_count);
    }
}

#[test]
fn test_builder_workflow_integration() {
    // Start with minimal builder
    let mut builder = TrnBuilder::new();
    
    // Build step by step
    builder = builder.platform("user");
    builder = builder.scope("alice");
    builder = builder.resource_type("tool");
    builder = builder.resource_id("myapi");
    builder = builder.version("v1.0");
    
    let trn = builder.build().unwrap();
    
    // Test the result
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), "alice");
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.resource_id(), "myapi");
    assert_eq!(trn.version(), "v1.0");
    assert!(trn.is_valid());
    
    // Test chaining
    let chained = TrnBuilder::new()
        .platform("org")
        .scope("company")
        .resource_type("model")
        .resource_id("bert")
        .version("v2.1")
        .build()
        .unwrap();
    
    assert_eq!(chained.to_string(), "trn:org:company:model:bert:v2.1");
    
    // Test modification
    let modified = TrnBuilder::new()
        .platform(chained.platform())
        .scope(chained.scope())
        .resource_type(chained.resource_type())
        .resource_id(chained.resource_id())
        .version("v3.0")  // Change version
        .build()
        .unwrap();
    
    assert!(chained.is_compatible_with(&modified));
    assert_ne!(chained.version(), modified.version());
}

#[test]
fn test_error_handling_integration() {
    // Test invalid TRN strings
    let invalid_trns = vec![
        "",
        "invalid",
        "trn:user:alice",
        "trn:user:alice:tool:myapi:v1.0:extra",
        "nottrn:user:alice:tool:myapi:v1.0",
        "trn::alice:tool:myapi:v1.0",
        "trn:user::tool:myapi:v1.0",
        "trn:user:alice::myapi:v1.0",
        "trn:user:alice:tool::v1.0",
        "trn:user:alice:tool:myapi:",
    ];
    
    for invalid_trn in invalid_trns {
        assert!(Trn::parse(invalid_trn).is_err(), "Should fail for: {}", invalid_trn);
        assert!(!is_valid_trn(invalid_trn), "Should be invalid: {}", invalid_trn);
    }
    
    // Test invalid builder usage
    let incomplete_builders = vec![
        TrnBuilder::new().scope("alice").resource_type("tool").resource_id("myapi").version("v1.0"),
        TrnBuilder::new().platform("user").resource_type("tool").resource_id("myapi").version("v1.0"),
        TrnBuilder::new().platform("user").scope("alice").resource_id("myapi").version("v1.0"),
        TrnBuilder::new().platform("user").scope("alice").resource_type("tool").version("v1.0"),
        TrnBuilder::new().platform("user").scope("alice").resource_type("tool").resource_id("myapi"),
    ];
    
    for builder in incomplete_builders {
        assert!(builder.build().is_err(), "Incomplete builder should fail");
    }
}

#[test]
fn test_serialization_integration() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    // Test JSON serialization
    let json = trn.to_json().unwrap();
    assert!(json.contains("user"));
    assert!(json.contains("alice"));
    assert!(json.contains("tool"));
    assert!(json.contains("myapi"));
    assert!(json.contains("v1.0"));
    
    let from_json = Trn::from_json(&json).unwrap();
    assert_eq!(trn.to_string(), from_json.to_string());
    
    // Test that deserialized TRN is valid
    assert!(from_json.is_valid());
}

#[test]
fn test_special_characters_integration() {
    let special_trn = Trn::new(
        "user", 
        "alice-smith", 
        "custom-type", 
        "my_api.v2", 
        "v1.0-beta.1"
    ).unwrap();
    
    // Test string round trip
    let trn_str = special_trn.to_string();
    let parsed = Trn::parse(&trn_str).unwrap();
    assert_eq!(special_trn.to_string(), parsed.to_string());
    
    // Test URL round trip
    let url = special_trn.to_url().unwrap();
    let from_url = url_to_trn(&url).unwrap();
    assert_eq!(special_trn.to_string(), from_url.to_string());
    
    // Test validation
    assert!(special_trn.is_valid());
    assert!(parsed.is_valid());
    assert!(from_url.is_valid());
    
    // Test pattern matching
    assert!(special_trn.matches_pattern("trn:user:alice-*:custom-type:my_api.v2:v1.0-*"));
    assert!(special_trn.matches_pattern("trn:user:alice-smith:*-type:*:*"));
}

#[test]
fn test_performance_integration() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Create, validate, and convert 1000 TRNs
    for i in 0..1000 {
        let trn = Trn::new("user", &format!("user{}", i), "tool", &format!("api{}", i), "v1.0").unwrap();
        assert!(trn.is_valid());
        
        let trn_str = trn.to_string();
        let parsed = Trn::parse(&trn_str).unwrap();
        assert_eq!(trn.to_string(), parsed.to_string());
        
        let url = trn.to_url().unwrap();
        let from_url = url_to_trn(&url).unwrap();
        assert_eq!(trn.to_string(), from_url.to_string());
        
        assert!(trn.matches_pattern("trn:user:*:tool:*:v1.0"));
    }
    
    let duration = start.elapsed();
    // Should complete 1000 full workflows in less than 1 second
    assert!(duration.as_millis() < 1000, "Integration test too slow: {:?}", duration);
}

#[test]
fn test_edge_cases_integration() {
    // Test minimum valid TRN (6 components with minimum required lengths)
    let min_trn = Trn::new("aa", "b", "tool", "d", "e").unwrap();
    assert!(min_trn.is_valid());
    
    let min_str = min_trn.to_string();
    assert_eq!(min_str, "trn:aa:b:tool:d:e");
    
    let min_parsed = Trn::parse(&min_str).unwrap();
    assert_eq!(min_trn, min_parsed);
    
    let min_url = min_trn.to_url().unwrap();
    let min_from_url = url_to_trn(&min_url).unwrap();
    assert_eq!(min_trn.to_string(), min_from_url.to_string());
    
    // Test with longer but valid components
    let long_trn = Trn::new(
        &"platform".repeat(2),
        &"scope".repeat(2),
        "tool",  // Use supported resource type
        &"resource".repeat(5),
        &"version".repeat(2)
    ).unwrap();
    
    assert!(long_trn.is_valid());
    
    let long_str = long_trn.to_string();
    let long_parsed = Trn::parse(&long_str).unwrap();
    assert_eq!(long_trn.to_string(), long_parsed.to_string());
}

#[test]
fn test_library_consistency() {
    // Test that all creation methods produce consistent results
    let platform = "user";
    let scope = "alice";
    let resource_type = "tool";
    let resource_id = "myapi";
    let version = "v1.0";
    
    // Method 1: Constructor
    let trn1 = Trn::new(platform, scope, resource_type, resource_id, version).unwrap();
    
    // Method 2: Builder
    let trn2 = TrnBuilder::new()
        .platform(platform)
        .scope(scope)
        .resource_type(resource_type)
        .resource_id(resource_id)
        .version(version)
        .build()
        .unwrap();
    
    // Method 3: Parsing
    let trn_string = format!("trn:{}:{}:{}:{}:{}", platform, scope, resource_type, resource_id, version);
    let trn3 = Trn::parse(&trn_string).unwrap();
    
    // All should be identical
    assert_eq!(trn1.to_string(), trn2.to_string());
    assert_eq!(trn2.to_string(), trn3.to_string());
    assert_eq!(trn3.to_string(), trn1.to_string());
    
    // All should be valid
    assert!(trn1.is_valid());
    assert!(trn2.is_valid());
    assert!(trn3.is_valid());
    
    // All should have same properties
    assert_eq!(trn1.platform(), trn2.platform());
    assert_eq!(trn2.platform(), trn3.platform());
    assert_eq!(trn1.scope(), trn2.scope());
    assert_eq!(trn2.scope(), trn3.scope());
    assert_eq!(trn1.resource_type(), trn2.resource_type());
    assert_eq!(trn2.resource_type(), trn3.resource_type());
    assert_eq!(trn1.resource_id(), trn2.resource_id());
    assert_eq!(trn2.resource_id(), trn3.resource_id());
    assert_eq!(trn1.version(), trn2.version());
    assert_eq!(trn2.version(), trn3.version());
} 