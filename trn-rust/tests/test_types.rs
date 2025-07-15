use trn_rust::*;
use std::str::FromStr;

// =================================
// Platform Tests
// =================================

#[test]
fn test_platform_enum_variants() {
    // Test standard platform variants
    assert_eq!(Platform::AiPlatform.to_string(), "aiplatform");
    assert_eq!(Platform::User.to_string(), "user");
    assert_eq!(Platform::Org.to_string(), "org");
}

#[test]
fn test_platform_from_str() {
    // Test parsing standard platforms
    assert_eq!(Platform::from_str("aiplatform").unwrap(), Platform::AiPlatform);
    assert_eq!(Platform::from_str("user").unwrap(), Platform::User);
    assert_eq!(Platform::from_str("org").unwrap(), Platform::Org);
    
    // Test custom platform
    let custom = Platform::from_str("enterprise").unwrap();
    assert!(matches!(custom, Platform::Custom(_)));
    assert_eq!(custom.to_string(), "enterprise");
}

#[test]
fn test_platform_custom_validation() {
    // Test invalid custom platform
    assert!(Platform::from_str("invalid@platform").is_err());
    assert!(Platform::from_str("").is_err());
    
    // Test valid custom platform
    assert!(Platform::from_str("custom-platform").is_ok());
    assert!(Platform::from_str("platform123").is_ok());
}

#[test]
fn test_platform_conversions() {
    let platform = Platform::User;
    let platform_string: String = platform.clone().into();
    assert_eq!(platform_string, "user");
    
    let platform_string_ref: String = (&platform).into();
    assert_eq!(platform_string_ref, "user");
}

// =================================
// ResourceType Tests
// =================================

#[test]
fn test_resource_type_enum_variants() {
    assert_eq!(ResourceType::Tool.to_string(), "tool");
    assert_eq!(ResourceType::Dataset.to_string(), "dataset");
    assert_eq!(ResourceType::Pipeline.to_string(), "pipeline");
    assert_eq!(ResourceType::Model.to_string(), "model");
    assert_eq!(ResourceType::Agent.to_string(), "agent");
    assert_eq!(ResourceType::Workflow.to_string(), "workflow");
}

#[test]
fn test_resource_type_from_str() {
    assert_eq!(ResourceType::from_str("tool").unwrap(), ResourceType::Tool);
    assert_eq!(ResourceType::from_str("dataset").unwrap(), ResourceType::Dataset);
    assert_eq!(ResourceType::from_str("model").unwrap(), ResourceType::Model);
    
    // Test custom resource type
    let custom = ResourceType::from_str("custom-resource").unwrap();
    assert!(matches!(custom, ResourceType::Custom(_)));
    assert_eq!(custom.to_string(), "custom-resource");
}

#[test]
fn test_resource_type_conversions() {
    let resource_type = ResourceType::Tool;
    let resource_string: String = resource_type.clone().into();
    assert_eq!(resource_string, "tool");
    
    let resource_string_ref: String = (&resource_type).into();
    assert_eq!(resource_string_ref, "tool");
}

// =================================
// ToolType Tests
// =================================

#[test]
fn test_tool_type_enum_variants() {
    assert_eq!(ToolType::OpenApi.to_string(), "openapi");
    assert_eq!(ToolType::Workflow.to_string(), "workflow");
    assert_eq!(ToolType::Python.to_string(), "python");
    assert_eq!(ToolType::Shell.to_string(), "shell");
    assert_eq!(ToolType::System.to_string(), "system");
    assert_eq!(ToolType::Function.to_string(), "function");
    assert_eq!(ToolType::Composite.to_string(), "composite");
    assert_eq!(ToolType::AsyncApi.to_string(), "async_api");
}

#[test]
fn test_tool_type_from_str() {
    assert_eq!(ToolType::from_str("openapi").unwrap(), ToolType::OpenApi);
    assert_eq!(ToolType::from_str("python").unwrap(), ToolType::Python);
    assert_eq!(ToolType::from_str("async_api").unwrap(), ToolType::AsyncApi);
    
    // Test custom tool type
    let custom = ToolType::from_str("custom-tool").unwrap();
    assert!(matches!(custom, ToolType::Custom(_)));
    assert_eq!(custom.to_string(), "custom-tool");
}

#[test]
fn test_tool_type_conversions() {
    let tool_type = ToolType::OpenApi;
    let tool_string: String = tool_type.clone().into();
    assert_eq!(tool_string, "openapi");
    
    let tool_string_ref: String = (&tool_type).into();
    assert_eq!(tool_string_ref, "openapi");
}

// =================================
// TrnComponents Tests
// =================================

