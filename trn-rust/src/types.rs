//! Core TRN types and data structures
//!
//! This module defines the main TRN types, enums, and structures used throughout
//! the library. It provides both owned and borrowed variants for optimal performance.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::constants::*;
use crate::error::{TrnError, TrnResult};

/// Platform enumeration for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// System platform for built-in tools
    #[serde(rename = "aiplatform")]
    AiPlatform,
    /// User platform for personal tools
    User,
    /// Organization platform for company tools
    Org,
    /// Custom platform (runtime validation required)
    #[serde(untagged)]
    Custom(CustomPlatform),
}

/// Custom platform wrapper for additional validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomPlatform(String);

impl CustomPlatform {
    /// Create a new custom platform with validation
    pub fn new(name: String) -> TrnResult<Self> {
        if !PLATFORM_REGEX.is_match(&name) {
            return Err(TrnError::validation(
                format!("Invalid platform name format: '{}'", name),
                "platform_format".to_string(),
                None,
            ));
        }
        
        if is_reserved_word(&name) {
            return Err(TrnError::reserved_word(name.clone(), "platform".to_string(), None));
        }
        
        Ok(Self(name))
    }
    
    /// Get the platform name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AiPlatform => write!(f, "aiplatform"),
            Self::User => write!(f, "user"),
            Self::Org => write!(f, "org"),
            Self::Custom(custom) => write!(f, "{}", custom.0),
        }
    }
}

impl FromStr for Platform {
    type Err = TrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aiplatform" => Ok(Self::AiPlatform),
            "user" => Ok(Self::User),
            "org" => Ok(Self::Org),
            _ => Ok(Self::Custom(CustomPlatform::new(s.to_string())?)),
        }
    }
}

impl From<Platform> for String {
    fn from(platform: Platform) -> Self {
        platform.to_string()
    }
}

impl From<&Platform> for String {
    fn from(platform: &Platform) -> Self {
        platform.to_string()
    }
}

/// Resource type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    /// Executable tools
    Tool,
    /// Data resources
    Dataset,
    /// Workflow templates
    Pipeline,
    /// AI model resources
    Model,
    /// AI agent resources
    Agent,
    /// Workflow resources
    Workflow,
    /// Custom resource type
    #[serde(untagged)]
    Custom(CustomResourceType),
}

/// Custom resource type wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomResourceType(String);

impl CustomResourceType {
    /// Create a new custom resource type with validation
    pub fn new(name: String) -> TrnResult<Self> {
        if !RESOURCE_TYPE_REGEX.is_match(&name) {
            return Err(TrnError::validation(
                format!("Invalid resource type format: '{}'", name),
                "resource_type_format".to_string(),
                None,
            ));
        }
        
        if is_reserved_word(&name) {
            return Err(TrnError::reserved_word(name.clone(), "resource_type".to_string(), None));
        }
        
        Ok(Self(name))
    }
    
    /// Get the resource type name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tool => write!(f, "tool"),
            Self::Dataset => write!(f, "dataset"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Model => write!(f, "model"),
            Self::Agent => write!(f, "agent"),
            Self::Workflow => write!(f, "workflow"),
            Self::Custom(custom) => write!(f, "{}", custom.0),
        }
    }
}

impl FromStr for ResourceType {
    type Err = TrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tool" => Ok(Self::Tool),
            "dataset" => Ok(Self::Dataset),
            "pipeline" => Ok(Self::Pipeline),
            "model" => Ok(Self::Model),
            "agent" => Ok(Self::Agent),
            "workflow" => Ok(Self::Workflow),
            _ => Ok(Self::Custom(CustomResourceType::new(s.to_string())?)),
        }
    }
}

impl From<ResourceType> for String {
    fn from(resource_type: ResourceType) -> Self {
        resource_type.to_string()
    }
}

impl From<&ResourceType> for String {
    fn from(resource_type: &ResourceType) -> Self {
        resource_type.to_string()
    }
}

/// Tool type enumeration (for resource_type = Tool)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// RESTful API tools
    #[serde(rename = "openapi")]
    OpenApi,
    /// Business process tools
    Workflow,
    /// Python execution tools
    Python,
    /// Shell command tools
    Shell,
    /// System operation tools
    System,
    /// Function tools
    Function,
    /// Composite tools
    Composite,
    /// Async/Event-driven API tools
    #[serde(rename = "async_api")]
    AsyncApi,
    /// Custom tool type
    #[serde(untagged)]
    Custom(CustomToolType),
}

/// Custom tool type wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomToolType(String);

