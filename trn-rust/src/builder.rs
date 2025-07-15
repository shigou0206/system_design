//! TRN builder pattern implementation
//!
//! This module provides a fluent builder interface for constructing TRN objects
//! with validation, convenience methods, and type safety.

use crate::constants::*;
use crate::error::{TrnError, TrnResult};
use crate::types::{Platform, ResourceType, ToolType, Trn};

/// Fluent builder for TRN construction
#[derive(Debug, Default, Clone)]
pub struct TrnBuilder {
    platform: Option<String>,
    scope: Option<String>,
    resource_type: Option<String>,
    type_: Option<String>,
    subtype: Option<String>,
    instance_id: Option<String>,
    version: Option<String>,
    tag: Option<String>,
    hash: Option<String>,
    validate_on_build: bool,
}

impl TrnBuilder {
    /// Create a new TRN builder
    pub fn new() -> Self {
        Self {
            validate_on_build: true,
            ..Default::default()
        }
    }

    /// Create a builder from an existing TRN
    pub fn from_trn(trn: &Trn) -> Self {
        Self {
            platform: Some(trn.platform().to_string()),
            scope: trn.scope().map(String::from),
            resource_type: Some(trn.resource_type().to_string()),
            type_: Some(trn.type_().to_string()),
            subtype: trn.subtype().map(String::from),
            instance_id: Some(trn.instance_id().to_string()),
            version: Some(trn.version().to_string()),
            tag: trn.tag().map(String::from),
            hash: trn.hash().map(String::from),
            validate_on_build: true,
        }
    }

    /// Set the platform
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set platform using enum
    pub fn platform_enum(mut self, platform: Platform) -> Self {
        self.platform = Some(platform.to_string());
        self
    }

    /// Set the scope
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Set the resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    /// Set resource type using enum
    pub fn resource_type_enum(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    /// Set the type
    pub fn type_(mut self, type_: impl Into<String>) -> Self {
        self.type_ = Some(type_.into());
        self
    }

    /// Set tool type using enum (convenience for resource_type = tool)
    pub fn tool_type(mut self, tool_type: ToolType) -> Self {
        self.resource_type = Some("tool".to_string());
        self.type_ = Some(tool_type.to_string());
        self
    }

    /// Set the subtype
    pub fn subtype(mut self, subtype: impl Into<String>) -> Self {
        self.subtype = Some(subtype.into());
        self
    }

    /// Set the instance ID
    pub fn instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = Some(instance_id.into());
        self
    }

