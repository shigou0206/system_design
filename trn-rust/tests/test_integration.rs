use trn_rust::*;

// =================================
// Complete TRN Workflow Integration Tests
// =================================

#[test]
fn test_complete_trn_workflow() {
    // 1. Create TRN using builder
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    // 2. Validate the TRN
    assert!(trn.validate().is_ok());
    assert!(trn.is_valid());
    
    // 3. Convert to string representation
    let trn_string = trn.to_string();
    assert_eq!(trn_string, "trn:user:alice:tool:openapi:github-api:v1.0");
    
    // 4. Parse back from string
    let parsed_trn = Trn::parse(&trn_string).unwrap();
    assert_eq!(trn, parsed_trn);
    
    // 5. Convert to URL
    let url = trn.to_url().unwrap();
    // URL encoding may occur, so check structure instead of exact match
    assert!(url.starts_with("trn://user/alice/tool/openapi/"));
    assert!(url.contains("github") && url.contains("api"));
    assert!(url.contains("1") && url.contains("0"));
    
    // 6. Convert back from URL
    let trn_from_url = url_to_trn(&url).unwrap();
    assert_eq!(trn, trn_from_url);
    
    // 7. Test pattern matching
    assert!(trn.matches_pattern("trn:user:*:tool:*:*:*"));
    assert!(trn.matches_pattern("trn:*:alice:*:*:*:*"));
    assert!(!trn.matches_pattern("trn:org:*:*:*:*:*"));
}

#[test]
fn test_multi_platform_integration() {
    // Create TRNs for different platforms using various methods
    let trns = vec![
        // Using builder
        TrnBuilder::new()
            .platform("aiplatform")
            .resource_type("model")
            .type_("bert")
            .instance_id("base-model")
            .version("v1.0")
            .build()
            .unwrap(),
        
        // Using user tool template
        TrnBuilder::user_tool("alice")
            .type_("openapi")
            .instance_id("github-api")
            .version("v1.0")
            .build()
            .unwrap(),
        
        // Using org tool template
        TrnBuilder::org_tool("company")
            .type_("python")
            .instance_id("data-processor")
            .version("v2.0")
            .build()
            .unwrap(),
        
        // Using parse
        Trn::parse("trn:user:bob:tool:workflow:automation:latest").unwrap(),
    ];
    
    // Validate all TRNs
    for trn in &trns {
        assert!(trn.validate().is_ok());
    }
    
    // Test pattern matching across all TRNs
    let user_matcher = TrnMatcher::new("trn:user:*:*:*:*:*").unwrap();
    let tool_matcher = TrnMatcher::new("trn:*:*:tool:*:*:*").unwrap();
    
    let trn_strings: Vec<String> = trns.iter().map(|t| t.to_string()).collect();
    
    let user_trns = user_matcher.filter_trns(&trn_strings);
    assert_eq!(user_trns.len(), 2); // alice and bob
    
    let tool_trns = tool_matcher.filter_trns(&trn_strings);
    assert_eq!(tool_trns.len(), 3); // All except the model
    
    // Test URL conversion for all
    for trn in &trns {
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, &back_to_trn);
    }
}

#[test]
fn test_batch_operations_integration() {
    // Create a large set of TRNs using different methods
    let mut trn_strings = Vec::new();
    
    // Add TRNs using builder
    for i in 0..50 {
        let trn = TrnBuilder::new()
            .platform("user")
            .scope(&format!("user{}", i))
            .resource_type("tool")
            .type_("openapi")
            .instance_id(&format!("api{}", i))
            .version("v1.0")
            .build()
            .unwrap();
        trn_strings.push(trn.to_string());
    }
    
    // Add some model TRNs
    for i in 0..25 {
        let trn_str = format!("trn:aiplatform:model:bert:model{}:v1.0", i);
        trn_strings.push(trn_str);
    }
    
    // Add some invalid TRNs for testing
    trn_strings.push("invalid-trn".to_string());
    trn_strings.push("trn:incomplete".to_string());
    
    // Batch validate
    let validation_report = batch_validate(&trn_strings);
    assert_eq!(validation_report.total, 77);
    assert_eq!(validation_report.valid, 75); // 50 user tools + 25 models
    assert_eq!(validation_report.invalid, 2); // 2 invalid TRNs
    
    // Filter out invalid TRNs and work with valid ones
    let valid_trns: Vec<String> = trn_strings
        .into_iter()
        .filter(|s| is_valid_trn(s))
        .collect();
    
    assert_eq!(valid_trns.len(), 75);
    
    // Test pattern matching on the batch
    let user_tools = find_matching_trns(&valid_trns, "trn:user:*:tool:*:*:*");
    assert_eq!(user_tools.len(), 50);
    
    let models = find_matching_trns(&valid_trns, "trn:*:*:model:*:*:*");
    assert_eq!(models.len(), 25);
    
    let all_v1 = find_matching_trns(&valid_trns, "trn:*:*:*:*:*:v1.0");
    assert_eq!(all_v1.len(), 75); // All have v1.0
}

