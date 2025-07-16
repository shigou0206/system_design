use trn_rust::{TrnBuilder, Trn};

#[test]
fn test_basic_builder() {
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), "alice");
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.resource_id(), "myapi");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_builder_all_platforms() {
    let test_cases = vec![
        ("user", "alice", "user platform"),
        ("org", "company", "org platform"),
        ("aiplatform", "system", "ai platform"),
        ("custom", "scope", "custom platform"),
    ];
    
    for (platform, scope, description) in test_cases {
        let trn = TrnBuilder::new()
            .platform(platform)
            .scope(scope)
            .resource_type("tool")
            .resource_id("myapi")
            .version("v1.0")
            .build()
            .unwrap();
        
        assert_eq!(trn.platform(), platform, "Failed for {}", description);
        assert_eq!(trn.scope(), scope, "Failed for {}", description);
    }
}

#[test]
fn test_builder_all_resource_types() {
    let resource_types = vec!["tool", "model", "dataset", "pipeline", "custom-type"];
    
    for resource_type in resource_types {
        let trn = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type(resource_type)
            .resource_id("myapi")
            .version("v1.0")
            .build()
            .unwrap();
        
        assert_eq!(trn.resource_type(), resource_type);
    }
}

#[test]
fn test_builder_different_versions() {
    let versions = vec![
        "v1.0",
        "latest",
        "dev",
        "main",
        "1.2.3",
        "v2.0-beta",
        "feature-branch",
    ];
    
    for version in versions {
        let trn = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .resource_id("myapi")
            .version(version)
            .build()
            .unwrap();
        
        assert_eq!(trn.version(), version);
    }
}

#[test]
fn test_builder_missing_required_fields() {
    // Missing platform
    let result = TrnBuilder::new()
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail without platform");
    
    // Missing scope
    let result = TrnBuilder::new()
        .platform("user")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail without scope");
    
    // Missing resource_type
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_id("myapi")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail without resource_type");
    
    // Missing resource_id
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail without resource_id");
    
    // Missing version
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .build();
    assert!(result.is_err(), "Should fail without version");
}

#[test]
fn test_builder_empty_fields() {
    // Empty platform
    let result = TrnBuilder::new()
        .platform("")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail with empty platform");
    
    // Empty scope
    let result = TrnBuilder::new()
        .platform("user")
        .scope("")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail with empty scope");
    
    // Empty resource_type
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("")
        .resource_id("myapi")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail with empty resource_type");
    
    // Empty resource_id
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("")
        .version("v1.0")
        .build();
    assert!(result.is_err(), "Should fail with empty resource_id");
    
    // Empty version
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("")
        .build();
    assert!(result.is_err(), "Should fail with empty version");
}

#[test]
fn test_builder_method_chaining() {
    // Test that all methods return self for chaining
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.to_string(), "trn:user:alice:tool:myapi:v1.0");
}

#[test]
fn test_builder_overwrite_fields() {
    // Test that setting a field twice uses the last value
    let trn = TrnBuilder::new()
        .platform("user")
        .platform("org")  // Overwrite
        .scope("alice")
        .scope("company")  // Overwrite
        .resource_type("tool")
        .resource_type("model")  // Overwrite
        .resource_id("myapi")
        .resource_id("bert")  // Overwrite
        .version("v1.0")
        .version("v2.0")  // Overwrite
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), "company");
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.resource_id(), "bert");
    assert_eq!(trn.version(), "v2.0");
}

#[test]
fn test_builder_complex_identifiers() {
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice-smith")
        .resource_type("custom-type")
        .resource_id("my_api_v2")
        .version("v1.0-beta")
        .build()
        .unwrap();
    
    assert_eq!(trn.scope(), "alice-smith");
    assert_eq!(trn.resource_type(), "custom-type");
    assert_eq!(trn.resource_id(), "my_api_v2");
    assert_eq!(trn.version(), "v1.0-beta");
}

#[test]
fn test_builder_with_different_inputs() {
    // Test that builder works with different input types
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("latest")
        .build()
        .unwrap();
    
    assert_eq!(trn.version(), "latest");
    
    let trn = TrnBuilder::new()
        .platform("org")
        .scope("company")
        .resource_type("model")
        .resource_id("bert")
        .version("dev")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.version(), "dev");
}

#[test]
fn test_builder_validation() {
    // Valid TRN should pass validation
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .resource_id("myapi")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert!(trn.is_valid());
    
    // Test that builder validates during build
    // (Invalid characters would be caught during validation)
}

#[test]
fn test_builder_modification() {
    let original = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    // Create a new TRN with modified version
    let modified = TrnBuilder::new()
        .platform(original.platform())
        .scope(original.scope())
        .resource_type(original.resource_type())
        .resource_id(original.resource_id())
        .version("v2.0")
        .build()
        .unwrap();
    
    assert_eq!(modified.platform(), "user");
    assert_eq!(modified.scope(), "alice");
    assert_eq!(modified.resource_type(), "tool");
    assert_eq!(modified.resource_id(), "myapi");
    assert_eq!(modified.version(), "v2.0");
}

#[test]
fn test_builder_clone() {
    let builder = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool");
    
    let trn1 = builder.clone()
        .resource_id("api1")
        .version("v1.0")
        .build()
        .unwrap();
    
    let trn2 = builder.clone()
        .resource_id("api2")
        .version("v2.0")
        .build()
        .unwrap();
    
    assert_eq!(trn1.resource_id(), "api1");
    assert_eq!(trn1.version(), "v1.0");
    assert_eq!(trn2.resource_id(), "api2");
    assert_eq!(trn2.version(), "v2.0");
}

#[test]
fn test_builder_to_string_before_build() {
    let _builder = TrnBuilder::new()
        .platform("user")
        .scope("alice");
    
    // Should be able to inspect builder state
    // (This would depend on builder implementation)
}

#[test]
fn test_builder_edge_cases() {
    // Test minimum valid components (with minimum required lengths)
    let trn = TrnBuilder::new()
        .platform("aa")  // Platform needs at least 2 chars
        .scope("b")
        .resource_type("tool")  // Use supported resource type
        .resource_id("d")
        .version("e")
        .build()
        .unwrap();
    
    assert_eq!(trn.to_string(), "trn:aa:b:tool:d:e");
    
    // Test with special characters
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice-123")
        .resource_type("custom-type")
        .resource_id("my_api.v2")
        .version("v1.0-beta.1")
        .build()
        .unwrap();
    
    assert!(trn.scope().contains("-"));
    assert!(trn.resource_type().contains("-")); // "custom-type" contains "-"
    assert!(trn.resource_id().contains("_"));
    assert!(trn.resource_id().contains("."));
    assert!(trn.version().contains("-"));
    assert!(trn.version().contains("."));
}

#[test]
fn test_builder_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Build 1000 TRNs
    for i in 0..1000 {
        let _trn = TrnBuilder::new()
            .platform("user")
            .scope(&format!("user{}", i))
            .resource_type("tool")
            .resource_id(&format!("api{}", i))
            .version("v1.0")
            .build()
            .unwrap();
    }
    
    let duration = start.elapsed();
    // Should build 1000 TRNs in less than 100ms
    assert!(duration.as_millis() < 100, "Builder too slow: {:?}", duration);
} 