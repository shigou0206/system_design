//! Core types for the JSON-RPC framework
//!
//! This module defines the fundamental data structures used throughout
//! the framework, including JSON-RPC message types, service contexts,
//! and streaming message handling.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use async_trait::async_trait;

use crate::core::error::JsonRpcError;
use crate::{Result, Error};

/// Type alias for message IDs
pub type MessageId = serde_json::Value;

/// JSON-RPC request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Method name to call
    pub method: String,
    /// Method parameters (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Request ID (for tracking responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: crate::JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
            id: Some(serde_json::Value::String(Uuid::new_v4().to_string())),
        }
    }
    
    /// Create a new JSON-RPC request with specific ID
    pub fn with_id(method: impl Into<String>, params: Option<serde_json::Value>, id: MessageId) -> Self {
        Self {
            jsonrpc: crate::JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
            id: Some(id),
        }
    }
    
    /// Create a notification (request without ID)
    pub fn notification(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: crate::JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
            id: None,
        }
    }
    
    /// Check if this is a notification
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
    
    /// Get the method name
    pub fn method(&self) -> &str {
        &self.method
    }
    
    /// Get the JSON-RPC version
    pub fn jsonrpc(&self) -> &str {
        &self.jsonrpc
    }
    
    /// Get the request ID
    pub fn id(&self) -> Option<&MessageId> {
        self.id.as_ref()
    }
}

/// JSON-RPC response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request ID this response corresponds to
    pub id: MessageId,
    /// Success result (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error details (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: MessageId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: crate::JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }
    
    /// Create an error response
    pub fn error(id: MessageId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: crate::JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
    
    /// Check if this response represents success
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
    
    /// Check if this response represents an error
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// Stream message with sequence number validation
/// 
/// This type ensures message ordering in streaming operations
/// through monotonically increasing sequence numbers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamMessage {
    /// Unique message ID
    pub message_id: MessageId,
    /// Monotonically increasing sequence number
    pub sequence_number: u64,
    /// The actual response data
    pub response: JsonRpcResponse,
    /// Message timestamp
    pub timestamp: SystemTime,
    /// Stream-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl StreamMessage {
    /// Create a new stream message
    pub fn new(response: JsonRpcResponse, sequence_number: u64) -> Self {
        Self {
            message_id: serde_json::Value::String(Uuid::new_v4().to_string()),
            sequence_number,
            response,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create stream message with metadata
    pub fn with_metadata(
        response: JsonRpcResponse, 
        sequence_number: u64,
        metadata: HashMap<String, serde_json::Value>
    ) -> Self {
        Self {
            message_id: serde_json::Value::String(Uuid::new_v4().to_string()),
            sequence_number,
            response,
            timestamp: SystemTime::now(),
            metadata,
        }
    }
    
    /// Validate sequence number ordering
    /// 
    /// Returns true if this message's sequence number is greater than
    /// the provided last sequence number (maintains monotonic ordering).
    pub fn validate_sequence(&self, last_sequence: u64) -> bool {
        self.sequence_number > last_sequence
    }
    
    /// Get the sequence gap from the expected next sequence
    pub fn sequence_gap(&self, expected_sequence: u64) -> i64 {
        self.sequence_number as i64 - expected_sequence as i64
    }
    
    /// Check if this message is in order
    pub fn is_in_order(&self, expected_sequence: u64) -> bool {
        self.sequence_number == expected_sequence
    }
}

/// Stream sequence validator for maintaining message order
#[derive(Debug)]
pub struct SequenceValidator {
    last_sequence: AtomicU64,
    allow_gaps: bool,
}

impl SequenceValidator {
    /// Create a new sequence validator
    pub fn new(allow_gaps: bool) -> Self {
        Self {
            last_sequence: AtomicU64::new(0),
            allow_gaps,
        }
    }
    
    /// Validate a stream message sequence
    pub fn validate(&self, message: &StreamMessage) -> Result<()> {
        let last = self.last_sequence.load(Ordering::SeqCst);
        
        if message.sequence_number <= last {
            return Err(Error::Validation {
                message: format!("Out of order sequence: expected {}, received {}", 
                    last + 1, message.sequence_number),
                source: None,
            });
        }
        
        if !self.allow_gaps && message.sequence_number != last + 1 {
            return Err(Error::Validation {
                message: format!("Sequence gap detected: expected {}, received {} (gap size: {})", 
                    last + 1, message.sequence_number, message.sequence_number - last - 1),
                source: None,
            });
        }
        
        self.last_sequence.store(message.sequence_number, Ordering::SeqCst);
        Ok(())
    }
    
    /// Get the last validated sequence number
    pub fn last_sequence(&self) -> u64 {
        self.last_sequence.load(Ordering::SeqCst)
    }
    
    /// Reset the validator
    pub fn reset(&self) {
        self.last_sequence.store(0, Ordering::SeqCst);
    }
}

/// Sequence validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum SequenceError {
    /// Message received out of order
    OutOfOrder { expected: u64, received: u64 },
    /// Gap detected in sequence
    Gap { expected: u64, received: u64, gap_size: u64 },
}

impl fmt::Display for SequenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SequenceError::OutOfOrder { expected, received } => {
                write!(f, "Message out of order: expected {}, received {}", expected, received)
            }
            SequenceError::Gap { expected, received, gap_size } => {
                write!(f, "Sequence gap: expected {}, received {} (gap: {})", expected, received, gap_size)
            }
        }
    }
}