impl CustomToolType {
    /// Create a new custom tool type with validation
    pub fn new(name: String) -> TrnResult<Self> {
        if !TYPE_REGEX.is_match(&name) {
            return Err(TrnError::validation(
                format!("Invalid tool type format: '{}'", name),
                "tool_type_format".to_string(),
                None,
            ));
        }
        
        if is_reserved_word(&name) {
            return Err(TrnError::reserved_word(name.clone(), "type".to_string(), None));
        }
        
        Ok(Self(name))
    }
    
    /// Get the tool type name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenApi => write!(f, "openapi"),
            Self::Workflow => write!(f, "workflow"),
            Self::Python => write!(f, "python"),
            Self::Shell => write!(f, "shell"),
            Self::System => write!(f, "system"),
            Self::Function => write!(f, "function"),
            Self::Composite => write!(f, "composite"),
            Self::AsyncApi => write!(f, "async_api"),
            Self::Custom(custom) => write!(f, "{}", custom.0),
        }
    }
}

impl FromStr for ToolType {
    type Err = TrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openapi" => Ok(Self::OpenApi),
            "workflow" => Ok(Self::Workflow),
            "python" => Ok(Self::Python),
            "shell" => Ok(Self::Shell),
            "system" => Ok(Self::System),
            "function" => Ok(Self::Function),
            "composite" => Ok(Self::Composite),
            "async_api" => Ok(Self::AsyncApi),
            _ => Ok(Self::Custom(CustomToolType::new(s.to_string())?)),
        }
    }
}

impl From<ToolType> for String {
    fn from(tool_type: ToolType) -> Self {
        tool_type.to_string()
    }
}

impl From<&ToolType> for String {
    fn from(tool_type: &ToolType) -> Self {
        tool_type.to_string()
    }
}

/// TRN components structure for zero-copy parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrnComponents<'a> {
    /// Platform identifier
    pub platform: &'a str,
    /// Scope identifier (optional)
    pub scope: Option<&'a str>,
    /// Resource type
    pub resource_type: &'a str,
    /// Type identifier
    pub type_: &'a str,
    /// Subtype identifier (optional)
    pub subtype: Option<&'a str>,
    /// Instance identifier
    pub instance_id: &'a str,
    /// Version identifier
    pub version: &'a str,
    /// Tag identifier (optional)
    pub tag: Option<&'a str>,
    /// Hash value (optional)
    pub hash: Option<&'a str>,
}

impl<'a> TrnComponents<'a> {
    /// Create new TRN components
    pub fn new(
        platform: &'a str,
        resource_type: &'a str,
        type_: &'a str,
        instance_id: &'a str,
        version: &'a str,
    ) -> Self {
        Self {
            platform,
            scope: None,
            resource_type,
            type_,
            subtype: None,
            instance_id,
            version,
            tag: None,
            hash: None,
        }
    }

    /// Set the scope
    pub fn with_scope(mut self, scope: &'a str) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set the subtype
    pub fn with_subtype(mut self, subtype: &'a str) -> Self {
        self.subtype = Some(subtype);
        self
    }

    /// Set the tag
    pub fn with_tag(mut self, tag: &'a str) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Set the hash
    pub fn with_hash(mut self, hash: &'a str) -> Self {
        self.hash = Some(hash);
        self
    }

    /// Convert to owned TRN
    pub fn to_owned(&self) -> Trn {
        Trn {
            platform: self.platform.to_string(),
            scope: self.scope.map(String::from),
            resource_type: self.resource_type.to_string(),
            type_: self.type_.to_string(),
            subtype: self.subtype.map(String::from),
            instance_id: self.instance_id.to_string(),
            version: self.version.to_string(),
            tag: self.tag.map(String::from),
            hash: self.hash.map(String::from),
        }
    }
}

/// Main TRN structure (owned variant)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Trn {
    /// Platform identifier
    platform: String,
    /// Scope identifier (optional)
    scope: Option<String>,
    /// Resource type
    resource_type: String,
    /// Type identifier
    type_: String,
    /// Subtype identifier (optional)
    subtype: Option<String>,
    /// Instance identifier
    instance_id: String,
    /// Version identifier
    version: String,
    /// Tag identifier (optional)
    tag: Option<String>,
    /// Hash value (optional)
    hash: Option<String>,
}

impl Trn {
    /// Create a new TRN with validation
    pub fn new(
        platform: impl Into<String>,
        resource_type: impl Into<String>,
        type_: impl Into<String>,
        instance_id: impl Into<String>,
        version: impl Into<String>,
    ) -> TrnResult<Self> {
        let trn = Self {
            platform: platform.into(),
            scope: None,
            resource_type: resource_type.into(),
            type_: type_.into(),
            subtype: None,
            instance_id: instance_id.into(),
            version: version.into(),
            tag: None,
            hash: None,
        };
        
        trn.validate()?;
        Ok(trn)
    }