    /// Set the version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set the hash
    pub fn hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }

    /// Set SHA-256 hash (convenience method)
    pub fn sha256_hash(mut self, hash: impl Into<String>) -> Self {
        let hash_str = hash.into();
        self.hash = Some(format!("sha256:{}", hash_str));
        self
    }

    /// Set MD5 hash (convenience method)
    pub fn md5_hash(mut self, hash: impl Into<String>) -> Self {
        let hash_str = hash.into();
        self.hash = Some(format!("md5:{}", hash_str));
        self
    }

    /// Enable or disable validation on build
    pub fn validate_on_build(mut self, validate: bool) -> Self {
        self.validate_on_build = validate;
        self
    }

    /// Clear the scope
    pub fn no_scope(mut self) -> Self {
        self.scope = None;
        self
    }

    /// Clear the subtype
    pub fn no_subtype(mut self) -> Self {
        self.subtype = None;
        self
    }

    /// Clear the tag
    pub fn no_tag(mut self) -> Self {
        self.tag = None;
        self
    }

    /// Clear the hash
    pub fn no_hash(mut self) -> Self {
        self.hash = None;
        self
    }

    /// Build the TRN object
    pub fn build(self) -> TrnResult<Trn> {
        // Check required fields
        let platform = self.platform.ok_or_else(|| {
            TrnError::builder_missing_field("platform")
        })?;

        let resource_type = self.resource_type.ok_or_else(|| {
            TrnError::builder_missing_field("resource_type")
        })?;

        let type_ = self.type_.ok_or_else(|| {
            TrnError::builder_missing_field("type")
        })?;

        let instance_id = self.instance_id.ok_or_else(|| {
            TrnError::builder_missing_field("instance_id")
        })?;

        let version = self.version.ok_or_else(|| {
            TrnError::builder_missing_field("version")
        })?;

        // Create TRN object
        Trn::new_full(
            platform,
            self.scope,
            resource_type,
            type_,
            self.subtype,
            instance_id,
            version,
            self.tag,
            self.hash,
        )
    }

    /// Build and return as string
    pub fn build_string(self) -> TrnResult<String> {
        let trn = self.build()?;
        Ok(trn.to_string())
    }

    /// Check if all required fields are set
    pub fn is_complete(&self) -> bool {
        self.platform.is_some()
            && self.resource_type.is_some()
            && self.type_.is_some()
            && self.instance_id.is_some()
            && self.version.is_some()
    }

    /// Get a list of missing required fields
    pub fn missing_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();

        if self.platform.is_none() {
            missing.push("platform");
        }
        if self.resource_type.is_none() {
            missing.push("resource_type");
        }
        if self.type_.is_none() {
            missing.push("type");
        }
        if self.instance_id.is_none() {
            missing.push("instance_id");
        }
        if self.version.is_none() {
            missing.push("version");
        }

        missing
    }

    /// Reset all fields to None
    pub fn reset(mut self) -> Self {
        self.platform = None;
        self.scope = None;
        self.resource_type = None;
        self.type_ = None;
        self.subtype = None;
        self.instance_id = None;
        self.version = None;
        self.tag = None;
        self.hash = None;
        self
    }

    /// Clone current state and continue building
    pub fn fork(&self) -> Self {
        self.clone()
    }
}

/// Convenience methods for common TRN patterns
impl TrnBuilder {
    /// Create a user tool TRN
    pub fn user_tool(username: impl Into<String>) -> Self {
        Self::new()
            .platform("user")
            .scope(username)
            .resource_type("tool")
    }

    /// Create an organization tool TRN
    pub fn org_tool(org_name: impl Into<String>) -> Self {
        Self::new()
            .platform("org")
            .scope(org_name)
            .resource_type("tool")
    }

    /// Create a system tool TRN
    pub fn system_tool() -> Self {
        Self::new()
            .platform("aiplatform")
            .resource_type("tool")
    }

    /// Create an OpenAPI tool TRN
    pub fn openapi_tool(mut self) -> Self {
        self.resource_type = Some("tool".to_string());
        self.type_ = Some("openapi".to_string());
        self
    }

    /// Create a tool TRN template with common defaults
    pub fn tool_template(
        scope: impl Into<String>,
        instance_id: impl Into<String>,
        version: impl Into<String>
    ) -> Self {
        Self::new()
            .platform("user")
            .scope(scope)
            .resource_type("tool")
            .type_("openapi")
            .instance_id(instance_id)
            .version(version)
    }

    /// Create a Python tool TRN
    pub fn python_tool(mut self) -> Self {
        self.resource_type = Some("tool".to_string());
        self.type_ = Some("python".to_string());
        self
    }

    /// Create a workflow tool TRN
    pub fn workflow_tool() -> Self {
        Self::new()
            .resource_type("tool")
            .type_("workflow")
    }

    /// Create a dataset TRN
    pub fn dataset(mut self) -> Self {
        self.resource_type = Some("dataset".to_string());
        self
    }

    /// Create a model TRN
    pub fn model(mut self) -> Self {
        self.resource_type = Some("model".to_string());
        self
    }

    /// Create a pipeline TRN
    pub fn pipeline() -> Self {
        Self::new()
            .resource_type("pipeline")
    }

    /// Set latest version
    pub fn latest(mut self) -> Self {
        self.version = Some("latest".to_string());
        self
    }

    /// Set stable version
    pub fn stable(mut self) -> Self {
        self.version = Some("stable".to_string());
        self
    }

