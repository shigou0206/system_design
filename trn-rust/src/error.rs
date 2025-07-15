//! Error types for TRN operations
//!
//! This module defines all error types that can occur during TRN parsing,
//! validation, and manipulation operations.

use std::fmt;

/// Result type alias for TRN operations
pub type TrnResult<T> = Result<T, TrnError>;

/// Main error type for TRN operations
#[derive(Debug, thiserror::Error)]
pub enum TrnError {
    /// TRN format is invalid
    #[error("TRN format error: {message}")]
    Format {
        /// Error message
        message: String,
        /// The invalid TRN string (if available)
        trn: Option<String>,
    },

    /// TRN validation failed
    #[error("TRN validation error: {message}")]
    Validation {
        /// Error message
        message: String,
        /// Validation rule that failed
        rule: String,
        /// The invalid TRN string (if available)
        trn: Option<String>,
    },

    /// TRN component is missing or invalid
    #[error("TRN component error in '{component}': {message}")]
    Component {
        /// Error message
        message: String,
        /// Component name that caused the error
        component: String,
        /// The invalid TRN string (if available)
        trn: Option<String>,
    },

    /// TRN length exceeds limits
    #[error("TRN length error: {message} ({length}/{max_length} characters)")]
    Length {
        /// Error message
        message: String,
        /// Actual length
        length: usize,
        /// Maximum allowed length
        max_length: usize,
        /// The invalid TRN string (if available)
        trn: Option<String>,
    },

    /// TRN contains invalid characters
    #[error("TRN character error: {message} ('{invalid_char}' at position {position})")]
    Character {
        /// Error message
        message: String,
        /// Invalid character
        invalid_char: char,
        /// Position of invalid character
        position: usize,
        /// The invalid TRN string (if available)
        trn: Option<String>,
    },

    /// TRN uses reserved words
    #[error("TRN reserved word error: '{reserved_word}' is reserved in component '{component}'")]
    ReservedWord {
        /// The reserved word that was used
        reserved_word: String,
        /// Component where the reserved word was used
        component: String,
        /// The invalid TRN string (if available)
        trn: Option<String>,
    },

    /// TRN hash verification failed
    #[error("TRN hash error: {message}")]
    Hash {
        /// Error message
        message: String,
        /// Expected hash value
        expected: Option<String>,
        /// Actual hash value
        actual: Option<String>,
        /// The TRN string (if available)
        trn: Option<String>,
    },

    /// TRN URL conversion failed
    #[error("TRN URL error: {message}")]
    Url {
        /// Error message
        message: String,
        /// The invalid URL (if available)
        url: Option<String>,
    },

    /// TRN alias resolution failed
    #[error("TRN alias error: {message}")]
    Alias {
        /// Error message
        message: String,
        /// The alias that failed to resolve
        alias: String,
        /// The TRN string (if available)
        trn: Option<String>,
    },

    /// TRN permission denied
    #[error("TRN permission error: {message}")]
    Permission {
        /// Error message
        message: String,
        /// User identifier
        user: Option<String>,
        /// Action that was denied
        action: Option<String>,
        /// The TRN string (if available)
        trn: Option<String>,
    },

    /// TRN resource not found
    #[error("TRN not found: {message}")]
    NotFound {
        /// Error message
        message: String,
        /// The TRN string (if available)
        trn: Option<String>,
    },

    /// TRN resource conflict
    #[error("TRN conflict: {message}")]
    Conflict {
        /// Error message
        message: String,
        /// The conflicting TRN string (if available)
        existing_trn: Option<String>,
        /// The TRN string that caused the conflict (if available)
        trn: Option<String>,
    },

    /// Invalid platform
    #[error("Invalid platform: '{platform}' is not supported")]
    InvalidPlatform {
        /// The invalid platform name
        platform: String,
    },

    /// Invalid resource type
    #[error("Invalid resource type: '{resource_type}' is not supported")]
    InvalidResourceType {
        /// The invalid resource type
        resource_type: String,
    },

    /// Invalid tool type
    #[error("Invalid tool type: '{tool_type}' is not supported")]
    InvalidToolType {
        /// The invalid tool type
        tool_type: String,
    },

    /// Pattern matching error
    #[error("Pattern matching error: {message}")]
    Pattern {
        /// Error message
        message: String,
        /// The invalid pattern
        pattern: String,
    },

    /// Version comparison error
    #[error("Version comparison error: {message}")]
    Version {
        /// Error message
        message: String,
        /// First version string
        version1: String,
        /// Second version string
        version2: String,
        /// Comparison operator
        operator: String,
    },

