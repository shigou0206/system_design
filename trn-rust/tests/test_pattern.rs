use trn_rust::*;

// =================================
// Basic Pattern Matching Tests
// =================================

#[test]
fn test_matches_pattern_basic() {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    // Test exact match
    let matcher1 = TrnMatcher::new("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert!(matcher1.matches(trn));
    
    // Test wildcard matches
    let matcher2 = TrnMatcher::new("trn:user:*:tool:openapi:github-api:v1.0").unwrap();
    assert!(matcher2.matches(trn));
    
    let matcher3 = TrnMatcher::new("trn:*:alice:tool:openapi:github-api:v1.0").unwrap();
    assert!(matcher3.matches(trn));
    
    let matcher4 = TrnMatcher::new("trn:user:alice:tool:*:github-api:v1.0").unwrap();
    assert!(matcher4.matches(trn));
    
    // Test non-matches
    let matcher5 = TrnMatcher::new("trn:user:bob:tool:openapi:github-api:v1.0").unwrap();
    assert!(!matcher5.matches(trn));
    
    let matcher6 = TrnMatcher::new("trn:org:alice:tool:openapi:github-api:v1.0").unwrap();
    assert!(!matcher6.matches(trn));
}

#[test]
fn test_matches_pattern_wildcards() {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    // Single component wildcards
    assert!(TrnMatcher::new("trn:*:alice:tool:openapi:github-api:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:user:*:tool:openapi:github-api:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:user:alice:*:openapi:github-api:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:user:alice:tool:*:github-api:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:user:alice:tool:openapi:*:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:user:alice:tool:openapi:github-api:*").unwrap().matches(trn));
    
    // Multiple wildcards
    assert!(TrnMatcher::new("trn:*:*:tool:openapi:github-api:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:user:alice:*:*:github-api:v1.0").unwrap().matches(trn));
    assert!(TrnMatcher::new("trn:*:*:*:*:*:*").unwrap().matches(trn));
}

#[test]
fn test_find_matching_trns() {
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:user:bob:tool:openapi:slack-api:v1.0".to_string(),
        "trn:org:company:model:bert:language-model:v2.0".to_string(),
        "trn:aiplatform:model:gpt:text-generator:latest".to_string(),
    ];
    
    // Match all user tools
    let user_tools = find_matching_trns(&trns, "trn:user:*:tool:*:*:*");
    assert_eq!(user_tools.len(), 2);
    
    // Match specific user
    let alice_trns = find_matching_trns(&trns, "trn:user:alice:*:*:*:*");
    assert_eq!(alice_trns.len(), 1);
    assert!(alice_trns[0].contains("alice"));
    
    // Match all models
    let models = find_matching_trns(&trns, "trn:*:*:model:*:*:*");
    assert_eq!(models.len(), 2);
    
    // Match v1.0 versions
    let v1_trns = find_matching_trns(&trns, "trn:*:*:*:*:*:v1.0");
    assert_eq!(v1_trns.len(), 2);
}

// =================================
// TrnMatcher Tests
// =================================