impl std::error::Error for SequenceError {}

/// Response payload containing the actual data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponsePayload {
    /// The main response data
    pub data: serde_json::Value,
    /// Response type indicator
    pub response_type: ResponseType,
    /// Content encoding (e.g., "json", "binary", "text")
    pub encoding: String,
    /// Data size in bytes
    pub size_bytes: Option<usize>,
}

impl ResponsePayload {
    /// Create a new response payload
    pub fn new(data: serde_json::Value, response_type: ResponseType) -> Self {
        Self {
            data,
            response_type,
            encoding: "json".to_string(),
            size_bytes: None,
        }
    }
    
    /// Create payload with encoding
    pub fn with_encoding(data: serde_json::Value, response_type: ResponseType, encoding: String) -> Self {
        Self {
            data,
            response_type,
            encoding,
            size_bytes: None,
        }
    }
    
    /// Calculate and set the size
    pub fn with_calculated_size(mut self) -> Self {
        if let Ok(serialized) = serde_json::to_string(&self.data) {
            self.size_bytes = Some(serialized.len());
        }
        self
    }
}

/// Response metadata information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseMetaInfo {
    /// Processing duration in milliseconds
    pub processing_duration_ms: Option<u64>,
    /// Server timestamp
    pub server_timestamp: SystemTime,
    /// Response cache information
    pub cache_info: Option<CacheInfo>,
    /// Resource usage statistics
    pub resource_usage: Option<ResourceUsage>,
    /// Tracing and correlation IDs
    pub trace_id: Option<String>,
    pub correlation_id: Option<String>,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl ResponseMetaInfo {
    /// Create new metadata
    pub fn new() -> Self {
        Self {
            processing_duration_ms: None,
            server_timestamp: SystemTime::now(),
            cache_info: None,
            resource_usage: None,
            trace_id: None,
            correlation_id: None,
            custom: HashMap::new(),
        }
    }
    
    /// Set processing duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.processing_duration_ms = Some(duration.as_millis() as u64);
        self
    }
    
    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Add custom metadata
    pub fn with_custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }
}

