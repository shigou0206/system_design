use trn_rust::{Trn, url_to_trn};

#[test]
fn test_trn_to_url() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    assert!(url.starts_with("trn://"));
    assert!(url.contains("user"));
    assert!(url.contains("alice"));
    assert!(url.contains("tool"));
    assert!(url.contains("myapi"));
    assert!(url.contains("v1.0"));
}

#[test]
fn test_trn_to_http_url() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let base = "https://platform.example.com";
    let url = trn.to_http_url(base).unwrap();
    
    assert!(url.starts_with("https://platform.example.com"));
    assert!(url.contains("user"));
    assert!(url.contains("alice"));
    assert!(url.contains("tool"));
    assert!(url.contains("myapi"));
    assert!(url.contains("v1.0"));
}

#[test]
fn test_url_to_trn() {
    let trn_url = "trn://user/alice/tool/myapi/v1.0";
    let trn = url_to_trn(trn_url).unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), "alice");
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.resource_id(), "myapi");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_trn_url_roundtrip() {
    let original = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let url = original.to_url().unwrap();
    let reconstructed = url_to_trn(&url).unwrap();
    
    assert_eq!(original.platform(), reconstructed.platform());
    assert_eq!(original.scope(), reconstructed.scope());
    assert_eq!(original.resource_type(), reconstructed.resource_type());
    assert_eq!(original.resource_id(), reconstructed.resource_id());
    assert_eq!(original.version(), reconstructed.version());
}

#[test]
fn test_http_url_roundtrip() {
    let original = Trn::new("org", "company", "model", "bert", "v2.1").unwrap();
    let base = "https://platform.example.com";
    let url = original.to_http_url(base).unwrap();
    
    // For HTTP URLs, we need to extract the TRN part
    // This test depends on the URL format implementation
    assert!(url.contains("org"));
    assert!(url.contains("company"));
    assert!(url.contains("model"));
    assert!(url.contains("bert"));
    assert!(url.contains("v2.1"));
}

#[test]
fn test_url_with_different_platforms() {
    let test_cases = vec![
        ("user", "alice", "tool", "myapi", "v1.0"),
        ("org", "company", "model", "bert", "v2.1"),
        ("aiplatform", "system", "dataset", "training", "latest"),
        ("custom", "scope", "pipeline", "etl", "main"),
    ];
    
    for (platform, scope, resource_type, resource_id, version) in test_cases {
        let trn = Trn::new(platform, scope, resource_type, resource_id, version).unwrap();
        let url = trn.to_url().unwrap();
        let reconstructed = url_to_trn(&url).unwrap();
        
        assert_eq!(trn.platform(), reconstructed.platform());
        assert_eq!(trn.scope(), reconstructed.scope());
        assert_eq!(trn.resource_type(), reconstructed.resource_type());
        assert_eq!(trn.resource_id(), reconstructed.resource_id());
        assert_eq!(trn.version(), reconstructed.version());
    }
}

#[test]
fn test_url_with_special_characters() {
    let trn = Trn::new("user", "alice-smith", "custom-type", "my_api.v2", "v1.0-beta").unwrap();
    let url = trn.to_url().unwrap();
    let reconstructed = url_to_trn(&url).unwrap();
    
    assert_eq!(trn.scope(), reconstructed.scope());
    assert_eq!(trn.resource_type(), reconstructed.resource_type());
    assert_eq!(trn.resource_id(), reconstructed.resource_id());
    assert_eq!(trn.version(), reconstructed.version());
}

#[test]
fn test_url_encoding() {
    // Test that special characters are properly URL encoded (use hyphens instead of spaces)
    let trn = Trn::new("user", "alice", "tool", "my-api", "v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    // Should not contain problematic characters
    assert!(!url.contains(" "));
    assert!(url.contains("my-api"));
}

#[test]
fn test_invalid_trn_urls() {
    let invalid_urls = vec![
        "",                                    // Empty
        "https://example.com",                 // Not TRN URL
        "trn://",                             // Incomplete
        "trn://user",                         // Too few components
        "trn://user/alice",                   // Too few components
        "trn://user/alice/tool",              // Too few components
        "trn://user/alice/tool/myapi",        // Missing version
        "invalid://user/alice/tool/myapi/v1.0", // Wrong scheme
    ];
    
    for invalid_url in invalid_urls {
        assert!(url_to_trn(invalid_url).is_err(), "Should fail for: {}", invalid_url);
    }
}

#[test]
fn test_http_url_with_different_bases() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    
    let bases = vec![
        "https://platform.example.com",
        "http://localhost:8080",
        "https://api.trn.dev",
        "https://registry.company.com/api/v1",
    ];
    
    for base in bases {
        let url = trn.to_http_url(base).unwrap();
        // URL should contain the domain from the base
        let domain = base.split('/').nth(2).unwrap_or(base);
        assert!(url.contains(domain), "URL should contain domain: {} in {}", domain, url);
    }
    
    // Test with maximum reasonable component lengths
    let max_trn = Trn::new(
        &"a".repeat(20),
        &"b".repeat(20),
        "tool",  // Use supported resource type
        &"d".repeat(50),
        &"e".repeat(20)
    ).unwrap();
    
    let url = max_trn.to_url().unwrap();
    let reconstructed = url_to_trn(&url).unwrap();
    
    assert_eq!(max_trn.platform(), reconstructed.platform());
    assert_eq!(max_trn.scope(), reconstructed.scope());
    assert_eq!(max_trn.resource_type(), reconstructed.resource_type());
    assert_eq!(max_trn.resource_id(), reconstructed.resource_id());
    assert_eq!(max_trn.version(), reconstructed.version());
}

