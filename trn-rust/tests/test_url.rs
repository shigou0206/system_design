use trn_rust::*;

// =================================
// Basic URL Conversion Tests
// =================================

#[test]
fn test_trn_to_url_basic() {
    // Test basic TRN to URL conversion
    let trn = Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://aiplatform/model/bert/base-model/v1.0");
}

#[test]
fn test_trn_to_url_with_scope() {
    // Test TRN with scope to URL conversion
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://user/alice/tool/openapi/github-api/v1.0");
}

#[test]
fn test_trn_to_url_with_organization() {
    // Test organization TRN to URL conversion
    let trn = Trn::parse("trn:org:company:model:bert:language-model:v2.0").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://org/company/model/bert/language-model/v2.0");
}

#[test]
fn test_trn_to_url_with_hash() {
    // Test TRN with hash to URL conversion
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://user/alice/tool/openapi/github-api/v1.0?hash=sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
}

// =================================
// URL to TRN Conversion Tests
// =================================

#[test]
fn test_url_to_trn_basic() {
    // Test basic URL to TRN conversion
    let trn = url_to_trn("trn://aiplatform/model/bert/base-model/v1.0").unwrap();
    assert_eq!(trn.platform(), "aiplatform");
    assert_eq!(trn.scope(), None);
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.type_(), "bert");
    assert_eq!(trn.instance_id(), "base-model");
    assert_eq!(trn.version(), "v1.0");
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_url_to_trn_with_scope() {
    // Test URL with scope to TRN conversion
    let trn = url_to_trn("trn://user/alice/tool/openapi/github-api/v1.0").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "github-api");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_url_to_trn_with_hash() {
    // Test URL with hash to TRN conversion
    let trn = url_to_trn("trn://user/alice/tool/openapi/github-api/v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.hash(), Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_url_to_trn_invalid_urls() {
    // Test invalid URL formats
    assert!(url_to_trn("http://example.com").is_err());
    assert!(url_to_trn("invalid-url").is_err());
    assert!(url_to_trn("").is_err());
    assert!(url_to_trn("trn://").is_err()); // Empty path
    assert!(url_to_trn("trn://user").is_err()); // Too few components
}

// =================================
// Roundtrip Conversion Tests
// =================================

#[test]
fn test_roundtrip_conversion_basic() {
    let original_trn = "trn:aiplatform:model:bert:base-model:v1.0";
    
    // TRN -> URL -> TRN
    let trn = Trn::parse(original_trn).unwrap();
    let url = trn.to_url().unwrap();
    let back_to_trn = url_to_trn(&url).unwrap();
    
    assert_eq!(trn, back_to_trn);
    assert_eq!(original_trn, back_to_trn.to_string());
}

#[test]
fn test_roundtrip_conversion_with_scope() {
    let original_trn = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    // TRN -> URL -> TRN
    let trn = Trn::parse(original_trn).unwrap();
    let url = trn.to_url().unwrap();
    let back_to_trn = url_to_trn(&url).unwrap();
    
    assert_eq!(trn, back_to_trn);
    assert_eq!(original_trn, back_to_trn.to_string());
}

#[test]
fn test_roundtrip_conversion_with_hash() {
    let original_trn = "trn:user:alice:tool:openapi:github-api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    
    // TRN -> URL -> TRN
    let trn = Trn::parse(original_trn).unwrap();
    let url = trn.to_url().unwrap();
    let back_to_trn = url_to_trn(&url).unwrap();
    
    assert_eq!(trn, back_to_trn);
    assert_eq!(original_trn, back_to_trn.to_string());
}

#[test]
fn test_roundtrip_conversion_complex() {
    let test_trns = vec![
        "trn:aiplatform:model:bert:base-model:v1.0",
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:org:company:dataset:csv:sales-data:v2.0",
        "trn:user:bob:tool:python:data-processor:latest",
        "trn:aiplatform:model:gpt:text-generator:v3.0@def4561222333444555666",
    ];
    
    for trn_str in test_trns {
        let trn = Trn::parse(trn_str).unwrap();
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        
        assert_eq!(trn, back_to_trn);
        assert_eq!(trn_str, back_to_trn.to_string());
    }
}

// =================================
// URL Validation Tests
// =================================

#[test]
fn test_url_validation() {
    // Valid TRN URLs
    assert!(Trn::parse("trn:aiplatform:model:bert:base-model:v1.0").unwrap().to_url().is_ok());
    assert!(Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap().to_url().is_ok());
    assert!(url_to_trn("trn://user/alice/tool/openapi/github-api/v1.0").is_ok());
    
    // Invalid URLs for TRN conversion
    assert!(url_to_trn("http://example.com/api").is_err());
    assert!(url_to_trn("ftp://example.com").is_err());
    assert!(url_to_trn("invalid-scheme://test").is_err());
}

// =================================
// Special Characters in URLs
// =================================

#[test]
fn test_url_with_special_characters() {
    // Test TRN with special characters that are valid in URLs
    let trn = Trn::parse("trn:user:user-123:tool:openapi:api-v2:v1.0").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://user/user-123/tool/openapi/api-v2/v1.0");
    
    // Roundtrip conversion
    let back_to_trn = url_to_trn(&url).unwrap();
    assert_eq!(trn, back_to_trn);
}

#[test]
fn test_url_with_underscores() {
    // Test TRN with underscores
    let trn = Trn::parse("trn:user:user_name:tool:python:script_v2:v2.0").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://user/user_name/tool/python/script_v2/v2.0");
    
    // Roundtrip conversion
    let back_to_trn = url_to_trn(&url).unwrap();
    assert_eq!(trn, back_to_trn);
}

// =================================
// Edge Cases and Error Handling
// =================================

#[test]
fn test_url_edge_cases() {
    // Test with very long component names
    let long_name = "a".repeat(100);
    let trn_str = format!("trn:user:{}:tool:openapi:{}:v1.0", long_name, long_name);
    let trn = Trn::parse(&trn_str).unwrap();
    let url = trn.to_url().unwrap();
    let back_to_trn = url_to_trn(&url).unwrap();
    assert_eq!(trn, back_to_trn);
}

#[test]
fn test_url_case_sensitivity() {
    // TRN components should be case-sensitive in URLs
    let trn1 = Trn::parse("trn:user:Alice:tool:openapi:github-api:v1.0").unwrap();
    let trn2 = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    
    let url1 = trn1.to_url().unwrap();
    let url2 = trn2.to_url().unwrap();
    
    assert_ne!(url1, url2); // URLs should be different
    assert!(url1.contains("Alice"));
    assert!(url2.contains("alice"));
}

// =================================
// Different Platform Types
// =================================

#[test]
fn test_url_conversion_all_platforms() {
    let test_cases = vec![
        ("trn:aiplatform:model:bert:base-model:v1.0", "trn://aiplatform/model/bert/base-model/v1.0"),
        ("trn:user:alice:tool:openapi:github-api:v1.0", "trn://user/alice/tool/openapi/github-api/v1.0"),
        ("trn:org:company:dataset:csv:sales-data:v2.0", "trn://org/company/dataset/csv/sales-data/v2.0"),
    ];
    
    for (trn_str, expected_url) in test_cases {
        let trn = Trn::parse(trn_str).unwrap();
        let url = trn.to_url().unwrap();
        assert_eq!(url, expected_url);
        
        // Test reverse conversion
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, back_to_trn);
    }
}