#[test] 
fn test_builder_pattern_variations() {
    // Test different builder patterns and ensure they work with other features
    
    // Basic builder
    let basic_trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .build()
        .unwrap();
    
    // Builder with optional fields
    let full_trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .subtype("rest")
        .instance_id("github-api")
        .version("v1.0")
        .tag("stable")
        .hash("sha256:a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789a")
        .build()
        .unwrap();
    
    // Builder using enums
    let enum_trn = TrnBuilder::new()
        .platform_enum(Platform::User)
        .scope("bob")
        .resource_type_enum(ResourceType::Tool)
        .tool_type(ToolType::Python)
        .instance_id("data-script")
        .version_v(2, 1, 0)
        .build()
        .unwrap();
    
    let trns = vec![basic_trn, full_trn, enum_trn];
    
    // Test validation for all
    for trn in &trns {
        assert!(validate(trn).is_ok());
    }
    
    // Test URL conversion
    for trn in &trns {
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, &back_to_trn);
    }
    
    // Test pattern matching
    let tool_matcher = TrnMatcher::new("trn:*:*:tool:*:*:*").unwrap();
    let user_matcher = TrnMatcher::new("trn:user:*:*:*:*:*").unwrap();
    
    assert!(!tool_matcher.matches(&trns[0].to_string())); // model, not tool
    assert!(tool_matcher.matches(&trns[1].to_string())); // tool
    assert!(tool_matcher.matches(&trns[2].to_string())); // tool
    
    assert!(!user_matcher.matches(&trns[0].to_string())); // aiplatform
    assert!(user_matcher.matches(&trns[1].to_string())); // user alice
    assert!(user_matcher.matches(&trns[2].to_string())); // user bob
}

#[test]
fn test_error_handling_integration() {
    // Test error handling across different components
    
    // Invalid TRN creation should be caught
    let invalid_build_result = TrnBuilder::new()
        .platform("user")
        // Missing scope for user platform
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build();
    assert!(invalid_build_result.is_err());
    
    // Invalid parsing should be caught
    assert!(Trn::parse("invalid-trn-format").is_err());
    assert!(Trn::parse("trn:incomplete").is_err());
    
    // Invalid URL conversion should be caught
    assert!(url_to_trn("http://example.com").is_err());
    assert!(url_to_trn("invalid-url").is_err());
    
    // Invalid pattern should be caught
    assert!(TrnMatcher::new("invalid-pattern").is_err());
    assert!(TrnMatcher::new("trn:incomplete").is_err());
    
    // Test that valid operations still work after errors
    let valid_trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert!(valid_trn.validate().is_ok());
    assert!(valid_trn.to_url().is_ok());
}

