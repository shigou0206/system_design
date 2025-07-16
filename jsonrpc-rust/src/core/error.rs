//! Error handling for the JSON-RPC framework
//!
//! This module provides comprehensive error handling with automatic conversions
//! from common error types, optional source location tracking for debugging,
//! and retry logic for transient failures.

use std::fmt;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JSON-RPC error codes as defined in the specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonRpcErrorCode {
    /// Parse error - Invalid JSON was received by the server
    ParseError,
    /// Invalid Request - The JSON sent is not a valid Request object
    InvalidRequest,
    /// Method not found - The method does not exist / is not available
    MethodNotFound,
    /// Invalid params - Invalid method parameter(s)
    InvalidParams,
    /// Internal error - Internal JSON-RPC error
    InternalError,
    /// Server errors (reserved range -32099 to -32000)
    /// The i32 value must be in the range -32099 to -32000
    ServerError(i32),
}

impl JsonRpcErrorCode {
    /// Get the numeric error code
    pub fn code(&self) -> i32 {
        match self {
            JsonRpcErrorCode::ParseError => -32700,
            JsonRpcErrorCode::InvalidRequest => -32600,
            JsonRpcErrorCode::MethodNotFound => -32601,
            JsonRpcErrorCode::InvalidParams => -32602,
            JsonRpcErrorCode::InternalError => -32603,
            JsonRpcErrorCode::ServerError(code) => *code,
        }
    }
    
    /// Check if this is a valid server error code
    pub fn is_valid_server_error(code: i32) -> bool {
        (-32099..=-32000).contains(&code)
    }
    
    /// Create a server error code
    pub fn server_error(code: i32) -> std::result::Result<Self, crate::core::error::Error> {
        if Self::is_valid_server_error(code) {
            Ok(JsonRpcErrorCode::ServerError(code))
        } else {
            Err(crate::core::error::Error::configuration(
                format!("Invalid server error code: {}. Must be in range -32099 to -32000", code)
            ))
        }
    }
}

/// Source location information for debugging (conditionally compiled)
#[cfg(feature = "debug-location")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file name
    pub file: String,
    /// Line number
    pub line: u32,
    /// Function or method name
    pub function: String,
    /// Column number (optional)
    pub column: Option<u32>,
}

#[cfg(feature = "debug-location")]
impl SourceLocation {
    /// Create a new source location
    pub fn new(file: impl Into<String>, line: u32, function: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line,
            function: function.into(),
            column: None,
        }
    }
    
    /// Create source location with column
    pub fn with_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }
}

#[cfg(feature = "debug-location")]
impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(column) = self.column {
            write!(f, "{}:{}:{} in {}", self.file, self.line, column, self.function)
        } else {
            write!(f, "{}:{} in {}", self.file, self.line, self.function)
        }
    }
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: JsonRpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.code(),
            message: message.into(),
            data: None,
        }
    }
    
    /// Create error with additional data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
    
    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(JsonRpcErrorCode::ParseError, message)
    }
    
    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(JsonRpcErrorCode::InvalidRequest, message)
    }
    
    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            JsonRpcErrorCode::MethodNotFound,
            format!("Method '{}' not found", method)
        )
    }
    
    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(JsonRpcErrorCode::InvalidParams, message)
    }
    
    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(JsonRpcErrorCode::InternalError, message)
    }
    
    /// Create a server error
    pub fn server_error(code: i32, message: impl Into<String>) -> std::result::Result<Self, crate::core::error::Error> {
        let error_code = JsonRpcErrorCode::server_error(code)?;
        Ok(Self::new(error_code, message))
    }
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error {}: {}", self.code, self.message)?;
        if let Some(data) = &self.data {
            write!(f, " (data: {})", data)?;
        }
        Ok(())
    }
}

impl std::error::Error for JsonRpcError {}