// =================================
// Version Format Tests
// =================================

#[test]
fn test_url_with_different_versions() {
    let version_test_cases = vec![
        "v1.0",
        "v2.0.0",
        "latest",
        "stable",
        "1.0",
        "v1.0-beta",
        "v2.1.0-alpha.1",
    ];
    
    for version in version_test_cases {
        let trn_str = format!("trn:aiplatform:model:bert:base-model:{}", version);
        let trn = Trn::parse(&trn_str).unwrap();
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        
        assert_eq!(trn, back_to_trn);
        assert_eq!(back_to_trn.version(), version);
    }
}

// =================================
// Performance Tests
// =================================

#[test]
fn test_url_conversion_performance() {
    // Test conversion performance with many TRNs
    let trns: Vec<String> = (0..1000)
        .map(|i| format!("trn:user:user{}:tool:openapi:api{}:v1.0", i, i))
        .collect();
    
    for trn_str in &trns {
        let trn = Trn::parse(trn_str).unwrap();
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, back_to_trn);
    }
}

// =================================
// Method Tests on Trn Objects
// =================================

#[test]
fn test_trn_to_url_method() {
    // Test the to_url method on Trn objects
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://user/alice/tool/openapi/github-api/v1.0");
    
    // Test that the method gives same result as function
    let url_from_function = trn.to_url().unwrap();
    assert_eq!(url, url_from_function);
}

// =================================
// Error Message Tests
// =================================

#[test]
fn test_url_conversion_error_messages() {
    // Test that error messages are informative
    let result = url_to_trn("http://example.com");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("scheme") || error_msg.contains("protocol") || error_msg.contains("invalid"));
    
    // Test invalid TRN URL structure
    let result2 = url_to_trn("trn://incomplete");
    assert!(result2.is_err());
    let error_msg2 = result2.unwrap_err().to_string();
    assert!(error_msg2.contains("component") || error_msg2.contains("path") || error_msg2.contains("invalid"));
}

// =================================
// Integration with Builder
// =================================

#[test]
fn test_url_conversion_with_builder() {
    // Test URL conversion with TRNs created using builder
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    let url = trn.to_url().unwrap();
    assert_eq!(url, "trn://user/alice/tool/openapi/github-api/v1.0");
    
    let back_to_trn = url_to_trn(&url).unwrap();
    assert_eq!(trn, back_to_trn);
}

// =================================
// Consistency Tests
// =================================

#[test]
fn test_url_conversion_consistency() {
    let trn_str = "trn:user:alice:tool:openapi:github-api:v1.0";
    
    // Multiple conversions should give consistent results
    for _ in 0..10 {
        let trn = Trn::parse(trn_str).unwrap();
        let url = trn.to_url().unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        
        assert_eq!(trn, back_to_trn);
        assert_eq!(url, "trn://user/alice/tool/openapi/github-api/v1.0");
    }
}

// =================================
// URL Format Validation
// =================================

#[test]
fn test_url_format_structure() {
    let trn = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    // Check URL structure
    assert!(url.starts_with("trn://"));
    assert!(url.contains("user"));
    assert!(url.contains("alice"));
    assert!(url.contains("tool"));
    assert!(url.contains("openapi"));
    assert!(url.contains("github-api"));
    assert!(url.contains("v1.0"));
    
    // Check component order
    let path = url.strip_prefix("trn://").unwrap();
    let components: Vec<&str> = path.split('/').collect();
    assert_eq!(components[0], "user");
    assert_eq!(components[1], "alice");
    assert_eq!(components[2], "tool");
    assert_eq!(components[3], "openapi");
    assert_eq!(components[4], "github-api");
    assert_eq!(components[5], "v1.0");
} 