impl Default for ResponseMetaInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache information for responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheInfo {
    /// Whether response was served from cache
    pub cache_hit: bool,
    /// Cache key used
    pub cache_key: Option<String>,
    /// Time to live in seconds
    pub ttl_seconds: Option<u64>,
    /// Cache generation timestamp
    pub cached_at: Option<SystemTime>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUsage {
    /// CPU time used in milliseconds
    pub cpu_time_ms: Option<u64>,
    /// Memory allocated in bytes
    pub memory_bytes: Option<u64>,
    /// I/O operations count
    pub io_operations: Option<u64>,
    /// Network bytes transferred
    pub network_bytes: Option<u64>,
}

/// Enhanced service response with separated payload and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceResponse {
    /// The actual response payload
    pub payload: ResponsePayload,
    /// Response metadata and diagnostic information
    pub meta_info: ResponseMetaInfo,
}

impl ServiceResponse {
    /// Create a new service response
    pub fn new(payload: ResponsePayload, meta_info: ResponseMetaInfo) -> Self {
        Self { payload, meta_info }
    }
    
    /// Create a simple success response
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            payload: ResponsePayload::new(data, ResponseType::Success),
            meta_info: ResponseMetaInfo::new(),
        }
    }
    
    /// Create an error response
    pub fn error(error_data: serde_json::Value) -> Self {
        Self {
            payload: ResponsePayload::new(error_data, ResponseType::Error),
            meta_info: ResponseMetaInfo::new(),
        }
    }
    
    /// Create a streaming response
    pub fn stream(data: serde_json::Value) -> Self {
        Self {
            payload: ResponsePayload::new(data, ResponseType::Stream),
            meta_info: ResponseMetaInfo::new(),
        }
    }
    
    /// Create an event response
    pub fn event(data: serde_json::Value) -> Self {
        Self {
            payload: ResponsePayload::new(data, ResponseType::Event),
            meta_info: ResponseMetaInfo::new(),
        }
    }
}

/// Response type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseType {
    /// Standard success response
    Success,
    /// Error response
    Error,
    /// Partial response in a stream
    Stream,
    /// Event notification
    Event,
    /// Server-sent event
    ServerSentEvent,
}

/// Client information for request context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientInfo {
    /// Client identifier
    pub client_id: Option<String>,
    /// Remote address
    pub remote_addr: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Client version
    pub version: Option<String>,
    /// Additional client metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service execution context
#[derive(Debug, Clone)]
pub struct ServiceContext {
    /// Unique request identifier
    pub request_id: String,
    /// Request received timestamp
    pub received_at: SystemTime,
    /// Client information
    pub client_info: Option<ClientInfo>,
    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// TRN context (if TRN integration is enabled)
    #[cfg(feature = "trn-integration")]
    pub trn_context: Option<TrnContext>,
    /// Authentication context
    pub auth_context: Option<AuthContext>,
}

impl ServiceContext {
    /// Create a new service context
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            received_at: SystemTime::now(),
            client_info: None,
            metadata: HashMap::new(),
            #[cfg(feature = "trn-integration")]
            trn_context: None,
            auth_context: None,
        }
    }
    
    /// Set client information
    pub fn with_client_info(mut self, client_info: ClientInfo) -> Self {
        self.client_info = Some(client_info);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
    
    /// Set TRN context
    #[cfg(feature = "trn-integration")]
    pub fn with_trn_context(mut self, trn_context: TrnContext) -> Self {
        self.trn_context = Some(trn_context);
        self
    }
    
    /// Set authentication context
    pub fn with_auth_context(mut self, auth_context: AuthContext) -> Self {
        self.auth_context = Some(auth_context);
        self
    }
}

/// Authentication context for request processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// User identifier
    pub user_id: String,
    /// Roles assigned to the user
    pub roles: Vec<String>,
    /// Permissions granted
    pub permissions: Vec<String>,
    /// Authentication method used
    pub auth_method: String,
    /// Token expiration time
    pub expires_at: Option<SystemTime>,
    /// Additional authentication metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthContext {
    /// Create a new authentication context
    pub fn new(user_id: impl Into<String>, auth_method: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            roles: Vec::new(),
            permissions: Vec::new(),
            auth_method: auth_method.into(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add a role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
    
    /// Add multiple roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles.extend(roles);
        self
    }
    
    /// Add a permission
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }
    
    /// Add multiple permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions.extend(permissions);
        self
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: SystemTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Check if the authentication is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| SystemTime::now() > exp)
    }
    
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }
}

