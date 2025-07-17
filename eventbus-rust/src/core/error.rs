//! Error types for the event bus system

use thiserror::Error;

/// Main error type for event bus operations
#[derive(Error, Debug)]
pub enum EventBusError {
    /// Storage related errors
    #[error("Storage error: {message}")]
    Storage {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Rule engine errors
    #[error("Rule engine error: {message}")]
    RuleEngine {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Tool invocation errors
    #[error("Tool invocation error: {message}")]
    ToolInvocation {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    /// Network/Transport errors
    #[error("Transport error: {message}")]
    Transport {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Permission denied errors
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
    
    /// Resource not found errors
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    /// Resource already exists errors
    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },
    
    /// Invalid input/parameters
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    
    /// Internal system errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource}")]
    ResourceLimit { resource: String },
    
    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    /// Rate limiting errors
    #[error("Rate limited: {message}")]
    RateLimited { message: String },
}

impl EventBusError {
    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a storage error with source
    pub fn storage_with_source(
        message: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Storage {
            message: message.into(),
            source: Some(source.into()),
        }
    }
    
    /// Create a rule engine error
    pub fn rule_engine(message: impl Into<String>) -> Self {
        Self::RuleEngine {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a tool invocation error
    pub fn tool_invocation(message: impl Into<String>) -> Self {
        Self::ToolInvocation {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    
    /// Create a transport error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }
    
    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }
    
    /// Create an already exists error
    pub fn already_exists(resource: impl Into<String>) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
        }
    }
    
    /// Create an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }
    
    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }
    
    /// Create a resource limit error
    pub fn resource_limit(resource: impl Into<String>) -> Self {
        Self::ResourceLimit {
            resource: resource.into(),
        }
    }
    
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
    
    /// Create a rate limited error
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::RateLimited {
            message: message.into(),
        }
    }
    
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Storage { .. } => true,
            Self::Transport { .. } => true,
            Self::Timeout { .. } => true,
            Self::ResourceLimit { .. } => true,
            Self::Internal { .. } => true,
            _ => false,
        }
    }
    
    /// Get error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            Self::Storage { .. } => "storage",
            Self::RuleEngine { .. } => "rule_engine",
            Self::ToolInvocation { .. } => "tool_invocation",
            Self::Serialization(_) => "serialization",
            Self::Configuration { .. } => "configuration",
            Self::Transport { .. } => "transport",
            Self::PermissionDenied { .. } => "permission",
            Self::NotFound { .. } => "not_found",
            Self::AlreadyExists { .. } => "already_exists",
            Self::InvalidInput { .. } => "invalid_input",
            Self::Internal { .. } => "internal",
            Self::Timeout { .. } => "timeout",
            Self::ResourceLimit { .. } => "resource_limit",
            Self::Validation { .. } => "validation",
            Self::RateLimited { .. } => "rate_limited",
        }
    }
}

/// Convert from jsonrpc-rust errors
impl From<jsonrpc_rust::Error> for EventBusError {
    fn from(err: jsonrpc_rust::Error) -> Self {
        match err {
            jsonrpc_rust::Error::Transport { message, source } => Self::Transport { message, source },
            jsonrpc_rust::Error::InvalidParams { message, source: _ } => Self::InvalidInput { message },
            jsonrpc_rust::Error::MethodNotFound { method } => Self::NotFound { resource: format!("method: {}", method) },
            jsonrpc_rust::Error::Timeout { operation, duration: _ } => Self::Timeout { operation },
            _ => Self::Internal {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
        }
    }
}

/// Convert from TRN errors if the feature is enabled
#[cfg(feature = "trn-integration")]
impl From<trn_rust::Error> for EventBusError {
    fn from(err: trn_rust::Error) -> Self {
        Self::InvalidInput {
            message: format!("TRN error: {}", err),
        }
    }
}

/// Convert from SQLx errors if persistence is enabled
#[cfg(feature = "persistence")]
impl From<sqlx::Error> for EventBusError {
    fn from(err: sqlx::Error) -> Self {
        Self::Storage {
            message: format!("Database error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = EventBusError::storage("Test storage error");
        assert_eq!(err.category(), "storage");
        assert!(err.is_retryable());
        
        let err = EventBusError::not_found("test_resource");
        assert_eq!(err.category(), "not_found");
        assert!(!err.is_retryable());
    }
    
    #[test]
    fn test_error_display() {
        let err = EventBusError::storage("Connection failed");
        assert_eq!(err.to_string(), "Storage error: Connection failed");
        
        let err = EventBusError::not_found("rule_123");
        assert_eq!(err.to_string(), "Not found: rule_123");
    }
} 