    /// Builder error - missing required field
    #[error("Builder error: missing required field '{field}'")]
    BuilderMissingField {
        /// Name of the missing field
        field: String,
    },

    /// Builder error - invalid field value
    #[error("Builder error: invalid value for field '{field}': {message}")]
    BuilderInvalidField {
        /// Name of the field
        field: String,
        /// Error message
        message: String,
    },

    /// Internal error (should not happen in normal usage)
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
    },
}

impl TrnError {
    /// Create a format error
    pub fn format<S: Into<String>>(message: S, trn: Option<String>) -> Self {
        Self::Format {
            message: message.into(),
            trn,
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S, rule: S, trn: Option<String>) -> Self {
        Self::Validation {
            message: message.into(),
            rule: rule.into(),
            trn,
        }
    }

    /// Create a component error
    pub fn component<S: Into<String>>(message: S, component: S, trn: Option<String>) -> Self {
        Self::Component {
            message: message.into(),
            component: component.into(),
            trn,
        }
    }

    /// Create a length error
    pub fn length<S: Into<String>>(
        message: S,
        length: usize,
        max_length: usize,
        trn: Option<String>,
    ) -> Self {
        Self::Length {
            message: message.into(),
            length,
            max_length,
            trn,
        }
    }

    /// Create a character error
    pub fn character<S: Into<String>>(
        message: S,
        invalid_char: char,
        position: usize,
        trn: Option<String>,
    ) -> Self {
        Self::Character {
            message: message.into(),
            invalid_char,
            position,
            trn,
        }
    }

    /// Create a reserved word error
    pub fn reserved_word<S: Into<String>>(
        reserved_word: S,
        component: S,
        trn: Option<String>,
    ) -> Self {
        Self::ReservedWord {
            reserved_word: reserved_word.into(),
            component: component.into(),
            trn,
        }
    }

    /// Create a hash error
    pub fn hash<S: Into<String>>(
        message: S,
        expected: Option<String>,
        actual: Option<String>,
        trn: Option<String>,
    ) -> Self {
        Self::Hash {
            message: message.into(),
            expected,
            actual,
            trn,
        }
    }

    /// Create a URL error
    pub fn url<S: Into<String>>(message: S, url: Option<String>) -> Self {
        Self::Url {
            message: message.into(),
            url,
        }
    }

    /// Create a pattern error
    pub fn pattern<S: Into<String>>(message: S, pattern: S) -> Self {
        Self::Pattern {
            message: message.into(),
            pattern: pattern.into(),
        }
    }

    /// Create a version error
    pub fn version<S: Into<String>>(message: S, version1: S, version2: S, operator: S) -> Self {
        Self::Version {
            message: message.into(),
            version1: version1.into(),
            version2: version2.into(),
            operator: operator.into(),
        }
    }

    /// Create a builder missing field error
    pub fn builder_missing_field<S: Into<String>>(field: S) -> Self {
        Self::BuilderMissingField {
            field: field.into(),
        }
    }

    /// Create a builder invalid field error
    pub fn builder_invalid_field<S: Into<String>>(field: S, message: S) -> Self {
        Self::BuilderInvalidField {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Get the TRN string associated with this error (if any)
    pub fn trn(&self) -> Option<&str> {
        match self {
            Self::Format { trn, .. }
            | Self::Validation { trn, .. }
            | Self::Component { trn, .. }
            | Self::Length { trn, .. }
            | Self::Character { trn, .. }
            | Self::ReservedWord { trn, .. }
            | Self::Hash { trn, .. }
            | Self::Alias { trn, .. }
            | Self::Permission { trn, .. }
            | Self::NotFound { trn, .. }
            | Self::Conflict { trn, .. } => trn.as_deref(),
            _ => None,
        }
    }

    /// Get the error category for JSON RPC compatibility
    pub fn error_code(&self) -> i32 {
        match self {
            Self::Format { .. } => -32000,
            Self::Validation { .. } => -32001,
            Self::Component { .. } => -32001,
            Self::Length { .. } => -32002,
            Self::Character { .. } => -32002,
            Self::ReservedWord { .. } => -32002,
            Self::Hash { .. } => -32003,
            Self::Alias { .. } => -32003,
            Self::Url { .. } => -32004,
            Self::Permission { .. } => -32020,
            Self::NotFound { .. } => -32030,
            Self::Conflict { .. } => -32031,
            Self::InvalidPlatform { .. } => -32040,
            Self::InvalidResourceType { .. } => -32041,
            Self::InvalidToolType { .. } => -32042,
            Self::Pattern { .. } => -32050,
            Self::Version { .. } => -32051,
            Self::BuilderMissingField { .. } => -32060,
            Self::BuilderInvalidField { .. } => -32061,
            Self::Internal { .. } => -32099,
        }
    }

    /// Get the error code (alias for error_code)
    pub fn code(&self) -> i32 {
        self.error_code()
    }

    /// Convert to JSON RPC error response format
    pub fn to_json_rpc(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.error_code(),
            "message": self.to_string(),
            "data": {
                "type": self.error_type_name(),
                "trn": self.trn(),
                "details": self.error_details()
            }
        })
    }

    /// Get the error type name
    fn error_type_name(&self) -> &'static str {
        match self {
            Self::Format { .. } => "TrnFormatError",
            Self::Validation { .. } => "TrnValidationError",
            Self::Component { .. } => "TrnComponentError",
            Self::Length { .. } => "TrnLengthError",
            Self::Character { .. } => "TrnCharacterError",
            Self::ReservedWord { .. } => "TrnReservedWordError",
            Self::Hash { .. } => "TrnHashError",
            Self::Url { .. } => "TrnUrlError",
            Self::Alias { .. } => "TrnAliasError",
            Self::Permission { .. } => "TrnPermissionError",
            Self::NotFound { .. } => "TrnNotFoundError",
            Self::Conflict { .. } => "TrnConflictError",
            Self::InvalidPlatform { .. } => "TrnInvalidPlatformError",
            Self::InvalidResourceType { .. } => "TrnInvalidResourceTypeError",
            Self::InvalidToolType { .. } => "TrnInvalidToolTypeError",
            Self::Pattern { .. } => "TrnPatternError",
            Self::Version { .. } => "TrnVersionError",
            Self::BuilderMissingField { .. } => "TrnBuilderMissingFieldError",
            Self::BuilderInvalidField { .. } => "TrnBuilderInvalidFieldError",
            Self::Internal { .. } => "TrnInternalError",
        }
    }