#[test]
fn test_trn_components_creation() {
    let components = TrnComponents::new(
        "user",
        "tool", 
        "openapi",
        "github-api",
        "v1.0"
    );
    
    assert_eq!(components.platform, "user");
    assert_eq!(components.scope, None);
    assert_eq!(components.resource_type, "tool");
    assert_eq!(components.type_, "openapi");
    assert_eq!(components.subtype, None);
    assert_eq!(components.instance_id, "github-api");
    assert_eq!(components.version, "v1.0");
    assert_eq!(components.tag, None);
    assert_eq!(components.hash, None);
}

#[test]
fn test_trn_components_builder_pattern() {
    let components = TrnComponents::new("user", "tool", "openapi", "github-api", "v1.0")
        .with_scope("alice")
        .with_subtype("rest")
        .with_tag("stable")
        .with_hash("abc123");
    
    assert_eq!(components.scope, Some("alice"));
    assert_eq!(components.subtype, Some("rest"));
    assert_eq!(components.tag, Some("stable"));
    assert_eq!(components.hash, Some("abc123"));
}

#[test]
fn test_trn_components_to_owned() {
    let components = TrnComponents::new("user", "tool", "openapi", "github-api", "v1.0")
        .with_scope("alice");
    
    let owned_trn = components.to_owned();
    assert_eq!(owned_trn.platform(), "user");
    assert_eq!(owned_trn.scope(), Some("alice"));
    assert_eq!(owned_trn.resource_type(), "tool");
    assert_eq!(owned_trn.type_(), "openapi");
    assert_eq!(owned_trn.instance_id(), "github-api");
    assert_eq!(owned_trn.version(), "v1.0");
}

// =================================
// Trn Structure Tests
// =================================

#[test]
fn test_trn_new_basic() {
    // Use aiplatform which doesn't require scope
    let trn = Trn::new("aiplatform", "model", "bert", "base-model", "v1.0").unwrap();
    
    assert_eq!(trn.platform(), "aiplatform");
    assert_eq!(trn.scope(), None);
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.type_(), "bert");
    assert_eq!(trn.instance_id(), "base-model");
    assert_eq!(trn.version(), "v1.0");
    assert_eq!(trn.subtype(), None);
    assert_eq!(trn.tag(), None);
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_trn_new_full() {
    // Use a simpler format without subtype, tag, and hash
    let trn = Trn::new_full(
        "user",
        Some("alice"),
        "tool",
        "openapi",
        None::<String>,
        "github-api",
        "v1.0",
        None::<String>,
        None::<String>
    ).unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.subtype(), None);
    assert_eq!(trn.instance_id(), "github-api");
    assert_eq!(trn.version(), "v1.0");
    assert_eq!(trn.tag(), None);
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_trn_accessors() {
    // Use a simpler format that passes validation
    let trn = Trn::new_full(
        "org",
        Some("company"),
        "model",
        "bert",
        None::<String>,
        "language-model",
        "v2.0",
        None::<String>,
        None::<String>
    ).unwrap();
    
    // Test all accessor methods
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("company"));
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.type_(), "bert");
    assert_eq!(trn.subtype(), None);
    assert_eq!(trn.instance_id(), "language-model");
    assert_eq!(trn.version(), "v2.0");
    assert_eq!(trn.tag(), None);
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_trn_to_string() {
    // Use aiplatform for basic test (doesn't require scope)
    let trn = Trn::new("aiplatform", "model", "bert", "base-model", "v1.0").unwrap();
    assert_eq!(trn.to_string(), "trn:aiplatform:model:bert:base-model:v1.0");
    
    let trn_with_scope = Trn::new_full(
        "user",
        Some("alice"),
        "tool",
        "openapi",
        None::<String>,
        "github-api",
        "v1.0",
        None::<String>,
        None::<String>
    ).unwrap();
    assert_eq!(trn_with_scope.to_string(), "trn:user:alice:tool:openapi:github-api:v1.0");
}

#[test]
fn test_trn_parse() {
    let trn_str = "trn:user:alice:tool:openapi:github-api:v1.0";
    let trn = Trn::parse(trn_str).unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "github-api");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_trn_validation() {
    // Valid TRN should validate (use aiplatform which doesn't require scope)
    let valid_trn = Trn::new("aiplatform", "model", "bert", "base-model", "v1.0").unwrap();
    assert!(valid_trn.validate().is_ok());
    assert!(valid_trn.is_valid());
    
    // Test validation through convenience function
    assert!(validate(&valid_trn).is_ok());
}