/// Retry policy for handling transient failures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 to 1.0) for randomizing delays
    pub jitter_factor: f64,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_attempts: u32, initial_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay: Duration::from_secs(60), // 1 minute max
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
    
    /// Create an exponential backoff policy
    pub fn exponential_backoff(max_attempts: u32) -> Self {
        Self::new(max_attempts, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(30))
            .with_backoff_multiplier(2.0)
            .with_jitter_factor(0.2)
    }
    
    /// Create a linear backoff policy
    pub fn linear_backoff(max_attempts: u32) -> Self {
        Self::new(max_attempts, Duration::from_millis(500))
            .with_backoff_multiplier(1.0)
            .with_jitter_factor(0.1)
    }
    
    /// Set maximum delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }
    
    /// Set backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }
    
    /// Set jitter factor
    pub fn with_jitter_factor(mut self, jitter: f64) -> Self {
        self.jitter_factor = jitter.clamp(0.0, 1.0);
        self
    }
    
    /// Calculate delay for a specific attempt (0-based)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt >= self.max_attempts {
            return Duration::ZERO;
        }
        
        let base_delay = self.initial_delay.as_millis() as f64;
        let multiplied_delay = base_delay * self.backoff_multiplier.powi(attempt as i32);
        let capped_delay = multiplied_delay.min(self.max_delay.as_millis() as f64);
        
        // Add jitter
        let jitter = (rand::random::<f64>() - 0.5) * 2.0 * self.jitter_factor;
        let final_delay = capped_delay * (1.0 + jitter);
        
        Duration::from_millis(final_delay.max(0.0) as u64)
    }
    
    /// Check if we should retry for a specific attempt
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

/// Main error type for the JSON-RPC framework
#[derive(Error, Debug)]
pub enum Error {
    /// JSON-RPC protocol errors
    #[error("JSON-RPC error: {0}")]
    JsonRpc(#[from] JsonRpcError),
    
    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Transport-related errors
    #[error("Transport error: {message}")]
    Transport {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Connection errors
    #[error("Connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Authentication and authorization errors
    #[error("Authentication error: {message}")]
    Authentication {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Authorization errors
    #[error("Authorization error: {message}")]
    Authorization {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Service errors
    #[error("Service error: {message}")]
    Service {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Method not found
    #[error("Method not found: {method}")]
    MethodNotFound { method: String },
    
    /// Invalid parameters
    #[error("Invalid parameters: {message}")]
    InvalidParams {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<Duration>,
    },
    
    /// Resource not found
    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// TRN (Tool Resource Name) related errors
    #[cfg(feature = "trn-integration")]
    #[error("TRN error: {0}")]
    Trn(#[from] trn_rust::TrnError),
    
    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { 
        operation: String,
        duration: Duration,
    },
    
    /// Cancellation errors
    #[error("Operation was cancelled: {operation}")]
    Cancelled { operation: String },
    
    /// Custom errors for extensibility
    #[error("Custom error: {message}")]
    Custom {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        /// Source location for debugging (conditionally compiled)
        #[cfg(feature = "debug-location")]
        location: Option<SourceLocation>,
    },
}

/// Error kind enumeration for categorizing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Protocol-related errors
    Protocol,
    /// Serialization errors
    Serialization,
    /// Transport errors
    Transport,
    /// Connection errors  
    Connection,
    /// Authentication errors
    Authentication,
    /// Authorization errors
    Authorization,
    /// Validation errors
    Validation,
    /// Service errors
    Service,
    /// Method not found
    MethodNotFound,
    /// Invalid parameters
    InvalidParams,
    /// Rate limiting
    RateLimit,
    /// Resource not found
    ResourceNotFound,
    /// Configuration errors
    Configuration,
    /// I/O errors
    Io,
    /// TRN-related errors
    #[cfg(feature = "trn-integration")]
    Trn,
    /// Timeout errors
    Timeout,
    /// Cancellation
    Cancelled,
    /// Custom errors
    Custom,
}

/// Type alias for Result with our Error type
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Get the error kind
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::JsonRpc(_) => ErrorKind::Protocol,
            Error::Serialization { .. } => ErrorKind::Serialization,
            Error::Transport { .. } => ErrorKind::Transport,
            Error::Connection { .. } => ErrorKind::Connection,
            Error::Authentication { .. } => ErrorKind::Authentication,
            Error::Authorization { .. } => ErrorKind::Authorization,
            Error::Validation { .. } => ErrorKind::Validation,
            Error::Service { .. } => ErrorKind::Service,
            Error::MethodNotFound { .. } => ErrorKind::MethodNotFound,
            Error::InvalidParams { .. } => ErrorKind::InvalidParams,
            Error::RateLimit { .. } => ErrorKind::RateLimit,
            Error::ResourceNotFound { .. } => ErrorKind::ResourceNotFound,
            Error::Configuration { .. } => ErrorKind::Configuration,
            Error::Io(_) => ErrorKind::Io,
            #[cfg(feature = "trn-integration")]
            Error::Trn(_) => ErrorKind::Trn,
            Error::Timeout { .. } => ErrorKind::Timeout,
            Error::Cancelled { .. } => ErrorKind::Cancelled,
            Error::Custom { .. } => ErrorKind::Custom,
        }
    }
    
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Transport { .. } => true,
            Error::Connection { .. } => true,
            Error::Service { .. } => true,
            Error::RateLimit { .. } => true,
            Error::Timeout { .. } => true,
            Error::Io(io_err) => {
                matches!(
                    io_err.kind(),
                    std::io::ErrorKind::TimedOut 
                    | std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                )
            },
            #[cfg(feature = "trn-integration")]
            Error::Trn(_) => false,
            Error::JsonRpc(_) | Error::Serialization { .. } | Error::Authentication { .. }
            | Error::Authorization { .. } | Error::Validation { .. } | Error::MethodNotFound { .. }
            | Error::InvalidParams { .. } | Error::ResourceNotFound { .. } 
            | Error::Configuration { .. } | Error::Cancelled { .. } => false,
            Error::Custom { .. } => false, // Custom errors should specify their own retry logic
        }
    }
    