    /// Create a new TRN with all components
    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        platform: impl Into<String>,
        scope: Option<impl Into<String>>,
        resource_type: impl Into<String>,
        type_: impl Into<String>,
        subtype: Option<impl Into<String>>,
        instance_id: impl Into<String>,
        version: impl Into<String>,
        tag: Option<impl Into<String>>,
        hash: Option<impl Into<String>>,
    ) -> TrnResult<Self> {
        let trn = Self {
            platform: platform.into(),
            scope: scope.map(Into::into),
            resource_type: resource_type.into(),
            type_: type_.into(),
            subtype: subtype.map(Into::into),
            instance_id: instance_id.into(),
            version: version.into(),
            tag: tag.map(Into::into),
            hash: hash.map(Into::into),
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
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    /// Get the resource type
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    /// Get the type
    pub fn type_(&self) -> &str {
        &self.type_
    }

    /// Get the subtype
    pub fn subtype(&self) -> Option<&str> {
        self.subtype.as_deref()
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the tag
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// Get the hash
    pub fn hash(&self) -> Option<&str> {
        self.hash.as_deref()
    }

    // Conversion methods
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        // Use fixed structure format - all 8 components always present
        let scope = self.scope.as_deref().unwrap_or("");
        let subtype = self.subtype.as_deref().unwrap_or("");
        let tag = self.tag.as_deref().unwrap_or("");
        
        let mut result = format!(
            "trn:{}:{}:{}:{}:{}:{}:{}:{}",
            self.platform,
            scope,
            self.resource_type,
            self.type_,
            subtype,
            self.instance_id,
            self.version,
            tag
        );

        if let Some(hash) = &self.hash {
            result.push(TRN_HASH_SEPARATOR);
            result.push_str(hash);
        }

        result
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
    /// Create a copy without the hash
    pub fn without_hash(&self) -> Self {
        let mut trn = self.clone();
        trn.hash = None;
        trn
    }

    /// Create a copy without the tag
    pub fn without_tag(&self) -> Self {
        let mut trn = self.clone();
        trn.tag = None;
        trn
    }

    /// Get the base TRN (without version, tag, and hash)
    pub fn base_trn(&self) -> Self {
        Self {
            platform: self.platform.clone(),
            scope: self.scope.clone(),
            resource_type: self.resource_type.clone(),
            type_: self.type_.clone(),
            subtype: self.subtype.clone(),
            instance_id: self.instance_id.clone(),
            version: "*".to_string(),
            tag: None,
            hash: None,
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
            && self.type_ == other.type_
            && self.subtype == other.subtype
            && self.instance_id == other.instance_id
    }

    // Mutable operations
    /// Set the scope
    pub fn set_scope(&mut self, scope: Option<String>) {
        self.scope = scope;
    }

    /// Set the subtype
    pub fn set_subtype(&mut self, subtype: Option<String>) {
        self.subtype = subtype;
    }

    /// Set the version
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Set the tag
    pub fn set_tag(&mut self, tag: Option<String>) {
        self.tag = tag;
    }

    /// Set the hash
    pub fn set_hash(&mut self, hash: Option<String>) {
        self.hash = hash;
    }

    /// Get components as borrowed structure
    pub fn components(&self) -> TrnComponents<'_> {
        TrnComponents {
            platform: &self.platform,
            scope: self.scope.as_deref(),
            resource_type: &self.resource_type,
            type_: &self.type_,
            subtype: self.subtype.as_deref(),
            instance_id: &self.instance_id,
            version: &self.version,
            tag: self.tag.as_deref(),
            hash: self.hash.as_deref(),
        }
    }

    /// Parse TRN from URL
    pub fn from_url(url: &str) -> TrnResult<Self> {
        crate::url::url_to_trn(url)
    }

    /// Get the tool type (for tool resources)
    pub fn tool_type(&self) -> Option<ToolType> {
        if self.resource_type == "tool" {
            ToolType::from_str(&self.type_).ok()
        } else {
            None
        }
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
        // Build TRN string with fixed structure - all 8 components always present
        let scope = self.scope.as_deref().unwrap_or("");
        let subtype = self.subtype.as_deref().unwrap_or("");
        let tag = self.tag.as_deref().unwrap_or("");
        
        let mut result = format!(
            "trn:{}:{}:{}:{}:{}:{}:{}:{}",
            self.platform,
            scope,
            self.resource_type,
            self.type_,
            subtype,
            self.instance_id,
            self.version,
            tag
        );
        
        if let Some(hash) = &self.hash {
            result.push_str(&format!("@{}", hash));
        }
        
        write!(f, "{}", result)
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