#[test]
fn test_trn_matcher_new() {
    let matcher = TrnMatcher::new("trn:user:*:tool:*:*:*").unwrap();
    assert_eq!(matcher.pattern_count(), 1);
    
    assert!(matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0"));
    assert!(matcher.matches("trn:user:bob:tool:python:data-processor:v2.0"));
    assert!(!matcher.matches("trn:org:company:tool:openapi:api-gateway:v1.0"));
}

#[test]
fn test_trn_matcher_empty() {
    let mut matcher = TrnMatcher::empty();
    assert_eq!(matcher.pattern_count(), 0);
    
    // Should not match anything when empty
    assert!(!matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0"));
    
    // Add a pattern
    matcher.add_pattern("trn:user:*:tool:*:*:*").unwrap();
    assert_eq!(matcher.pattern_count(), 1);
    assert!(matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0"));
}

#[test]
fn test_trn_matcher_with_pattern() {
    let matcher = TrnMatcher::with_pattern("trn:*:*:model:*:*:*").unwrap();
    assert_eq!(matcher.pattern_count(), 1);
    
    assert!(matcher.matches("trn:aiplatform:model:bert:base-model:v1.0"));
    assert!(matcher.matches("trn:org:company:model:gpt:text-generator:latest"));
    assert!(!matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0"));
}

#[test]
fn test_trn_matcher_multiple_patterns() {
    let mut matcher = TrnMatcher::empty();
    matcher.add_pattern("trn:user:*:tool:*:*:*").unwrap();
    matcher.add_pattern("trn:*:*:model:*:*:*").unwrap();
    assert_eq!(matcher.pattern_count(), 2);
    
    // Should match user tools
    assert!(matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0"));
    
    // Should match models
    assert!(matcher.matches("trn:aiplatform:model:bert:base-model:v1.0"));
    
    // Should not match org tools
    assert!(!matcher.matches("trn:org:company:tool:python:data-processor:v1.0"));
}

#[test]
fn test_trn_matcher_matches_pattern_by_index() {
    let mut matcher = TrnMatcher::empty();
    matcher.add_pattern("trn:user:*:tool:*:*:*").unwrap();
    matcher.add_pattern("trn:*:*:model:*:*:*").unwrap();
    
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    assert!(matcher.matches_pattern(trn, 0)); // Matches first pattern
    assert!(!matcher.matches_pattern(trn, 1)); // Doesn't match second pattern
    
    let model_trn = "trn:aiplatform:model:bert:base-model:v1.0";
    assert!(!matcher.matches_pattern(model_trn, 0)); // Doesn't match first pattern
    assert!(matcher.matches_pattern(model_trn, 1)); // Matches second pattern
}

#[test]
fn test_trn_matcher_matching_patterns() {
    let mut matcher = TrnMatcher::empty();
    matcher.add_pattern("trn:user:*:*:*:*:*").unwrap();
    matcher.add_pattern("trn:*:alice:*:*:*:*").unwrap();
    matcher.add_pattern("trn:*:*:tool:*:*:*").unwrap();
    
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    let matching = matcher.matching_patterns(trn);
    assert_eq!(matching.len(), 3); // Should match all three patterns
}

#[test]
fn test_trn_matcher_filter_trns() {
    let matcher = TrnMatcher::new("trn:user:*:tool:*:*:*").unwrap();
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:user:bob:tool:python:data-processor:v2.0".to_string(),
        "trn:org:company:model:bert:language-model:v1.0".to_string(),
        "trn:aiplatform:model:gpt:text-generator:latest".to_string(),
    ];
    
    let filtered = matcher.filter_trns(&trns);
    assert_eq!(filtered.len(), 2); // Only user tools should match
    assert!(filtered[0].contains("alice"));
    assert!(filtered[1].contains("bob"));
}

#[test]
fn test_trn_matcher_clear() {
    let mut matcher = TrnMatcher::new("trn:user:*:tool:*:*:*").unwrap();
    assert_eq!(matcher.pattern_count(), 1);
    
    matcher.clear();
    assert_eq!(matcher.pattern_count(), 0);
    assert!(!matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0"));
}

// =================================
// Pattern Validation Tests
// =================================

#[test]
fn test_pattern_validation() {
    // Valid patterns
    assert!(TrnMatcher::new("trn:*:*:*:*:*:*").is_ok());
    assert!(TrnMatcher::new("trn:user:alice:tool:openapi:github-api:v1.0").is_ok());
    assert!(TrnMatcher::new("trn:user:*:tool:*:*:*").is_ok());
    
    // Invalid patterns - too few components
    assert!(TrnMatcher::new("trn:user:alice").is_err());
    assert!(TrnMatcher::new("trn:*:*").is_err());
    
    // Invalid patterns - wrong prefix
    assert!(TrnMatcher::new("invalid:*:*:*:*:*:*").is_err());
    assert!(TrnMatcher::new("not-trn:user:alice:tool:openapi:github-api:v1.0").is_err());
}

// =================================
// Complex Pattern Scenarios
// =================================

#[test]
fn test_complex_pattern_scenarios() {
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
        "trn:user:alice:tool:python:data-analysis:v2.0".to_string(),
        "trn:user:bob:tool:openapi:slack-api:v1.0".to_string(),
        "trn:org:company:tool:python:ml-pipeline:v1.0".to_string(),
        "trn:org:company:model:bert:classifier:v2.0".to_string(),
        "trn:aiplatform:model:gpt:text-generator:latest".to_string(),
    ];
    
    // Test platform-specific patterns
    let user_trns = find_matching_trns(&trns, "trn:user:*:*:*:*:*");
    assert_eq!(user_trns.len(), 3);
    
    let org_trns = find_matching_trns(&trns, "trn:org:*:*:*:*:*");
    assert_eq!(org_trns.len(), 2);
    
    let system_trns = find_matching_trns(&trns, "trn:aiplatform:*:*:*:*:*");
    assert_eq!(system_trns.len(), 1);
    
    // Test resource type patterns
    let tools = find_matching_trns(&trns, "trn:*:*:tool:*:*:*");
    assert_eq!(tools.len(), 4);
    
    let models = find_matching_trns(&trns, "trn:*:*:model:*:*:*");
    assert_eq!(models.len(), 2);
    
    // Test specific user patterns
    let alice_tools = find_matching_trns(&trns, "trn:user:alice:*:*:*:*");
    assert_eq!(alice_tools.len(), 2);
    
    let bob_tools = find_matching_trns(&trns, "trn:user:bob:*:*:*:*");
    assert_eq!(bob_tools.len(), 1);
    
    // Test tool type patterns
    let openapi_tools = find_matching_trns(&trns, "trn:*:*:tool:openapi:*:*");
    assert_eq!(openapi_tools.len(), 2);
    
    let python_tools = find_matching_trns(&trns, "trn:*:*:tool:python:*:*");
    assert_eq!(python_tools.len(), 2);
    
    // Test version patterns
    let v1_trns = find_matching_trns(&trns, "trn:*:*:*:*:*:v1.0");
    assert_eq!(v1_trns.len(), 3);
    
    let v2_trns = find_matching_trns(&trns, "trn:*:*:*:*:*:v2.0");
    assert_eq!(v2_trns.len(), 2);
    
    let latest_trns = find_matching_trns(&trns, "trn:*:*:*:*:*:latest");
    assert_eq!(latest_trns.len(), 1);
}

// =================================
// Edge Cases and Error Handling
// =================================

#[test]
fn test_pattern_edge_cases() {
    // Test with empty TRN list
    let empty_trns: Vec<String> = vec![];
    let matches = find_matching_trns(&empty_trns, "trn:*:*:*:*:*:*");
    assert_eq!(matches.len(), 0);
    
    // Test with no matching TRNs
    let trns = vec![
        "trn:user:alice:tool:openapi:github-api:v1.0".to_string(),
    ];
    let no_matches = find_matching_trns(&trns, "trn:org:*:*:*:*:*");
    assert_eq!(no_matches.len(), 0);
    
    // Test with all matching TRNs
    let all_matches = find_matching_trns(&trns, "trn:*:*:*:*:*:*");
    assert_eq!(all_matches.len(), 1);
}

#[test]
fn test_pattern_with_special_characters() {
    let trns = vec![
        "trn:org:user-123:tool:openapi:apiv2:v1.0".to_string(),
        "trn:user:username:tool:python:scriptv2:v2.0".to_string(),
    ];
    
    // Should match patterns with special characters (use org platform for dashes)
    let matches1 = find_matching_trns(&trns, "trn:org:user-123:*:*:*:*");
    assert_eq!(matches1.len(), 1);
    
    let matches2 = find_matching_trns(&trns, "trn:user:username:*:*:*:*");
    assert_eq!(matches2.len(), 1);
    
    // Wildcard should match both
    let all_matches = find_matching_trns(&trns, "trn:*:*:*:*:*:*");
    assert_eq!(all_matches.len(), 2);
}

// =================================
// Performance Tests
// =================================

#[test]
fn test_pattern_matching_performance() {
    // Create a large set of TRNs for performance testing
    let trns: Vec<String> = (0..1000)
        .map(|i| format!("trn:user:user{}:tool:openapi:api{}:v1.0", i, i))
        .collect();
    
    // Test pattern matching performance
    let matches = find_matching_trns(&trns, "trn:user:*:tool:*:*:*");
    assert_eq!(matches.len(), 1000); // All should match
    
    // Test specific pattern
    let specific_matches = find_matching_trns(&trns, "trn:user:user500:*:*:*:*");
    assert_eq!(specific_matches.len(), 1);
}

#[test]
fn test_matcher_performance() {
    let matcher = TrnMatcher::new("trn:user:*:tool:*:*:*").unwrap();
    
    // Test with many TRNs
    for i in 0..100 {
        let trn = format!("trn:user:user{}:tool:openapi:api{}:v1.0", i, i);
        assert!(matcher.matches(&trn));
    }
}

// =================================
// Trn Object Pattern Matching
// =================================

#[test]
fn test_trn_object_pattern_matching() {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    
    // Test the matches_pattern method on Trn objects
    assert!(trn.matches_pattern("trn:user:*:tool:*:*:*"));
    assert!(trn.matches_pattern("trn:*:alice:*:*:*:*"));
    assert!(trn.matches_pattern("trn:*:*:*:*:github-api:*"));
    assert!(!trn.matches_pattern("trn:org:*:*:*:*:*"));
    assert!(!trn.matches_pattern("trn:user:bob:*:*:*:*"));
}

// =================================
// Mixed Pattern Tests
// =================================

#[test]
fn test_mixed_pattern_combinations() {
    let mut matcher = TrnMatcher::empty();
    
    // Add multiple different patterns
    matcher.add_pattern("trn:user:alice:*:*:*:*").unwrap();
    matcher.add_pattern("trn:*:*:model:*:*:*").unwrap();
    matcher.add_pattern("trn:*:*:*:*:*:latest").unwrap();
    
    // Test various TRNs
    assert!(matcher.matches("trn:user:alice:tool:openapi:github-api:v1.0")); // Matches first pattern
    assert!(matcher.matches("trn:aiplatform:model:bert:base-model:v1.0")); // Matches second pattern  
    assert!(matcher.matches("trn:user:bob:tool:python:script:latest")); // Matches third pattern
    assert!(matcher.matches("trn:user:alice:model:bert:classifier:latest")); // Matches multiple patterns
    
    // Should not match
    assert!(!matcher.matches("trn:user:bob:tool:openapi:github-api:v1.0")); // Doesn't match any pattern
    assert!(!matcher.matches("trn:org:company:tool:python:script:v1.0")); // Doesn't match any pattern
}

// =================================
// Consistency Tests
// =================================

#[test]
fn test_pattern_matching_consistency() {
    let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    let pattern = "trn:user:*:tool:*:*:*";
    
    // These should all give the same result
    let basic_matcher = TrnMatcher::new(pattern).unwrap();
    assert_eq!(basic_matcher.matches(trn), true);
    
    let matcher = TrnMatcher::new(pattern).unwrap();
    assert_eq!(matcher.matches(trn), true);
    
    let trns = vec![trn.to_string()];
    let found = find_matching_trns(&trns, pattern);
    assert_eq!(found.len(), 1);
    
    let filtered = matcher.filter_trns(&trns);
    assert_eq!(filtered.len(), 1);
} 