    /// Set semantic version (e.g., "1.2.3")
    pub fn semver(mut self, major: u32, minor: u32, patch: u32) -> Self {
        self.version = Some(format!("{}.{}.{}", major, minor, patch));
        self
    }

    /// Set version with v prefix (e.g., "v1.2.3")
    pub fn version_v(mut self, major: u32, minor: u32, patch: u32) -> Self {
        self.version = Some(format!("v{}.{}.{}", major, minor, patch));
        self
    }

    /// Set major.minor version
    pub fn version_major_minor(mut self, major: u32, minor: u32) -> Self {
        self.version = Some(format!("{}.{}", major, minor));
        self
    }
}

/// Template-based builder for common patterns
#[allow(dead_code)]
pub struct TrnTemplate;
#[allow(dead_code)]
impl TrnTemplate {
    /// GitHub API tool template
    #[allow(dead_code)]
    pub fn github_api(username: impl Into<String>) -> TrnBuilder {
        TrnBuilder::user_tool(username)
            .tool_type(ToolType::OpenApi)
            .instance_id("github-api")
            .latest()
    }

    /// Slack API tool template
    #[allow(dead_code)]
    pub fn slack_api(org: impl Into<String>) -> TrnBuilder {
        TrnBuilder::org_tool(org)
            .tool_type(ToolType::OpenApi)
            .instance_id("slack-api")
            .latest()
    }

    /// Data processing pipeline template
    #[allow(dead_code)]
    pub fn data_pipeline(name: impl Into<String>) -> TrnBuilder {
        TrnBuilder::system_tool()
            .type_("workflow")
            .instance_id(name)
            .latest()
    }

    /// Python script tool template
    pub fn python_script(username: impl Into<String>, script_name: impl Into<String>) -> TrnBuilder {
        TrnBuilder::user_tool(username)
            .python_tool()
            .instance_id(script_name)
            .latest()
    }

    /// Machine learning model template
    pub fn ml_model(name: impl Into<String>, version: impl Into<String>) -> TrnBuilder {
        TrnBuilder::new()
            .platform("aiplatform")
            .model()
            .type_("ml")
            .instance_id(name)
            .version(version)
    }

    /// Dataset template
    pub fn dataset_template(name: impl Into<String>, version: impl Into<String>) -> TrnBuilder {
        TrnBuilder::new()
            .platform("aiplatform")
            .dataset()
            .type_("structured")
            .instance_id(name)
            .version(version)
    }
}

/// Validation builder for step-by-step TRN construction with validation
#[allow(dead_code)]
pub struct ValidatedTrnBuilder {
    builder: TrnBuilder,
}

#[allow(dead_code)]
impl ValidatedTrnBuilder {
    /// Create a new validated builder
    pub fn new() -> Self {
        Self {
            builder: TrnBuilder::new(),
        }
    }

    /// Set platform with validation
    pub fn platform(mut self, platform: impl Into<String>) -> TrnResult<Self> {
        let platform_str = platform.into();
        
        // Validate platform
        if !PLATFORM_REGEX.is_match(&platform_str) {
            return Err(TrnError::builder_invalid_field(
                "platform",
                "Invalid platform format",
            ));
        }

        if is_reserved_word(&platform_str) {
            return Err(TrnError::builder_invalid_field(
                "platform",
                "Platform uses reserved word",
            ));
        }

        self.builder = self.builder.platform(platform_str);
        Ok(self)
    }

    /// Set scope with validation
    pub fn scope(mut self, scope: impl Into<String>) -> TrnResult<Self> {
        let scope_str = scope.into();
        
        if !SCOPE_REGEX.is_match(&scope_str) {
            return Err(TrnError::builder_invalid_field(
                "scope",
                "Invalid scope format",
            ));
        }

        self.builder = self.builder.scope(scope_str);
        Ok(self)
    }

    /// Continue with regular builder after validation
    pub fn continue_building(self) -> TrnBuilder {
        self.builder
    }

    /// Build with final validation
    pub fn build(self) -> TrnResult<Trn> {
        self.builder.build()
    }
}