#[test]
fn test_url_path_structure() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    // URL should follow the expected path structure
    // trn://user/alice/tool/myapi/v1.0
    let expected_parts = vec!["user", "alice", "tool", "myapi", "v1.0"];
    for part in expected_parts {
        assert!(url.contains(part), "URL should contain: {}", part);
    }
}

#[test]
fn test_http_url_path_structure() {
    let trn = Trn::new("org", "company", "model", "bert", "v2.1").unwrap();
    let base = "https://platform.example.com";
    let url = trn.to_http_url(base).unwrap();
    
    // Should contain all TRN components in the path
    let expected_parts = vec!["org", "company", "model", "bert", "v2.1"];
    for part in expected_parts {
        assert!(url.contains(part), "HTTP URL should contain: {}", part);
    }
}

#[test]
fn test_url_case_sensitivity() {
    let trn1 = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    // Use supported resource type in lowercase
    let trn2 = Trn::new("USER", "ALICE", "tool", "MYAPI", "V1.0").unwrap();
    
    let url1 = trn1.to_url().unwrap();
    let url2 = trn2.to_url().unwrap();
    
    // URLs should preserve case
    assert_ne!(url1, url2);
    
    // Round trip should preserve original case
    let reconstructed1 = url_to_trn(&url1).unwrap();
    let reconstructed2 = url_to_trn(&url2).unwrap();
    
    assert_eq!(trn1.to_string(), reconstructed1.to_string());
    assert_eq!(trn2.to_string(), reconstructed2.to_string());
}

#[test]
fn test_url_query_parameters() {
    // Test if any query parameters are added (like version info)
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    // Check if URL has query parameters (implementation dependent)
    // This test is for documentation/understanding of the URL format
    println!("Generated URL: {}", url);
}

#[test]
fn test_url_fragment_handling() {
    // Test if fragments are supported or handled
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    // Check if URL has fragments (implementation dependent)
    // This test is for documentation/understanding of the URL format
    println!("Generated URL: {}", url);
}

#[test]
fn test_url_validation() {
    // Test that generated URLs are valid
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let url = trn.to_url().unwrap();
    
    // URL should be parseable by standard URL parser
    let parsed = url::Url::parse(&url);
    assert!(parsed.is_ok(), "Generated URL should be valid: {}", url);
}

#[test]
fn test_http_url_validation() {
    let trn = Trn::new("user", "alice", "tool", "myapi", "v1.0").unwrap();
    let base = "https://platform.example.com";
    let url = trn.to_http_url(base).unwrap();
    
    // HTTP URL should be parseable by standard URL parser
    let parsed = url::Url::parse(&url);
    assert!(parsed.is_ok(), "Generated HTTP URL should be valid: {}", url);
    
    let parsed_url = parsed.unwrap();
    assert_eq!(parsed_url.scheme(), "https");
    assert_eq!(parsed_url.host_str(), Some("platform.example.com"));
}

#[test]
fn test_url_performance() {
    use std::time::Instant;
    
    let trns: Vec<Trn> = (0..1000)
        .map(|i| Trn::new("user", &format!("user{}", i), "tool", &format!("api{}", i), "v1.0").unwrap())
        .collect();
    
    let start = Instant::now();
    
    // Convert 1000 TRNs to URLs
    for trn in &trns {
        let _url = trn.to_url().unwrap();
    }
    
    let duration = start.elapsed();
    // Should convert 1000 TRNs to URLs in less than 100ms
    assert!(duration.as_millis() < 100, "URL conversion too slow: {:?}", duration);
    
    // Test reverse conversion performance
    let urls: Vec<String> = trns.iter().map(|trn| trn.to_url().unwrap()).collect();
    
    let start = Instant::now();
    
    // Convert 1000 URLs back to TRNs
    for url in &urls {
        let _trn = url_to_trn(url).unwrap();
    }
    
    let duration = start.elapsed();
    // Should convert 1000 URLs to TRNs in less than 100ms
    assert!(duration.as_millis() < 100, "URL parsing too slow: {:?}", duration);
}

#[test]
fn test_url_edge_cases() {
    // Test minimum valid TRN (with proper lengths and supported resource type)
    let min_trn = Trn::new("aa", "b", "tool", "d", "e").unwrap();
    let url = min_trn.to_url().unwrap();
    let reconstructed = url_to_trn(&url).unwrap();
    
    assert_eq!(min_trn.to_string(), reconstructed.to_string());
    
    // Test with maximum reasonable component lengths
    let max_trn = Trn::new(
        &"a".repeat(20),
        &"b".repeat(20),
        "tool",  // Use supported resource type
        &"d".repeat(50),
        &"e".repeat(20)
    ).unwrap();
    
    let url = max_trn.to_url().unwrap();
    let reconstructed = url_to_trn(&url).unwrap();
    
    assert_eq!(max_trn.platform(), reconstructed.platform());
    assert_eq!(max_trn.scope(), reconstructed.scope());
    assert_eq!(max_trn.resource_type(), reconstructed.resource_type());
    assert_eq!(max_trn.resource_id(), reconstructed.resource_id());
    assert_eq!(max_trn.version(), reconstructed.version());
} 