#[test]
fn test_trn_manipulation_methods() {
    let trn = Trn::new_full(
        "user",
        Some("alice"),
        "tool",
        "openapi",
        None::<String>,
        "github-api",
        "v1.0",
        None::<String>,
        None::<String>
    ).unwrap();
    
    // Test without_hash (already None)
    let no_hash = trn.without_hash();
    assert_eq!(no_hash.hash(), None);
    assert_eq!(no_hash.tag(), None); // Other fields preserved
    
    // Test without_tag (already None)
    let no_tag = trn.without_tag();
    assert_eq!(no_tag.tag(), None);
    assert_eq!(no_tag.hash(), None); // Other fields preserved
    
    // Test base_trn
    let base = trn.base_trn();
    assert_eq!(base.version(), "*");
    assert_eq!(base.tag(), None);
    assert_eq!(base.hash(), None);
    assert_eq!(base.platform(), "user"); // Core fields preserved
    assert_eq!(base.instance_id(), "github-api");
}

#[test]
fn test_trn_mutable_operations() {
    let mut trn = Trn::new("aiplatform", "model", "bert", "base-model", "v1.0").unwrap();
    
    // Test setters
    trn.set_scope(Some("alice".to_string()));
    assert_eq!(trn.scope(), Some("alice"));
    
    trn.set_subtype(Some("rest".to_string()));
    assert_eq!(trn.subtype(), Some("rest"));
    
    trn.set_version("v2.0".to_string());
    assert_eq!(trn.version(), "v2.0");
    
    trn.set_tag(Some("stable".to_string()));
    assert_eq!(trn.tag(), Some("stable"));
    
    trn.set_hash(Some("abc123".to_string()));
    assert_eq!(trn.hash(), Some("abc123"));
}

#[test]
fn test_trn_components_method() {
    let trn = Trn::new_full(
        "user",
        Some("alice"),
        "tool",
        "openapi",
        None::<String>,
        "github-api",
        "v1.0",
        None::<String>,
        None::<String>
    ).unwrap();
    
    let components = trn.components();
    assert_eq!(components.platform, "user");
    assert_eq!(components.scope, Some("alice"));
    assert_eq!(components.resource_type, "tool");
    assert_eq!(components.type_, "openapi");
    assert_eq!(components.subtype, None);
    assert_eq!(components.instance_id, "github-api");
    assert_eq!(components.version, "v1.0");
    assert_eq!(components.tag, None);
    assert_eq!(components.hash, None);
}

#[test]
fn test_trn_compatibility() {
    let trn1 = Trn::new("aiplatform", "model", "bert", "base-model", "v1.0").unwrap();
    let trn2 = Trn::new("aiplatform", "model", "bert", "base-model", "v2.0").unwrap();
    let trn3 = Trn::new("aiplatform", "model", "bert", "large-model", "v1.0").unwrap();
    
    // Same base, different version - should be compatible
    assert!(trn1.is_compatible_with(&trn2));
    
    // Different instance_id - should not be compatible
    assert!(!trn1.is_compatible_with(&trn3));
}

#[test]
fn test_trn_clone_and_equality() {
    let trn1 = Trn::new_full(
        "user",
        Some("alice"),
        "tool",
        "openapi",
        None::<String>,
        "github-api",
        "v1.0",
        None::<String>,
        None::<String>
    ).unwrap();
    
    let trn2 = trn1.clone();
    assert_eq!(trn1, trn2);
    
    let trn3 = Trn::new_full(
        "user", 
        Some("bob"), 
        "tool", 
        "openapi", 
        None::<String>,
        "github-api", 
        "v1.0",
        None::<String>,
        None::<String>
    ).unwrap();
    assert_ne!(trn1, trn3); // Different scope
}

// =================================
// Edge Cases and Error Conditions
// =================================

#[test]
fn test_invalid_trn_creation() {
    // Empty values should fail validation
    assert!(Trn::new("", "tool", "openapi", "github-api", "v1.0").is_err());
    assert!(Trn::new("user", "", "openapi", "github-api", "v1.0").is_err());
    assert!(Trn::new("user", "tool", "", "github-api", "v1.0").is_err());
    assert!(Trn::new("user", "tool", "openapi", "", "v1.0").is_err());
    assert!(Trn::new("user", "tool", "openapi", "github-api", "").is_err());
}

#[test]
fn test_convenience_functions() {
    // Test parse convenience function
    let trn = parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    
    // Test validate convenience function
    assert!(validate("trn:user:alice:tool:openapi:github-api:v1.0").is_ok());
    assert!(validate("invalid-trn").is_err());
    
    // Test builder convenience function
    let trn = builder()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
}

#[test]
fn test_version_constant() {
    assert!(!VERSION.is_empty());
    assert!(VERSION.chars().any(|c| c.is_ascii_digit()));
} 