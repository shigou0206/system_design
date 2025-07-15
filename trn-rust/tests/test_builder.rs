use trn_rust::*;

// =================================
// Basic Builder Tests
// =================================

#[test]
fn test_builder_new() {
    let builder = TrnBuilder::new();
    assert!(!builder.is_complete());
    
    let missing = builder.missing_fields();
    assert!(missing.contains(&"platform"));
    assert!(missing.contains(&"resource_type"));
    assert!(missing.contains(&"type"));
    assert!(missing.contains(&"instance_id"));
    assert!(missing.contains(&"version"));
}

#[test]
fn test_builder_basic_flow() {
    let trn = TrnBuilder::new()
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
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "github-api");
    assert_eq!(trn.version(), "v1.0");
}

#[test]
fn test_builder_from_trn() {
    let original = Trn::parse("trn:user:alice:tool:openapi:github-api:v1.0").unwrap();
    let builder = TrnBuilder::from_trn(&original);
    
    assert!(builder.is_complete());
    
    let rebuilt = builder.build().unwrap();
    assert_eq!(original, rebuilt);
}

#[test]
fn test_builder_build_string() {
    let trn_string = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .build_string()
        .unwrap();
    
    assert_eq!(trn_string, "trn:aiplatform:model:bert:base-model:v1.0");
}

// =================================
// Platform and Enum Tests
// =================================

#[test]
fn test_builder_platform_enum() {
    let trn = TrnBuilder::new()
        .platform_enum(Platform::User)
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "user");
}

#[test]
fn test_builder_resource_type_enum() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type_enum(ResourceType::Model)
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "model");
}

#[test]
fn test_builder_tool_type() {
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .tool_type(ToolType::OpenApi)
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.type_(), "openapi");
}

// =================================
// Optional Fields Tests
// =================================

#[test]
fn test_builder_with_optional_fields() {
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .subtype("rest")
        .instance_id("github-api")
        .version("v1.0")
        .tag("stable")
        .hash("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
        .build()
        .unwrap();
    
    assert_eq!(trn.subtype(), Some("rest"));
    assert_eq!(trn.tag(), Some("stable"));
    assert_eq!(trn.hash(), Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
}

#[test]
fn test_builder_no_optional_fields() {
    let builder = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .subtype("rest")
        .instance_id("github-api")
        .version("v1.0")
        .tag("stable")
        .hash("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
        .no_subtype()
        .no_tag()
        .no_hash();
    
    let trn = builder.build().unwrap();
    assert_eq!(trn.subtype(), None);
    assert_eq!(trn.tag(), None);
    assert_eq!(trn.hash(), None);
}

#[test]
fn test_builder_no_scope() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .no_scope()
        .build()
        .unwrap();
    
    assert_eq!(trn.scope(), None);
}

// =================================
// Hash Tests
// =================================

#[test]
fn test_builder_sha256_hash() {
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .sha256_hash("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
        .build()
        .unwrap();
    
    assert_eq!(trn.hash(), Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
}

#[test]
fn test_builder_md5_hash() {
    let trn = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .md5_hash("12345678901234567890123456789012")
        .build()
        .unwrap();
    
    assert_eq!(trn.hash(), Some("md5:12345678901234567890123456789012"));
}

// =================================
// Validation Tests
// =================================

#[test]
fn test_builder_validate_on_build() {
    let builder = TrnBuilder::new()
        .platform("user")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .validate_on_build(true);
    
    // Should fail because user platform requires scope
    assert!(builder.build().is_err());
}

#[test]
fn test_builder_validate_on_build_false() {
    let builder = TrnBuilder::new()
        .platform("user")
        .scope("alice")  // Add required scope field
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .validate_on_build(false);
    
    // Should succeed with validation disabled
    assert!(builder.build().is_ok());
}

// =================================
// Completion and Missing Fields Tests
// =================================

#[test]
fn test_builder_is_complete() {
    let incomplete = TrnBuilder::new()
        .platform("user")
        .scope("alice");
    assert!(!incomplete.is_complete());
    
    let complete = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0");
    assert!(complete.is_complete());
}

#[test]
fn test_builder_missing_fields() {
    let builder = TrnBuilder::new()
        .platform("user")
        .resource_type("tool");
    
    let missing = builder.missing_fields();
    assert!(missing.contains(&"type"));
    assert!(missing.contains(&"instance_id"));
    assert!(missing.contains(&"version"));
    assert!(!missing.contains(&"platform"));
    assert!(!missing.contains(&"resource_type"));
}

// =================================
// Reset and Fork Tests
// =================================

#[test]
fn test_builder_reset() {
    let builder = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .reset();
    
    assert!(!builder.is_complete());
    let missing = builder.missing_fields();
    assert!(missing.contains(&"platform"));
}

#[test]
fn test_builder_fork() {
    let original = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool");
    
    let forked = original.fork()
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0");
    
    assert!(!original.is_complete());
    assert!(forked.is_complete());
}

// =================================
// Template Functions Tests
// =================================

#[test]
fn test_user_tool_template() {
    let trn = TrnBuilder::user_tool("alice")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
}

#[test]
fn test_org_tool_template() {
    let trn = TrnBuilder::org_tool("company")
        .type_("python")
        .instance_id("data-processor")
        .version("v2.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("company"));
    assert_eq!(trn.resource_type(), "tool");
}

#[test]
fn test_system_tool_template() {
    let trn = TrnBuilder::system_tool()
        .type_("shell")
        .instance_id("file-processor")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "aiplatform");
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.scope(), None);
}

// =================================
// Specific Tool Type Tests
// =================================

#[test]
fn test_openapi_tool() {
    let trn = TrnBuilder::user_tool("alice")
        .openapi_tool()
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.type_(), "openapi");
}

#[test]
fn test_python_tool() {
    let trn = TrnBuilder::user_tool("alice")
        .python_tool()
        .instance_id("data-processor")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.type_(), "python");
}

#[test]
fn test_workflow_tool() {
    let trn = TrnBuilder::workflow_tool()
        .platform("aiplatform")
        .instance_id("automation-flow")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.type_(), "workflow");
}