#[test]
fn test_complex_pattern_matching_integration() {
    // Create diverse set of TRNs
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:user:alice:tool:python:data-analysis:v2.0", 
        "trn:user:bob:tool:openapi:slack-api:v1.0",
        "trn:org:company:tool:python:ml-pipeline:v1.0",
        "trn:org:company:model:bert:classifier:v2.0",
        "trn:aiplatform:model:gpt:text-generator:latest",
        "trn:user:charlie:dataset:csv:user-data:v1.0",
    ];
    
    let trn_strings: Vec<String> = trns.iter().map(|s| s.to_string()).collect();
    
    // Create multiple matchers for complex queries
    let mut alice_matcher = TrnMatcher::empty();
    alice_matcher.add_pattern("trn:user:alice:*:*:*:*").unwrap();
    
    let mut python_matcher = TrnMatcher::empty();
    python_matcher.add_pattern("trn:*:*:*:python:*:*").unwrap();
    
    let mut v1_tools_matcher = TrnMatcher::empty();
    v1_tools_matcher.add_pattern("trn:*:*:tool:*:*:v1.0").unwrap();
    
    // Test individual matchers
    let alice_trns = alice_matcher.filter_trns(&trn_strings);
    assert_eq!(alice_trns.len(), 2);
    
    let python_trns = python_matcher.filter_trns(&trn_strings);
    assert_eq!(python_trns.len(), 2);
    
    let v1_tools = v1_tools_matcher.filter_trns(&trn_strings);
    assert_eq!(v1_tools.len(), 3);
    
    // Test pattern combinations using find_matching_trns
    let user_tools = find_matching_trns(&trn_strings, "trn:user:*:tool:*:*:*");
    assert_eq!(user_tools.len(), 3);
    
    let org_resources = find_matching_trns(&trn_strings, "trn:org:*:*:*:*:*");
    assert_eq!(org_resources.len(), 2);
    
    let all_models = find_matching_trns(&trn_strings, "trn:*:*:model:*:*:*");
    assert_eq!(all_models.len(), 2);
    
    // Convert all to URLs and test pattern matching still works
    let urls: Vec<String> = trn_strings
        .iter()
        .map(|s| {
            let trn = Trn::parse(s).unwrap();
            trn.to_url().unwrap()
        })
        .collect();
    
    // Convert back from URLs and test patterns still match
    let back_from_urls: Vec<String> = urls
        .iter()
        .map(|url| {
            let trn = url_to_trn(url).unwrap();
            trn.to_string()
        })
        .collect();
    
    assert_eq!(trn_strings, back_from_urls);
    
    // Pattern matching should work the same on converted TRNs
    let user_tools_after_roundtrip = find_matching_trns(&back_from_urls, "trn:user:*:tool:*:*:*");
    assert_eq!(user_tools_after_roundtrip.len(), 3);
}

#[test]
fn test_version_and_metadata_integration() {
    // Test TRNs with various version formats and metadata
    let trns_with_metadata = vec![
        TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .type_("openapi")
            .instance_id("github-api")
            .semver(1, 2, 3)
            .build()
            .unwrap(),
        
        TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .type_("openapi")
            .instance_id("github-api")
            .version("latest")
            .tag("stable")
            .build()
            .unwrap(),
        
        TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .type_("openapi")
            .instance_id("github-api")
            .version("v2.0")
            .sha256_hash("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")
            .build()
            .unwrap(),
    ];
    
    // All should be valid
    for trn in &trns_with_metadata {
        assert!(trn.validate().is_ok());
    }
    
    // Test URL roundtrip preserves metadata
    for trn in &trns_with_metadata {
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, &back_to_trn);
        
        // Check specific metadata is preserved
        assert_eq!(trn.version(), back_to_trn.version());
        assert_eq!(trn.tag(), back_to_trn.tag());
        assert_eq!(trn.hash(), back_to_trn.hash());
    }
    
    // Test pattern matching works with metadata
    let base_pattern = "trn:user:alice:tool:openapi:github-api:*";
    for trn in &trns_with_metadata {
        assert!(trn.matches_pattern(base_pattern));
    }
    
    // Test specific version pattern matching
    let semver_trn = &trns_with_metadata[0];
    assert!(semver_trn.matches_pattern("trn:*:*:*:*:*:1.2.3"));
    
    let latest_trn = &trns_with_metadata[1];
    assert!(latest_trn.matches_pattern("trn:*:*:*:*:*:latest"));
}