    /// Convert to a JSON-RPC error
    pub fn to_jsonrpc_error(&self) -> JsonRpcError {
        match self {
            Error::JsonRpc(err) => err.clone(),
            Error::MethodNotFound { method } => JsonRpcError::method_not_found(method),
            Error::InvalidParams { message, .. } => JsonRpcError::invalid_params(message),
            Error::Serialization { message, .. } => JsonRpcError::parse_error(message),
            _ => JsonRpcError::internal_error(self.to_string()),
        }
    }
    
    /// Create a transport error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create an authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a service error
    pub fn service(message: impl Into<String>) -> Self {
        Self::Service {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::MethodNotFound {
            method: method.to_string(),
        }
    }
    
    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::InvalidParams {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a rate limit error
    pub fn rate_limit(message: impl Into<String>, retry_after: Option<Duration>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
        }
    }
    
    /// Create a resource not found error
    pub fn resource_not_found(resource: impl Into<String>) -> Self {
        Self::ResourceNotFound {
            resource: resource.into(),
        }
    }
    
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            source: None,
        }
    }
    
    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration: Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration,
        }
    }
    
    /// Create a cancellation error
    pub fn cancelled(operation: impl Into<String>) -> Self {
        Self::Cancelled {
            operation: operation.into(),
        }
    }
    
    /// Create a custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
            source: None,
            #[cfg(feature = "debug-location")]
            location: None,
        }
    }
    
    /// Create a custom error with source location (debug builds)
    #[cfg(feature = "debug-location")]
    pub fn custom_with_location(
        message: impl Into<String>, 
        location: SourceLocation
    ) -> Self {
        Self::Custom {
            message: message.into(),
            source: None,
            location: Some(location),
        }
    }
    
    /// Add source location to error (debug builds)
    #[cfg(feature = "debug-location")]
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        if let Self::Custom { location: ref mut loc, .. } = self {
            *loc = Some(location);
        }
        self
    }
}