/// TRN (Tool Resource Name) context for request processing
#[cfg(feature = "trn-integration")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrnContext {
    /// Platform identifier
    pub platform: String,
    /// Scope (namespace/tenant)
    pub scope: String,
    /// Resource type
    pub resource_type: String,
    /// Resource identifier
    pub resource_id: String,
    /// Resource version
    pub version: String,
    /// Tenant ID for multi-tenancy
    pub tenant_id: String,
    /// Namespace for isolation
    pub namespace: String,
    /// Additional TRN metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(feature = "trn-integration")]
impl TrnContext {
    /// Create a new TRN context
    pub fn new(
        platform: impl Into<String>,
        scope: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            platform: platform.into(),
            scope: scope.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            version: version.into(),
            tenant_id: String::new(),
            namespace: "default".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set tenant ID
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = tenant_id.into();
        self
    }
    
    /// Set namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }
    
    /// Convert to TRN string format
    pub fn to_trn_string(&self) -> String {
        format!(
            "trn:{}:{}:{}:{}:{}",
            self.platform, self.scope, self.resource_type, self.resource_id, self.version
        )
    }
    
    /// Parse from TRN string
    pub fn from_trn_string(trn: &str) -> Result<Self, crate::core::error::Error> {
        let parts: Vec<&str> = trn.split(':').collect();
        if parts.len() != 6 || parts[0] != "trn" {
            return Err(crate::core::error::Error::validation(
                format!("Invalid TRN format: {}", trn)
            ));
        }
        
        Ok(Self::new(
            parts[1].to_string(),
            parts[2].to_string(),
            parts[3].to_string(),
            parts[4].to_string(),
            parts[5].to_string(),
        ))
    }
}

/// Message metadata trait for extensible message types
pub trait MessageMetadata: Send + Sync + 'static {
    /// Get message timestamp
    fn timestamp(&self) -> SystemTime;
    
    /// Get message source
    fn source(&self) -> Option<&str>;
    
    /// Get custom metadata
    fn custom_data(&self) -> &HashMap<String, serde_json::Value>;
}

/// Default implementation of MessageMetadata
#[derive(Debug, Clone)]
pub struct DefaultMessageMetadata {
    pub timestamp: SystemTime,
    pub source: Option<String>,
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl Default for DefaultMessageMetadata {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            source: None,
            custom_data: HashMap::new(),
        }
    }
}

impl MessageMetadata for DefaultMessageMetadata {
    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    
    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
    
    fn custom_data(&self) -> &HashMap<String, serde_json::Value> {
        &self.custom_data
    }
}

