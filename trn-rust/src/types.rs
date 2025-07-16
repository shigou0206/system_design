//! Type definitions for TRN structures
//!
//! This module contains all the core type definitions for representing
//! TRN (Tool Resource Name) structures in Rust.

use crate::constants::*;
use crate::error::{TrnError, TrnResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Platform types that are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// User platform - user-owned resources
    User,
    /// Organization platform - org-owned resources  
    Org,
    /// AI Platform - platform-owned resources
    AiPlatform,
    /// Custom platform
    Custom(String),
}

impl Platform {
    /// Check if platform is valid
    pub fn is_valid(&self) -> bool {
        match self {
            Platform::User | Platform::Org | Platform::AiPlatform => true,
            Platform::Custom(name) => !name.is_empty() && name.len() <= PLATFORM_MAX_LENGTH,
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::User => write!(f, "user"),
            Platform::Org => write!(f, "org"),
            Platform::AiPlatform => write!(f, "aiplatform"),
            Platform::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl FromStr for Platform {
    type Err = TrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Platform::User),
            "org" => Ok(Platform::Org),
            "aiplatform" => Ok(Platform::AiPlatform),
            other => {
                if other.is_empty() || other.len() > PLATFORM_MAX_LENGTH {
                    Err(TrnError::validation(
                        "Invalid platform name".to_string(),
                        "platform_format".to_string(),
                        Some(s.to_string()),
                    ))
                } else {
                    Ok(Platform::Custom(other.to_string()))
                }
            }
        }
    }
}

/// Resource types that are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Tool resource
    Tool,
    /// Model resource
    Model,
    /// Dataset resource
    Dataset,
    /// Pipeline resource
    Pipeline,
    /// Custom resource type
    Custom(String),
}

impl ResourceType {
    /// Check if resource type is valid
    pub fn is_valid(&self) -> bool {
        match self {
            ResourceType::Tool | ResourceType::Model | ResourceType::Dataset | ResourceType::Pipeline => true,
            ResourceType::Custom(name) => !name.is_empty() && name.len() <= RESOURCE_TYPE_MAX_LENGTH,
        }
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Tool => write!(f, "tool"),
            ResourceType::Model => write!(f, "model"),
            ResourceType::Dataset => write!(f, "dataset"),
            ResourceType::Pipeline => write!(f, "pipeline"),
            ResourceType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl FromStr for ResourceType {
    type Err = TrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tool" => Ok(ResourceType::Tool),
            "model" => Ok(ResourceType::Model),
            "dataset" => Ok(ResourceType::Dataset),
            "pipeline" => Ok(ResourceType::Pipeline),
            other => {
                if other.is_empty() || other.len() > RESOURCE_TYPE_MAX_LENGTH {
                    Err(TrnError::validation(
                        "Invalid resource type name".to_string(),
                        "resource_type_format".to_string(),
                        Some(s.to_string()),
                    ))
                } else {
                    Ok(ResourceType::Custom(other.to_string()))
                }
            }
        }
    }
}

/// TRN components structure for zero-copy parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrnComponents<'a> {
    /// Platform identifier
    pub platform: &'a str,
    /// Scope identifier (required)
    pub scope: &'a str,
    /// Resource type
    pub resource_type: &'a str,
    /// Resource identifier
    pub resource_id: &'a str,
    /// Version identifier
    pub version: &'a str,
}

impl<'a> TrnComponents<'a> {
    /// Create new TRN components
    pub fn new(
        platform: &'a str,
        scope: &'a str,
        resource_type: &'a str,
        resource_id: &'a str,
        version: &'a str,
    ) -> Self {
        Self {
            platform,
            scope,
            resource_type,
            resource_id,
            version,
        }
    }

    /// Convert to owned TRN
    pub fn to_owned(&self) -> Trn {
        Trn {
            platform: self.platform.to_string(),
            scope: self.scope.to_string(),
            resource_type: self.resource_type.to_string(),
            resource_id: self.resource_id.to_string(),
            version: self.version.to_string(),
        }
    }
}

/// Main TRN structure (owned variant)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Trn {
    /// Platform identifier
    platform: String,
    /// Scope identifier (required)
    scope: String,
    /// Resource type
    resource_type: String,
    /// Resource identifier
    resource_id: String,
    /// Version identifier
    version: String,
}

impl Trn {
    /// Create a new TRN with validation
    pub fn new(
        platform: impl Into<String>,
        scope: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        version: impl Into<String>,
    ) -> TrnResult<Self> {
        let trn = Self {
            platform: platform.into(),
            scope: scope.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            version: version.into(),
        };
        
        trn.validate()?;
        Ok(trn)
    }

    /// Parse a TRN string
    pub fn parse(input: &str) -> TrnResult<Self> {
        crate::parsing::parse_trn(input)
    }

    /// Create TRN from components
    pub fn from_components(components: TrnComponents<'_>) -> TrnResult<Self> {
        let trn = components.to_owned();
        trn.validate()?;
        Ok(trn)
    }

    /// Validate this TRN
    pub fn validate(&self) -> TrnResult<()> {
        crate::validation::validate_trn_struct(self)
    }

    /// Check if this TRN is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    // Accessors
    /// Get the platform
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Get the scope
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// Get the resource type
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    /// Get the resource ID
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    // Conversion methods
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!(
            "trn:{}:{}:{}:{}:{}",
            self.platform,
            self.scope,
            self.resource_type,
            self.resource_id,
            self.version
        )
    }

    /// Convert to URL format
    pub fn to_url(&self) -> TrnResult<String> {
        crate::url::trn_to_url(self)
    }

    /// Convert to HTTP URL
    pub fn to_http_url(&self, base: &str) -> TrnResult<String> {
        crate::url::trn_to_http_url(self, base)
    }

    // Manipulation methods
    /// Get the base TRN (without version)
    pub fn base_trn(&self) -> Self {
        Self {
            platform: self.platform.clone(),
            scope: self.scope.clone(),
            resource_type: self.resource_type.clone(),
            resource_id: self.resource_id.clone(),
            version: "*".to_string(),
        }
    }

    /// Check if this TRN matches a pattern
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        crate::pattern::matches_pattern(&self.to_string(), pattern)
    }

    /// Check if this TRN is compatible with another TRN
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.platform == other.platform
            && self.scope == other.scope
            && self.resource_type == other.resource_type
            && self.resource_id == other.resource_id
    }

    // Mutable operations
    /// Set the scope
    pub fn set_scope(&mut self, scope: String) {
        self.scope = scope;
    }

    /// Set the version
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Get components as borrowed structure
    pub fn components(&self) -> TrnComponents<'_> {
        TrnComponents {
            platform: &self.platform,
            scope: &self.scope,
            resource_type: &self.resource_type,
            resource_id: &self.resource_id,
            version: &self.version,
        }
    }

    /// Parse TRN from URL
    pub fn from_url(url: &str) -> TrnResult<Self> {
        crate::url::url_to_trn(url)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to YAML string
    #[cfg(feature = "cli")]
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Serialize to TOML string
    #[cfg(feature = "cli")]
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }
}

impl fmt::Display for Trn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "trn:{}:{}:{}:{}:{}",
            self.platform,
            self.scope,
            self.resource_type,
            self.resource_id,
            self.version
        )
    }
}

impl FromStr for Trn {
    type Err = TrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<TrnComponents<'_>> for Trn {
    fn from(components: TrnComponents<'_>) -> Self {
        components.to_owned()
    }
}