    /// Get additional error details for JSON response
    fn error_details(&self) -> serde_json::Value {
        match self {
            Self::Length {
                length,
                max_length,
                ..
            } => serde_json::json!({
                "length": length,
                "max_length": max_length
            }),
            Self::Character {
                invalid_char,
                position,
                ..
            } => serde_json::json!({
                "invalid_char": invalid_char,
                "position": position
            }),
            Self::ReservedWord {
                reserved_word,
                component,
                ..
            } => serde_json::json!({
                "reserved_word": reserved_word,
                "component": component
            }),
            Self::Hash {
                expected, actual, ..
            } => serde_json::json!({
                "expected": expected,
                "actual": actual
            }),
            Self::Version {
                version1,
                version2,
                operator,
                ..
            } => serde_json::json!({
                "version1": version1,
                "version2": version2,
                "operator": operator
            }),
            _ => serde_json::Value::Null,
        }
    }
}

/// Specialized error for TRN parsing
#[derive(Debug, Clone, PartialEq)]
pub struct TrnParseError {
    /// Error message
    pub message: String,
    /// Position where parsing failed
    pub position: usize,
    /// The input that failed to parse
    pub input: String,
}

impl fmt::Display for TrnParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at position {}: {} in '{}'",
            self.position, self.message, self.input
        )
    }
}

impl std::error::Error for TrnParseError {}

impl From<TrnParseError> for TrnError {
    fn from(err: TrnParseError) -> Self {
        Self::Format {
            message: err.message,
            trn: Some(err.input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = TrnError::format("Invalid format", Some("invalid-trn".to_string()));
        assert_eq!(err.trn(), Some("invalid-trn"));
        assert_eq!(err.error_code(), -32000);
    }

    #[test]
    fn test_json_rpc_conversion() {
        let err = TrnError::length("Too long", 300, 256, Some("long-trn".to_string()));
        let json = err.to_json_rpc();
        
        assert_eq!(json["code"], -32002);
        assert_eq!(json["data"]["type"], "TrnLengthError");
        assert_eq!(json["data"]["details"]["length"], 300);
        assert_eq!(json["data"]["details"]["max_length"], 256);
    }

    #[test]
    fn test_parse_error_conversion() {
        let parse_err = TrnParseError {
            message: "Expected colon".to_string(),
            position: 5,
            input: "invalid".to_string(),
        };
        
        let trn_err: TrnError = parse_err.into();
        assert!(matches!(trn_err, TrnError::Format { .. }));
        assert_eq!(trn_err.trn(), Some("invalid"));
    }
} 