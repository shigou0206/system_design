//! TRN builder pattern implementation
//!
//! This module provides a fluent builder interface for constructing TRN objects
//! with validation, convenience methods, and type safety for the simplified 6-component format.

use crate::error::{TrnError, TrnResult};
use crate::types::{Platform, ResourceType, Trn};

/// Fluent builder for TRN construction
#[derive(Debug, Default, Clone)]
pub struct TrnBuilder {
    platform: Option<String>,
    scope: Option<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    version: Option<String>,
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
            scope: Some(trn.scope().to_string()),
            resource_type: Some(trn.resource_type().to_string()),
            resource_id: Some(trn.resource_id().to_string()),
            version: Some(trn.version().to_string()),
            validate_on_build: true,
        }
    }

    /// Set the platform
    pub fn platform<S: Into<String>>(mut self, platform: S) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set the scope
    pub fn scope<S: Into<String>>(mut self, scope: S) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Set the resource type
    pub fn resource_type<S: Into<String>>(mut self, resource_type: S) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    /// Set the resource ID
    pub fn resource_id<S: Into<String>>(mut self, resource_id: S) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set the version
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the platform using enum
    pub fn platform_enum(mut self, platform: Platform) -> Self {
        self.platform = Some(platform.to_string());
        self
    }

    /// Set the resource type using enum
    pub fn resource_type_enum(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    /// Enable or disable validation on build
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate_on_build = validate;
        self
    }

    /// Build the TRN object
    pub fn build(self) -> TrnResult<Trn> {
        // Check required fields
        let platform = self.platform.ok_or_else(|| {
            TrnError::builder_missing_field("platform")
        })?;

        let scope = self.scope.ok_or_else(|| {
            TrnError::builder_missing_field("scope")
        })?;

        let resource_type = self.resource_type.ok_or_else(|| {
            TrnError::builder_missing_field("resource_type")
        })?;

        let resource_id = self.resource_id.ok_or_else(|| {
            TrnError::builder_missing_field("resource_id")
        })?;

        let version = self.version.ok_or_else(|| {
            TrnError::builder_missing_field("version")
        })?;

        // Create TRN object using public constructor
        Trn::new(platform, scope, resource_type, resource_id, version)
    }

    /// Build and return as string
    pub fn build_string(self) -> TrnResult<String> {
        let trn = self.build()?;
        Ok(trn.to_string())
    }

    /// Check if all required fields are set
    pub fn is_complete(&self) -> bool {
        self.platform.is_some()
            && self.scope.is_some()
            && self.resource_type.is_some()
            && self.resource_id.is_some()
            && self.version.is_some()
    }

    /// Get a list of missing required fields
    pub fn missing_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        
        if self.platform.is_none() {
            missing.push("platform");
        }
        if self.scope.is_none() {
            missing.push("scope");
        }
        if self.resource_type.is_none() {
            missing.push("resource_type");
        }
        if self.resource_id.is_none() {
            missing.push("resource_id");
        }
        if self.version.is_none() {
            missing.push("version");
        }
        
        missing
    }

    /// Reset the builder to default state
    pub fn reset(mut self) -> Self {
        self.platform = None;
        self.scope = None;
        self.resource_type = None;
        self.resource_id = None;
        self.version = None;
        self
    }

    /// Clone the current builder state
    pub fn clone_builder(&self) -> Self {
        self.clone()
    }
}

/// Create a new TRN builder
#[allow(dead_code)]
pub fn builder() -> TrnBuilder {
    TrnBuilder::new()
}

// Convenience methods for common platforms
impl TrnBuilder {
    /// Set platform to "user"
    pub fn user_platform(self) -> Self {
        self.platform("user")
    }

    /// Set platform to "system"
    pub fn system_platform(self) -> Self {
        self.platform("system")
    }

    /// Set platform to "github"
    pub fn github_platform(self) -> Self {
        self.platform("github")
    }

    /// Set platform to "local"
    pub fn local_platform(self) -> Self {
        self.platform("local")
    }
}

// Convenience methods for common resource types
impl TrnBuilder {
    /// Set resource type to "tool"
    pub fn tool_resource(self) -> Self {
        self.resource_type("tool")
    }

    /// Set resource type to "function"
    pub fn function_resource(self) -> Self {
        self.resource_type("function")
    }

    /// Set resource type to "workflow"
    pub fn workflow_resource(self) -> Self {
        self.resource_type("workflow")
    }

    /// Set resource type to "agent"
    pub fn agent_resource(self) -> Self {
        self.resource_type("agent")
    }
}

// Builder pattern for specific TRN types
impl TrnBuilder {
    /// Build a tool TRN with common defaults
    pub fn tool<S: Into<String>>(
        platform: S,
        scope: S,
        resource_id: S,
        version: S,
    ) -> TrnBuilder {
        TrnBuilder::new()
            .platform(platform)
            .scope(scope)
            .resource_type("tool")
            .resource_id(resource_id)
            .version(version)
    }

    /// Build a function TRN with common defaults
    pub fn function<S: Into<String>>(
        platform: S,
        scope: S,
        resource_id: S,
        version: S,
    ) -> TrnBuilder {
        TrnBuilder::new()
            .platform(platform)
            .scope(scope)
            .resource_type("function")
            .resource_id(resource_id)
            .version(version)
    }

    /// Build a workflow TRN with common defaults
    pub fn workflow<S: Into<String>>(
        platform: S,
        scope: S,
        resource_id: S,
        version: S,
    ) -> TrnBuilder {
        TrnBuilder::new()
            .platform(platform)
            .scope(scope)
            .resource_type("workflow")
            .resource_id(resource_id)
            .version(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_complete_workflow() {
        let trn = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .resource_id("getUserById")
            .version("v1.0")
            .build()
            .expect("Should build successfully");

        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), "alice");
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.resource_id(), "getUserById");
        assert_eq!(trn.version(), "v1.0");
    }

    #[test]
    fn test_builder_missing_required_field() {
        let result = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            // Missing resource_id
            .version("v1.0")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_from_trn() {
        let original = Trn::new("user", "alice", "tool", "getUserById", "v1.0")
            .expect("Should create TRN");

        let builder_trn = TrnBuilder::from_trn(&original)
            .build()
            .expect("Should build from existing TRN");

        assert_eq!(original.to_string(), builder_trn.to_string());
    }

    #[test]
    fn test_convenience_constructors() {
        let trn = TrnBuilder::tool("user", "alice", "getUserById", "v1.0")
            .build()
            .expect("Should build tool TRN");

        assert_eq!(trn.resource_type(), "tool");
    }

    #[test]
    fn test_is_complete() {
        let incomplete = TrnBuilder::new()
            .platform("user")
            .scope("alice");

        assert!(!incomplete.is_complete());

        let complete = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .resource_id("getUserById")
            .version("v1.0");

        assert!(complete.is_complete());
    }

    #[test]
    fn test_missing_fields() {
        let builder = TrnBuilder::new()
            .platform("user")
            .scope("alice");

        let missing = builder.missing_fields();
        assert!(missing.contains(&"resource_type"));
        assert!(missing.contains(&"resource_id"));
        assert!(missing.contains(&"version"));
        assert!(!missing.contains(&"platform"));
        assert!(!missing.contains(&"scope"));
    }

    #[test]
    fn test_build_string() {
        let trn_string = TrnBuilder::new()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .resource_id("getUserById")
            .version("v1.0")
            .build_string()
            .expect("Should build as string");

        assert_eq!(trn_string, "trn:user:alice:tool:getUserById:v1.0");
    }
} 