/// Service information for registration and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service description
    pub description: String,
    /// Available methods
    pub methods: Vec<MethodInfo>,
    /// Service health endpoint
    pub health_endpoint: Option<String>,
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Method information for service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodInfo {
    /// Method name
    pub name: String,
    /// Method description
    pub description: String,
    /// Input parameters schema (JSON Schema)
    pub params_schema: Option<serde_json::Value>,
    /// Return value schema (JSON Schema)
    pub returns_schema: Option<serde_json::Value>,
    /// Example parameters
    pub example_params: Option<serde_json::Value>,
    /// Example return value
    pub example_returns: Option<serde_json::Value>,
    /// Whether authentication is required
    pub auth_required: bool,
    /// Required permissions
    pub required_permissions: Vec<String>,
    /// Method metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::BidirectionalStream;
    
    #[cfg(feature = "trn-integration")]
    use std::collections::HashMap;
    
    #[test]
    fn test_auth_context() {
        let auth = AuthContext::new("test-user", "bearer")
            .with_permission("read")
            .with_permission("write")
            .with_role("user");
        
        assert!(auth.has_permission("read"));
        assert!(auth.has_permission("write"));
        assert!(!auth.has_permission("admin"));
        assert!(auth.has_role("user"));
        assert!(!auth.has_role("admin"));
        assert!(!auth.is_expired());
    }
    
    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_trn_context() {
        let trn = TrnContext::new("platform", "scope", "tool", "weather", "v1.0")
            .with_tenant_id("tenant-123")
            .with_namespace("production");
            
        let trn_string = trn.to_trn_string();
        assert_eq!(trn_string, "trn:platform:scope:tool:weather:v1.0");
        
        let parsed = TrnContext::from_trn_string(&trn_string).unwrap();
        assert_eq!(parsed.platform, "platform");
        assert_eq!(parsed.resource_id, "weather");
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_trn_context_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("environment".to_string(), serde_json::json!("production"));
        metadata.insert("version".to_string(), serde_json::json!("1.2.3"));
        
        let mut trn = TrnContext::new("user", "alice", "tool", "weather-api", "v1.0")
            .with_tenant_id("acme-corp")
            .with_namespace("production");
        trn.metadata = metadata;
        
        assert_eq!(trn.tenant_id, "acme-corp");
        assert_eq!(trn.namespace, "production");
        assert_eq!(trn.metadata.len(), 2);
        assert_eq!(trn.metadata.get("environment").unwrap(), &serde_json::json!("production"));
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_trn_context_string_conversion() {
        let trn = TrnContext::new("org", "openai", "model", "gpt-4", "v1.0");
        let trn_string = trn.to_trn_string();
        assert_eq!(trn_string, "trn:org:openai:model:gpt-4:v1.0");
        
        // Test roundtrip conversion
        let parsed = TrnContext::from_trn_string(&trn_string).unwrap();
        assert_eq!(parsed.platform, "org");
        assert_eq!(parsed.scope, "openai");
        assert_eq!(parsed.resource_type, "model");
        assert_eq!(parsed.resource_id, "gpt-4");
        assert_eq!(parsed.version, "v1.0");
        assert_eq!(parsed.to_trn_string(), trn_string);
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_trn_context_invalid_format() {
        // Test invalid TRN formats
        let invalid_trns = vec![
            "not-a-trn",
            "trn:only:three:parts",
            "trn:too:many:parts:here:now:extra",
            "",
            "wrong:user:alice:tool:weather:v1.0", // Wrong prefix
        ];
        
        for invalid in invalid_trns {
            let result = TrnContext::from_trn_string(invalid);
            println!("Testing invalid TRN: '{}' -> {:?}", invalid, result);
            assert!(result.is_err(), "Expected error for invalid TRN: {}", invalid);
        }
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_service_context_with_trn() {
        let trn_context = TrnContext::new("user", "alice", "tool", "calculator", "v2.0")
            .with_tenant_id("tenant-456")
            .with_namespace("development");
            
        let auth_context = AuthContext::new("alice", "bearer")
            .with_permission("calculator:use");
        
        let context = ServiceContext::new("req-123")
            .with_trn_context(trn_context.clone())
            .with_auth_context(auth_context)
            .with_metadata("priority".to_string(), serde_json::json!("high"));
        
        assert!(context.trn_context.is_some());
        let trn = context.trn_context.as_ref().unwrap();
        assert_eq!(trn.resource_id, "calculator");
        assert_eq!(trn.tenant_id, "tenant-456");
        assert_eq!(trn.namespace, "development");
        
        assert!(context.auth_context.is_some());
        assert_eq!(context.metadata.get("priority").unwrap(), &serde_json::json!("high"));
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_trn_context_builder_pattern() {
        let trn = TrnContext::new("aiplatform", "huggingface", "dataset", "common-crawl", "latest")
            .with_tenant_id("research-team")
            .with_namespace("experiment-42");
        
        assert_eq!(trn.platform, "aiplatform");
        assert_eq!(trn.scope, "huggingface");
        assert_eq!(trn.resource_type, "dataset");
        assert_eq!(trn.resource_id, "common-crawl");
        assert_eq!(trn.version, "latest");
        assert_eq!(trn.tenant_id, "research-team");
        assert_eq!(trn.namespace, "experiment-42");
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_trn_context_serialization() {
        let trn = TrnContext::new("user", "bob", "pipeline", "data-processing", "v3.1")
            .with_tenant_id("data-team")
            .with_namespace("staging");
        
        // Test JSON serialization
        let json = serde_json::to_string(&trn).unwrap();
        let deserialized: TrnContext = serde_json::from_str(&json).unwrap();
        
        assert_eq!(trn.platform, deserialized.platform);
        assert_eq!(trn.scope, deserialized.scope);
        assert_eq!(trn.resource_type, deserialized.resource_type);
        assert_eq!(trn.resource_id, deserialized.resource_id);
        assert_eq!(trn.version, deserialized.version);
        assert_eq!(trn.tenant_id, deserialized.tenant_id);
        assert_eq!(trn.namespace, deserialized.namespace);
    }

    #[cfg(feature = "trn-integration")]
    #[test]
    fn test_multiple_trn_contexts() {
        // Test handling multiple TRN contexts for different resources
        let tool_trn = TrnContext::new("user", "alice", "tool", "weather-api", "v1.0");
        let model_trn = TrnContext::new("org", "openai", "model", "gpt-4", "v1.0");
        let dataset_trn = TrnContext::new("aiplatform", "kaggle", "dataset", "housing-prices", "v2.1");
        
        let contexts = vec![tool_trn, model_trn, dataset_trn];
        
        // Verify each context maintains its identity
        assert_eq!(contexts[0].resource_type, "tool");
        assert_eq!(contexts[1].resource_type, "model");
        assert_eq!(contexts[2].resource_type, "dataset");
        
        // Verify TRN string generation
        let trn_strings: Vec<String> = contexts.iter()
            .map(|c| c.to_trn_string())
            .collect();
        
        assert_eq!(trn_strings[0], "trn:user:alice:tool:weather-api:v1.0");
        assert_eq!(trn_strings[1], "trn:org:openai:model:gpt-4:v1.0");
        assert_eq!(trn_strings[2], "trn:aiplatform:kaggle:dataset:housing-prices:v2.1");
    }

    #[test]
    fn test_stream_message_sequence_validation() {
        let response = JsonRpcResponse::success(
            serde_json::json!(1),
            serde_json::json!({"data": "test"})
        );
        
        let msg1 = StreamMessage::new(response.clone(), 1);
        let msg2 = StreamMessage::new(response.clone(), 2);
        let msg3 = StreamMessage::new(response, 3);
        
        // Test sequence validation
        assert!(msg1.validate_sequence(0));
        assert!(msg2.validate_sequence(1));
        assert!(msg3.validate_sequence(2));
        
        // Test out of order
        assert!(!msg1.validate_sequence(1));
        assert!(!msg2.validate_sequence(2));
    }

    #[test]
    fn test_sequence_validator() {
        let validator = SequenceValidator::new(false); // No gaps allowed
        let response = JsonRpcResponse::success(
            serde_json::json!(1),
            serde_json::json!({"data": "test"})
        );
        
        let msg1 = StreamMessage::new(response.clone(), 1);
        let msg2 = StreamMessage::new(response.clone(), 2);
        let msg4 = StreamMessage::new(response, 4); // Gap!
        
        assert!(validator.validate(&msg1).is_ok());
        assert!(validator.validate(&msg2).is_ok());
        assert!(validator.validate(&msg4).is_err()); // Should fail due to gap
        
        assert_eq!(validator.last_sequence(), 2);
    }

    #[test]
    fn test_service_response_structure() {
        let payload = ResponsePayload::new(
            serde_json::json!({"result": "success"}),
            ResponseType::Success
        ).with_calculated_size();
        
        let meta_info = ResponseMetaInfo::new()
            .with_duration(Duration::from_millis(150))
            .with_trace_id("trace-123".to_string())
            .with_custom("server_id".to_string(), serde_json::json!("srv-001"));
        
        let response = ServiceResponse::new(payload, meta_info);
        
        assert_eq!(response.payload.response_type, ResponseType::Success);
        assert_eq!(response.meta_info.processing_duration_ms, Some(150));
        assert_eq!(response.meta_info.trace_id, Some("trace-123".to_string()));
        assert!(response.payload.size_bytes.is_some());
    }
    
    #[tokio::test]
    async fn test_channel_bidirectional_stream() {
        let (mut stream, mut peer) = ChannelBidirectionalStream::new();
        
        // Test that streams are initially open
        assert!(stream.is_open());
        assert!(peer.is_open());
        
        // Test sending a request
        let request = JsonRpcRequest::new("test.method", Some(serde_json::json!({"param": "value"})));
        stream.send(request.clone()).await.unwrap();
        
        // Test receiving the request on the peer side
        let received_request = peer.receive_request().await.unwrap();
        assert_eq!(received_request.method, "test.method");
        
        // Test sending a response from peer
        let response = JsonRpcResponse::success(
            serde_json::json!(1),
            serde_json::json!({"result": "success"})
        );
        peer.send_response(response.clone()).await.unwrap();
        
        // Test receiving the response on the stream side
        let received_response = stream.receive().await.unwrap();
        assert_eq!(received_response.result, Some(serde_json::json!({"result": "success"})));
        
        // Test metadata
        let stream_with_meta = stream.with_metadata("session_id", serde_json::json!("abc123"));
        assert_eq!(stream_with_meta.get_metadata("session_id"), Some(&serde_json::json!("abc123")));
        
        // Test closing
        peer.close().await.unwrap();
        assert!(!peer.is_open());
    }
    
    #[tokio::test]
    async fn test_bidirectional_stream_error_handling() {
        let (mut stream, mut peer) = ChannelBidirectionalStream::new();
        
        // Close the stream first
        stream.close().await.unwrap();
        
        // Attempt to send on closed stream should fail
        let request = JsonRpcRequest::new("test", None);
        let result = stream.send(request).await;
        assert!(result.is_err());
        
        // Attempt to receive on closed stream should fail
        let result = stream.receive().await;
        assert!(result.is_err());
        
        // Close peer
        peer.close().await.unwrap();
        
        // Attempt operations on closed peer should fail
        let response = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({}));
        let result = peer.send_response(response).await;
        assert!(result.is_err());
        
        let result = peer.receive_request().await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_bidirectional_stream_concurrent_operations() {
        let (mut stream, mut peer) = ChannelBidirectionalStream::new();
        
        // Spawn a task to handle peer side
        let peer_handle = tokio::spawn(async move {
            // Receive a request
            let request = peer.receive_request().await.unwrap();
            assert_eq!(request.method, "echo");
            
            // Send back a response
            let response = JsonRpcResponse::success(
                request.id.unwrap_or(serde_json::json!(null)),
                request.params.unwrap_or(serde_json::json!(null))
            );
            peer.send_response(response).await.unwrap();
        });
        
        // Send a request from stream side
        let request = JsonRpcRequest::with_id(
            "echo", 
            Some(serde_json::json!({"message": "hello"})), 
            serde_json::json!(42)
        );
        stream.send(request).await.unwrap();
        
        // Receive the echo response
        let response = stream.receive().await.unwrap();
        assert_eq!(response.result, Some(serde_json::json!({"message": "hello"})));
        assert_eq!(response.id, serde_json::json!(42));
        
        // Wait for peer task to complete
        peer_handle.await.unwrap();
    }
} 

/// Channel-based bidirectional stream implementation
/// 
/// This provides a concrete implementation of BidirectionalStream
/// using channels for communication between peers.
#[derive(Debug)]
pub struct ChannelBidirectionalStream {
    /// Channel for sending requests
    pub request_sender: tokio::sync::mpsc::UnboundedSender<JsonRpcRequest>,
    /// Channel for receiving responses  
    pub response_receiver: tokio::sync::mpsc::UnboundedReceiver<JsonRpcResponse>,
    /// Stream state
    pub is_open: bool,
    /// Stream metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ChannelBidirectionalStream {
    /// Create a new channel-based bidirectional stream
    pub fn new() -> (Self, ChannelBidirectionalStreamPeer) {
        let (req_tx, req_rx) = tokio::sync::mpsc::unbounded_channel();
        let (resp_tx, resp_rx) = tokio::sync::mpsc::unbounded_channel();
        
        let stream = Self {
            request_sender: req_tx,
            response_receiver: resp_rx,
            is_open: true,
            metadata: HashMap::new(),
        };
        
        let peer = ChannelBidirectionalStreamPeer {
            request_receiver: req_rx,
            response_sender: resp_tx,
            is_open: true,
            metadata: HashMap::new(),
        };
        
        (stream, peer)
    }
    
    /// Add metadata to the stream
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

#[async_trait]
impl crate::core::traits::BidirectionalStream for ChannelBidirectionalStream {
    async fn send(&mut self, message: JsonRpcRequest) -> Result<()> {
        if !self.is_open {
            return Err(Error::Transport {
                message: "Stream is closed".to_string(),
                source: None,
            });
        }
        
        self.request_sender
            .send(message)
            .map_err(|_| Error::Transport {
                message: "Failed to send message - channel closed".to_string(),
                source: None,
            })
    }
    
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        if !self.is_open {
            return Err(Error::Transport {
                message: "Stream is closed".to_string(),
                source: None,
            });
        }
        
        self.response_receiver
            .recv()
            .await
            .ok_or_else(|| Error::Transport {
                message: "No response received - channel closed".to_string(),
                source: None,
            })
    }
    
    async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        // Note: channels will be closed when dropped
        Ok(())
    }
    
    fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Peer side of the channel-based bidirectional stream
#[derive(Debug)]
pub struct ChannelBidirectionalStreamPeer {
    /// Channel for receiving requests
    pub request_receiver: tokio::sync::mpsc::UnboundedReceiver<JsonRpcRequest>,
    /// Channel for sending responses
    pub response_sender: tokio::sync::mpsc::UnboundedSender<JsonRpcResponse>,
    /// Stream state
    pub is_open: bool,
    /// Stream metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ChannelBidirectionalStreamPeer {
    /// Receive a request from the peer
    pub async fn receive_request(&mut self) -> Result<JsonRpcRequest> {
        if !self.is_open {
            return Err(Error::Transport {
                message: "Stream is closed".to_string(),
                source: None,
            });
        }
        
        self.request_receiver
            .recv()
            .await
            .ok_or_else(|| Error::Transport {
                message: "No request received - channel closed".to_string(),
                source: None,
            })
    }
    
    /// Send a response to the peer
    pub async fn send_response(&mut self, response: JsonRpcResponse) -> Result<()> {
        if !self.is_open {
            return Err(Error::Transport {
                message: "Stream is closed".to_string(),
                source: None,
            });
        }
        
        self.response_sender
            .send(response)
            .map_err(|_| Error::Transport {
                message: "Failed to send response - channel closed".to_string(),
                source: None,
            })
    }
    
    /// Close the peer side
    pub async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        Ok(())
    }
    
    /// Check if the peer is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Add metadata to the peer
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
    
    /// Get metadata value from the peer
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
} 