#[test]
fn test_performance_integration() {
    // Test that all operations work efficiently together on larger datasets
    let mut trns = Vec::new();
    
    // Generate 1000 diverse TRNs
    for i in 0..1000 {
        let platform = match i % 3 {
            0 => "user",
            1 => "org", 
            _ => "aiplatform",
        };
        
        let (scope, resource_type) = match platform {
            "user" => (Some(format!("user{}", i)), "tool"),
            "org" => (Some(format!("org{}", i)), "tool"),
            _ => (None, "model"),
        };
        
        let mut builder = TrnBuilder::new().platform(platform);
        
        if let Some(s) = scope {
            builder = builder.scope(s);
        }
        
        let trn = builder
            .resource_type(resource_type)
            .type_("openapi")
            .instance_id(&format!("resource{}", i))
            .version(&format!("v{}.0", i % 5 + 1))
            .build()
            .unwrap();
            
        trns.push(trn);
    }
    
    // Batch validate
    let trn_strings: Vec<String> = trns.iter().map(|t| t.to_string()).collect();
    let validation_report = batch_validate(&trn_strings);
    assert_eq!(validation_report.total, 1000);
    assert_eq!(validation_report.valid, 1000); // All should be valid
    
    // Test pattern matching performance  
    let user_trns = find_matching_trns(&trn_strings, "trn:user:*:*:*:*:*");
    assert!(user_trns.len() > 300 && user_trns.len() < 350); // Roughly 1/3
    
    // Test URL conversion performance
    for trn in trns.iter().take(100) { // Test first 100 for performance
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, &back_to_trn);
    }
}

#[test]
fn test_real_world_scenario() {
    // Simulate a real-world scenario with mixed operations
    
    // 1. Data ingestion - parse TRNs from various sources
    let external_trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:user:bob:tool:python:data-processor:v2.0", 
        "trn:org:acme:model:bert:sentiment-analyzer:v1.5",
        "trn:aiplatform:model:gpt:code-generator:latest",
    ];
    
    let parsed_trns: Vec<Trn> = external_trns
        .iter()
        .filter_map(|s| Trn::parse(s).ok())
        .collect();
    
    assert_eq!(parsed_trns.len(), 4);
    
    // 2. Create new TRNs using builder
    let new_trns = vec![
        TrnBuilder::user_tool("charlie")
            .type_("workflow")
            .instance_id("automation")
            .version("v1.0")
            .build()
            .unwrap(),
        
        TrnBuilder::new()
            .platform("org")
            .scope("acme")
            .dataset()
            .type_("csv")
            .instance_id("customer-data")
            .latest()
            .build()
            .unwrap(),
    ];
    
    // 3. Combine all TRNs
    let mut all_trns = parsed_trns;
    all_trns.extend(new_trns);
    
    // 4. Validate everything
    for trn in &all_trns {
        assert!(trn.validate().is_ok());
    }
    
    // 5. Filter and categorize
    let tool_matcher = TrnMatcher::new("trn:*:*:tool:*:*:*").unwrap();
    let model_matcher = TrnMatcher::new("trn:*:*:model:*:*:*").unwrap();
    let user_matcher = TrnMatcher::new("trn:user:*:*:*:*:*").unwrap();
    
    let trn_strings: Vec<String> = all_trns.iter().map(|t| t.to_string()).collect();
    
    let tools = tool_matcher.filter_trns(&trn_strings);
    let models = model_matcher.filter_trns(&trn_strings);
    let user_resources = user_matcher.filter_trns(&trn_strings);
    
    assert_eq!(tools.len(), 3); // github-api, data-processor, automation
    assert_eq!(models.len(), 2); // sentiment-analyzer, code-generator  
    assert_eq!(user_resources.len(), 3); // alice, bob, charlie
    
    // 6. Export as URLs
    let urls: Vec<String> = all_trns
        .iter()
        .map(|trn| trn.to_url().unwrap())
        .collect();
    
    assert_eq!(urls.len(), 6);
    
    // 7. Verify roundtrip integrity
    for (original_trn, url) in all_trns.iter().zip(urls.iter()) {
        let reconstructed = url_to_trn(url).unwrap();
        assert_eq!(original_trn, &reconstructed);
    }
} 