impl Default for ValidatedTrnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_builder() {
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
    fn test_missing_fields() {
        let builder = TrnBuilder::new()
            .platform("user")
            .scope("alice");

        assert!(!builder.is_complete());
        let missing = builder.missing_fields();
        assert!(missing.contains(&"resource_type"));
        assert!(missing.contains(&"type"));
        assert!(missing.contains(&"instance_id"));
        assert!(missing.contains(&"version"));
    }

    #[test]
    fn test_builder_error_missing_field() {
        let result = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .build();

        assert!(result.is_err());
        if let Err(TrnError::BuilderMissingField { field }) = result {
            assert_eq!(field, "resource_type");
        } else {
            panic!("Expected BuilderMissingField error");
        }
    }

    #[test]
    fn test_convenience_methods() {
        let trn = TrnBuilder::user_tool("alice")
            .openapi_tool()
            .instance_id("github-api")
            .latest()
            .build()
            .unwrap();

        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.type_(), "openapi");
        assert_eq!(trn.version(), "latest");
    }

    #[test]
    fn test_templates() {
        let trn = TrnTemplate::github_api("alice")
            .build()
            .unwrap();

        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
        assert_eq!(trn.type_(), "openapi");
        assert_eq!(trn.instance_id(), "github-api");
        assert_eq!(trn.version(), "latest");
    }

    #[test]
    fn test_hash_convenience() {
        let trn = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .type_("openapi")
            .instance_id("api")
            .version("v1.0")
            .sha256_hash("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
            .build()
            .unwrap();

        assert_eq!(trn.hash(), Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    }

    #[test]
    fn test_builder_fork() {
        let base_builder = TrnBuilder::user_tool("alice")
            .openapi_tool()
            .instance_id("api");

        let trn1 = base_builder.fork()
            .version("v1.0")
            .build()
            .unwrap();

        let trn2 = base_builder.fork()
            .version("v2.0")
            .tag("beta")
            .build()
            .unwrap();

        assert_eq!(trn1.version(), "v1.0");
        assert_eq!(trn2.version(), "v2.0");
        assert_eq!(trn2.tag(), Some("beta"));
    }

    #[test]
    fn test_from_trn() {
        // Simple test that reproduces the issue
        let original = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .type_("openapi")
            .instance_id("api")
            .version("v1.0")
            .build()
            .unwrap();

        let modified = TrnBuilder::from_trn(&original)
            .version("v2.0")
            .tag("beta")
            .build()
            .unwrap();

        assert_eq!(modified.platform(), "user");
        assert_eq!(modified.scope(), Some("alice"));
        assert_eq!(modified.version(), "v2.0");
        assert_eq!(modified.tag(), Some("beta"));
    }

    #[test]
    fn test_validated_builder() {
        let trn = ValidatedTrnBuilder::new()
            .platform("user").unwrap()
            .scope("alice").unwrap()
            .continue_building()
            .resource_type("tool")
            .type_("openapi")
            .instance_id("api")
            .version("v1.0")
            .build()
            .unwrap();

        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
    }

    #[test]
    fn test_validated_builder_error() {
        let result = ValidatedTrnBuilder::new()
            .platform("INVALID");  // uppercase not allowed

        assert!(result.is_err());
    }

    #[test]
    fn test_semantic_versioning() {
        let trn = TrnBuilder::user_tool("alice")
            .openapi_tool()
            .instance_id("api")
            .semver(1, 2, 3)
            .build()
            .unwrap();

        assert_eq!(trn.version(), "1.2.3");

        let trn_v = TrnBuilder::user_tool("alice")
            .openapi_tool()
            .instance_id("api")
            .version_v(1, 2, 3)
            .build()
            .unwrap();

        assert_eq!(trn_v.version(), "v1.2.3");
    }

    #[test]
    fn test_build_string() {
        let trn_str = TrnBuilder::user_tool("alice")
            .openapi_tool()
            .instance_id("github-api")
            .version("v1.0")
            .build_string()
            .unwrap();

        assert_eq!(trn_str, "trn:user:alice:tool:openapi:github-api:v1.0");
    }
} 