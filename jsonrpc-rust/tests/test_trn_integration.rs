//! TRN Integration Tests
//! 
//! This module tests the integration between jsonrpc-rust and the TRN-rust library,
//! ensuring proper functionality, error handling, and compatibility.

#![cfg(feature = "trn-integration")]

use jsonrpc_rust::core::types::{TrnContext, ServiceContext, AuthContext};
use jsonrpc_rust::core::error::Error;
use serde_json::json;

#[test]
fn test_trn_context_basic_functionality() {
    let trn = TrnContext::new("user", "alice", "tool", "weather-api", "v1.0")
        .with_tenant_id("acme-corp")
        .with_namespace("production");
    
    // Test basic properties
    assert_eq!(trn.platform, "user");
    assert_eq!(trn.scope, "alice");
    assert_eq!(trn.resource_type, "tool");
    assert_eq!(trn.resource_id, "weather-api");
    assert_eq!(trn.version, "v1.0");
    assert_eq!(trn.tenant_id, "acme-corp");
    assert_eq!(trn.namespace, "production");
    
    // Test TRN string conversion
    let trn_string = trn.to_trn_string();
    assert_eq!(trn_string, "trn:user:alice:tool:weather-api:v1.0");
}

#[test]
fn test_trn_context_metadata_handling() {
    let mut trn = TrnContext::new("org", "openai", "model", "gpt-4", "v1.0");
    
    // Add metadata
    trn.metadata.insert("cost_per_token".to_string(), json!(0.00003));
    trn.metadata.insert("max_tokens".to_string(), json!(8192));
    trn.metadata.insert("capabilities".to_string(), json!(["text", "code", "reasoning"]));
    
    assert_eq!(trn.metadata.len(), 3);
    assert_eq!(trn.metadata.get("cost_per_token").unwrap(), &json!(0.00003));
    assert_eq!(trn.metadata.get("max_tokens").unwrap(), &json!(8192));
}

#[test]
fn test_trn_string_parsing_valid() {
    let valid_trns = vec![
        "trn:user:alice:tool:weather:v1.0",
        "trn:org:openai:model:gpt-4:latest",
        "trn:aiplatform:huggingface:dataset:common-crawl:v2.1",
        "trn:enterprise:acme:pipeline:data-processing:v3.0.1",
    ];
    
    for trn_str in valid_trns {
        let parsed = TrnContext::from_trn_string(trn_str);
        assert!(parsed.is_ok(), "Failed to parse valid TRN: {}", trn_str);
        
        let trn = parsed.unwrap();
        assert_eq!(trn.to_trn_string(), trn_str);
    }
}

#[test]
fn test_trn_string_parsing_invalid() {
            let invalid_trns = vec![
            "",                                    // Empty string
            "not-a-trn",                          // Not TRN format
            "trn:",                               // Incomplete
            "trn:only:three:parts",               // Too few parts
            "trn:too:many:parts:here:now:extra",  // Too many parts
            "wrong:user:alice:tool:weather:v1.0", // Wrong prefix
        ];
    
    for invalid_trn in invalid_trns {
        let result = TrnContext::from_trn_string(invalid_trn);
        assert!(result.is_err(), "Should fail to parse invalid TRN: {}", invalid_trn);
        
        if let Err(Error::Custom { message, .. }) = result {
            assert!(message.contains("Invalid TRN format"));
        } else {
            panic!("Expected Custom error for invalid TRN: {}", invalid_trn);
        }
    }
}

#[test]
fn test_service_context_trn_integration() {
    let trn_context = TrnContext::new("user", "bob", "tool", "calculator", "v2.0")
        .with_tenant_id("math-team")
        .with_namespace("staging");
    
    let auth_context = AuthContext::new("bob", "api_key")
        .with_permission("calculator:use")
        .with_role("developer");
    
    let context = ServiceContext::new("request-456")
        .with_trn_context(trn_context.clone())
        .with_auth_context(auth_context)
        .with_metadata("environment".to_string(), json!("staging"))
        .with_metadata("region".to_string(), json!("us-west-2"));
    
    // Verify TRN context integration
    assert!(context.trn_context.is_some());
    let trn = context.trn_context.as_ref().unwrap();
    assert_eq!(trn.resource_id, "calculator");
    assert_eq!(trn.tenant_id, "math-team");
    assert_eq!(trn.namespace, "staging");
    
    // Verify other context data
    assert!(context.auth_context.is_some());
    assert_eq!(context.request_id, "request-456");
    assert_eq!(context.metadata.len(), 2);
}

#[test]
fn test_trn_context_serialization_roundtrip() {
    let original = TrnContext::new("aiplatform", "kaggle", "dataset", "titanic", "v1.0")
        .with_tenant_id("data-science")
        .with_namespace("experiment-123");
    
    // Test JSON serialization roundtrip
    let json_str = serde_json::to_string(&original).unwrap();
    let from_json: TrnContext = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(original.platform, from_json.platform);
    assert_eq!(original.scope, from_json.scope);
    assert_eq!(original.resource_type, from_json.resource_type);
    assert_eq!(original.resource_id, from_json.resource_id);
    assert_eq!(original.version, from_json.version);
    assert_eq!(original.tenant_id, from_json.tenant_id);
    assert_eq!(original.namespace, from_json.namespace);
    assert_eq!(original.to_trn_string(), from_json.to_trn_string());
}