// =================================
// Resource Type Templates Tests
// =================================

#[test]
fn test_dataset_template() {
    let trn = TrnBuilder::new()
        .platform("org")
        .scope("company")
        .dataset()
        .type_("csv")
        .instance_id("sales-data")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "dataset");
}

#[test]
fn test_model_template() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .model()
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "model");
}

#[test]
fn test_pipeline_template() {
    let trn = TrnBuilder::pipeline()
        .platform("aiplatform")
        .type_("etl")
        .instance_id("data-processor")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "pipeline");
}

// =================================
// Version Helper Tests
// =================================

#[test]
fn test_latest_version() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .latest()
        .build()
        .unwrap();
    
    assert_eq!(trn.version(), "latest");
}

#[test]
fn test_stable_version() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .stable()
        .build()
        .unwrap();
    
    assert_eq!(trn.version(), "stable");
}

#[test]
fn test_semver() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .semver(1, 2, 3)
        .build()
        .unwrap();
    
    assert_eq!(trn.version(), "1.2.3");
}

#[test]
fn test_version_v() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version_v(2, 1, 0)
        .build()
        .unwrap();
    
    assert_eq!(trn.version(), "v2.1.0");
}

#[test]
fn test_version_major_minor() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version_major_minor(1, 5)
        .build()
        .unwrap();
    
    assert_eq!(trn.version(), "1.5");
}

// =================================
// Predefined Templates Tests
// =================================

#[test]
fn test_github_api_template() {
    let trn = TrnBuilder::user_tool("alice")
        .openapi_tool()
        .instance_id("github-api")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "github-api");
}

#[test]
fn test_slack_api_template() {
    let trn = TrnBuilder::org_tool("company")
        .openapi_tool()
        .instance_id("slack-api")
        .version("v2.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("company"));
    assert_eq!(trn.type_(), "openapi");
    assert_eq!(trn.instance_id(), "slack-api");
}

#[test]
fn test_data_pipeline_template() {
    let trn = TrnBuilder::pipeline()
        .platform("aiplatform")
        .type_("etl")
        .instance_id("etl-processor")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "pipeline");
    assert_eq!(trn.instance_id(), "etl-processor");
}

#[test]
fn test_python_script_template() {
    let trn = TrnBuilder::user_tool("alice")
        .python_tool()
        .instance_id("data-analysis")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "user");
    assert_eq!(trn.scope(), Some("alice"));
    assert_eq!(trn.type_(), "python");
    assert_eq!(trn.instance_id(), "data-analysis");
}

#[test]
fn test_ml_model_template() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .model()
        .type_("bert")
        .instance_id("bert-classifier")
        .version("v2.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "model");
    assert_eq!(trn.instance_id(), "bert-classifier");
    assert_eq!(trn.version(), "v2.0");
}

#[test]
fn test_dataset_template_predefined() {
    let trn = TrnBuilder::new()
        .platform("aiplatform")
        .dataset()
        .type_("csv")
        .instance_id("sales-data")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.resource_type(), "dataset");
    assert_eq!(trn.instance_id(), "sales-data");
    assert_eq!(trn.version(), "v1.0");
}

// =================================
// Error Cases Tests
// =================================

#[test]
fn test_builder_incomplete_build() {
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .build();
    
    assert!(result.is_err());
}

#[test]
fn test_builder_invalid_build() {
    // User platform without scope should fail
    let result = TrnBuilder::new()
        .platform("user")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("github-api")
        .version("v1.0")
        .build();
    
    assert!(result.is_err());
}

// =================================
// Convenience Function Tests
// =================================

#[test]
fn test_builder_convenience_function() {
    let trn = builder()
        .platform("aiplatform")
        .resource_type("model")
        .type_("bert")
        .instance_id("base-model")
        .version("v1.0")
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "aiplatform");
    assert_eq!(trn.resource_type(), "model");
}

// =================================
// Complex Builder Scenarios
// =================================

#[test]
fn test_builder_complex_scenario() {
    // Test building multiple related TRNs with different configurations
    let trn1 = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("api1")
        .version("v1.0")
        .build()
        .unwrap();
    
    let trn2 = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .instance_id("api2")
        .version("v2.0")
        .tag("stable")
        .build()
        .unwrap();
    
    assert_eq!(trn1.instance_id(), "api1");
    assert_eq!(trn1.version(), "v1.0");
    assert_eq!(trn2.instance_id(), "api2");
    assert_eq!(trn2.version(), "v2.0");
    assert_eq!(trn2.tag(), Some("stable"));
}


#[test]
fn test_builder_method_chaining() {
    let trn = TrnBuilder::new()
        .platform("org")
        .scope("company")
        .resource_type("tool")
        .type_("python")
        .subtype("script")
        .instance_id("data-processor")
        .version_v(1, 2, 3)
        .tag("production")
        .sha256_hash("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")
        .validate_on_build(true)
        .build()
        .unwrap();
    
    assert_eq!(trn.platform(), "org");
    assert_eq!(trn.scope(), Some("company"));
    assert_eq!(trn.resource_type(), "tool");
    assert_eq!(trn.type_(), "python");
    assert_eq!(trn.subtype(), Some("script"));
    assert_eq!(trn.instance_id(), "data-processor");
    assert_eq!(trn.version(), "v1.2.3");
    assert_eq!(trn.tag(), Some("production"));
    assert_eq!(trn.hash(), Some("sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"));
} 