// Automatic conversions from common error types
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Self::Validation {
            message: format!("UUID error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::Validation {
            message: format!("Parse integer error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Self::Validation {
            message: format!("Parse float error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Validation {
            message: format!("UTF-8 error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Validation {
            message: format!("UTF-8 error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Convenience macro for creating errors with source location in debug builds
#[cfg(feature = "debug-location")]
#[macro_export]
macro_rules! error_here {
    ($msg:expr) => {
        $crate::core::error::Error::custom_with_location(
            $msg,
            $crate::core::error::SourceLocation::new(
                file!(),
                line!(),
                "unknown" // Rust doesn't have a macro for function names
            )
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::core::error::Error::custom_with_location(
            format!($fmt, $($arg)*),
            $crate::core::error::SourceLocation::new(
                file!(),
                line!(),
                "unknown"
            )
        )
    };
}

#[cfg(not(feature = "debug-location"))]
#[macro_export]
macro_rules! error_here {
    ($msg:expr) => {
        $crate::core::error::Error::custom($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::core::error::Error::custom(format!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_jsonrpc_error_creation() {
        let error = JsonRpcError::method_not_found("test_method");
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("test_method"));
    }

    #[test]
    fn test_error_kind() {
        let error = Error::method_not_found("test");
        assert_eq!(error.kind(), ErrorKind::MethodNotFound);
        
        let io_error = Error::from(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert_eq!(io_error.kind(), ErrorKind::Io);
    }

    #[test]
    fn test_error_to_jsonrpc() {
        let error = Error::method_not_found("test_method");
        let jsonrpc_error = error.to_jsonrpc_error();
        assert_eq!(jsonrpc_error.code, -32601);
    }

    #[test]
    fn test_server_error_range() {
        assert!(JsonRpcErrorCode::is_valid_server_error(-32001));
        assert!(!JsonRpcErrorCode::is_valid_server_error(-32700));
        assert!(!JsonRpcErrorCode::is_valid_server_error(-31999));
        
        let result = JsonRpcErrorCode::server_error(-32001);
        assert!(result.is_ok());
        
        let invalid_result = JsonRpcErrorCode::server_error(-32700);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::exponential_backoff(3);
        
        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
        
        let delay0 = policy.delay_for_attempt(0);
        let delay1 = policy.delay_for_attempt(1);
        let delay2 = policy.delay_for_attempt(2);
        
        assert!(delay0 > Duration::ZERO);
        assert!(delay1 > delay0);
        assert!(delay2 > delay1);
    }

    #[test]
    fn test_error_retryable() {
        assert!(Error::transport("connection failed").is_retryable());
        assert!(Error::timeout("operation", Duration::from_secs(30)).is_retryable());
        assert!(!Error::method_not_found("test").is_retryable());
        assert!(!Error::validation("invalid input").is_retryable());
    }

    #[test]
    fn test_automatic_conversions() {
        let json_error = serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF"));
        let our_error: Error = json_error.into();
        assert_eq!(our_error.kind(), ErrorKind::Serialization);
        
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let our_error: Error = io_error.into();
        assert_eq!(our_error.kind(), ErrorKind::Io);
    }

    #[cfg(feature = "debug-location")]
    #[test]
    fn test_source_location() {
        let location = SourceLocation::new("test.rs", 42, "test_function")
            .with_column(10);
        
        assert_eq!(location.file, "test.rs");
        assert_eq!(location.line, 42);
        assert_eq!(location.function, "test_function");
        assert_eq!(location.column, Some(10));
        
        let error = Error::custom_with_location("test error", location);
        if let Error::Custom { location: Some(loc), .. } = error {
            assert_eq!(loc.file, "test.rs");
        } else {
            panic!("Expected custom error with location");
        }
    }

    #[test]
    fn test_display_trait() {
        let jsonrpc_error = JsonRpcError::method_not_found("test_method");
        let display_string = format!("{}", jsonrpc_error);
        assert!(display_string.contains("JSON-RPC Error"));
        assert!(display_string.contains("-32601"));
        assert!(display_string.contains("test_method"));

        let jsonrpc_error_with_data = JsonRpcError::parse_error("Invalid JSON")
            .with_data(serde_json::json!({"details": "Unexpected character"}));
        let display_string = format!("{}", jsonrpc_error_with_data);
        assert!(display_string.contains("data:"));
    }
} 