#[test]
fn test_trn_context_clone_and_equality() {
    let trn1 = TrnContext::new("user", "alice", "workflow", "image-gen", "v1.5")
        .with_tenant_id("creative-team")
        .with_namespace("production");
    
    let trn2 = trn1.clone();
    
    // Test clone preserves all fields
    assert_eq!(trn1.platform, trn2.platform);
    assert_eq!(trn1.scope, trn2.scope);
    assert_eq!(trn1.resource_type, trn2.resource_type);
    assert_eq!(trn1.resource_id, trn2.resource_id);
    assert_eq!(trn1.version, trn2.version);
    assert_eq!(trn1.tenant_id, trn2.tenant_id);
    assert_eq!(trn1.namespace, trn2.namespace);
    assert_eq!(trn1.to_trn_string(), trn2.to_trn_string());
}

#[test]
fn test_trn_context_edge_cases() {
    // Test with special characters (that are valid in TRN)
    let trn = TrnContext::new("user", "alice-dev", "tool", "weather_api", "v1.0.0");
    let trn_string = trn.to_trn_string();
    assert_eq!(trn_string, "trn:user:alice-dev:tool:weather_api:v1.0.0");
    
    // Test roundtrip with special characters
    let parsed = TrnContext::from_trn_string(&trn_string).unwrap();
    assert_eq!(parsed.scope, "alice-dev");
    assert_eq!(parsed.resource_id, "weather_api");
    assert_eq!(parsed.version, "v1.0.0");
}

#[test]
fn test_multiple_trn_contexts_in_batch() {
    let contexts = vec![
        TrnContext::new("user", "alice", "tool", "weather", "v1.0"),
        TrnContext::new("org", "openai", "model", "gpt-3", "v1.0"),
        TrnContext::new("aiplatform", "huggingface", "dataset", "wikitext", "latest"),
        TrnContext::new("enterprise", "acme", "pipeline", "etl", "v2.1"),
    ];
    
    // Test batch TRN string generation
    let trn_strings: Vec<String> = contexts.iter()
        .map(|c| c.to_trn_string())
        .collect();
    
    assert_eq!(trn_strings.len(), 4);
    assert_eq!(trn_strings[0], "trn:user:alice:tool:weather:v1.0");
    assert_eq!(trn_strings[1], "trn:org:openai:model:gpt-3:v1.0");
    assert_eq!(trn_strings[2], "trn:aiplatform:huggingface:dataset:wikitext:latest");
    assert_eq!(trn_strings[3], "trn:enterprise:acme:pipeline:etl:v2.1");
    
    // Test batch parsing back
    let parsed_contexts: Result<Vec<TrnContext>, _> = trn_strings.iter()
        .map(|s| TrnContext::from_trn_string(s))
        .collect();
    
    assert!(parsed_contexts.is_ok());
    let parsed = parsed_contexts.unwrap();
    assert_eq!(parsed.len(), 4);
    
    // Verify each parsed context matches original
    for (original, parsed) in contexts.iter().zip(parsed.iter()) {
        assert_eq!(original.to_trn_string(), parsed.to_trn_string());
    }
}

#[test]
fn test_trn_context_with_complex_metadata() {
    let mut trn = TrnContext::new("org", "anthropic", "model", "claude-3", "v3.0");
    
    // Add complex metadata
    trn.metadata.insert("model_info".to_string(), json!({
        "parameters": "175B",
        "training_data": "Constitutional AI",
        "capabilities": ["reasoning", "coding", "analysis"],
        "safety_level": "high"
    }));
    
    trn.metadata.insert("pricing".to_string(), json!({
        "input_tokens": 0.000008,
        "output_tokens": 0.000024,
        "currency": "USD"
    }));
    
    // Test serialization with complex metadata
    let json_str = serde_json::to_string(&trn).unwrap();
    let deserialized: TrnContext = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(trn.metadata.len(), deserialized.metadata.len());
    assert_eq!(
        trn.metadata.get("model_info"), 
        deserialized.metadata.get("model_info")
    );
    assert_eq!(
        trn.metadata.get("pricing"), 
        deserialized.metadata.get("pricing")
    );
}

#[test]
fn test_trn_context_namespace_isolation() {
    let production_trn = TrnContext::new("user", "alice", "tool", "api", "v1.0")
        .with_namespace("production")
        .with_tenant_id("team-1");
    
    let staging_trn = TrnContext::new("user", "alice", "tool", "api", "v1.0")
        .with_namespace("staging")
        .with_tenant_id("team-1");
    
    let dev_trn = TrnContext::new("user", "alice", "tool", "api", "v1.0")
        .with_namespace("development")
        .with_tenant_id("team-1");
    
    // Same TRN string but different namespaces
    assert_eq!(production_trn.to_trn_string(), staging_trn.to_trn_string());
    assert_eq!(staging_trn.to_trn_string(), dev_trn.to_trn_string());
    
    // But different namespaces
    assert_ne!(production_trn.namespace, staging_trn.namespace);
    assert_ne!(staging_trn.namespace, dev_trn.namespace);
    
    // Same tenant
    assert_eq!(production_trn.tenant_id, staging_trn.tenant_id);
    assert_eq!(staging_trn.tenant_id, dev_trn.tenant_id);
}

#[test]
fn test_trn_error_integration() {
    // Test that TRN parsing errors are properly integrated with our error system
    let invalid_trn = "invalid:trn:format";
    
    match TrnContext::from_trn_string(invalid_trn) {
        Err(Error::Custom { message, .. }) => {
            assert!(message.contains("Invalid TRN format"));
            assert!(message.contains(invalid_trn));
        }
        _ => panic!("Expected Custom error for invalid TRN format"),
    }
} 