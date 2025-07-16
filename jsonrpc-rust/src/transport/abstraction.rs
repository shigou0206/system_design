//! Transport abstraction layer
//! 
//! This module provides higher-level abstractions built on top of the core
//! Transport and Connection traits, adding features like connection pooling,
//! message codecs, and configuration management.

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::error::{Error, Result};
use crate::core::traits::{Transport, Connection, Message};
use crate::core::types::{JsonRpcRequest, JsonRpcResponse, MessageId};

/// Enhanced transport layer with connection management and message codecs
/// 
/// This trait extends the basic Transport trait with higher-level features
/// like connection pooling, message framing, and automatic reconnection.
#[async_trait]
pub trait TransportLayer: Send + Sync {
    /// The connection type used by this transport
    type Connection: Connection;
    
    /// Configuration type for this transport
    type Config: TransportConfig;
    
    /// Create a new transport with the given configuration
    async fn new(config: Self::Config) -> Result<Self>
    where
        Self: Sized;
    
    /// Get or create a connection to the specified address
    async fn get_connection(&mut self, address: &str) -> Result<Arc<RwLock<Self::Connection>>>;
    
    /// Send a JSON-RPC message through the transport
    async fn send_message(&mut self, message: JsonRpcMessage, address: &str) -> Result<()>;
    
    /// Receive a JSON-RPC message from the transport
    async fn receive_message(&mut self) -> Result<JsonRpcMessage>;
    
    /// Close all connections and cleanup resources
    async fn shutdown(&mut self) -> Result<()>;
    
    /// Get transport statistics
    fn stats(&self) -> TransportStats;
    
    /// Get active connections count
    fn connection_count(&self) -> usize;
}

/// Connection manager for handling multiple transport connections
#[async_trait]
pub trait ConnectionManager: Send + Sync {
    /// The connection type managed by this manager
    type Connection: Connection;
    
    /// Add a new connection to the pool
    async fn add_connection(&mut self, id: String, connection: Self::Connection) -> Result<()>;
    
    /// Remove a connection from the pool
    async fn remove_connection(&mut self, id: &str) -> Result<Option<Self::Connection>>;
    
    /// Get a connection by ID
    async fn get_connection(&self, id: &str) -> Option<Arc<RwLock<Self::Connection>>>;
    
    /// Get all active connection IDs
    async fn list_connections(&self) -> Vec<String>;
    
    /// Close all connections
    async fn close_all(&mut self) -> Result<()>;
    
    /// Get connection statistics
    fn connection_stats(&self) -> HashMap<String, ConnectionInfo>;
}

/// Message codec for encoding/decoding JSON-RPC messages
pub trait MessageCodec: Send + Sync {
    /// Encode a JSON-RPC message to bytes
    fn encode(&self, message: &JsonRpcMessage) -> Result<Vec<u8>>;
    
    /// Decode bytes to a JSON-RPC message
    fn decode(&self, data: &[u8]) -> Result<JsonRpcMessage>;
    
    /// Get the framing used by this codec
    fn framing(&self) -> FramingType;
    
    /// Check if the codec supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Transport configuration trait
pub trait TransportConfig: Send + Sync + Clone + fmt::Debug {
    /// Validate the configuration
    fn validate(&self) -> Result<()>;
    
    /// Get the timeout settings
    fn timeouts(&self) -> TimeoutConfig;
    
    /// Get the retry settings
    fn retry_config(&self) -> RetryConfig;
    
    /// Get connection limits
    fn connection_limits(&self) -> ConnectionLimits;
}

/// Unified JSON-RPC message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum JsonRpcMessage {
    /// Standard request message
    Request(JsonRpcRequest),
    /// Response message
    Response(JsonRpcResponse),
    /// Notification (no response expected)
    Notification {
        jsonrpc: String,
        method: String,
        params: Option<serde_json::Value>,
    },
    /// Batch of messages
    Batch(Vec<JsonRpcMessage>),
    /// Stream data
    Stream {
        id: MessageId,
        sequence: u64,
        data: serde_json::Value,
        metadata: HashMap<String, serde_json::Value>,
    },
}

impl JsonRpcMessage {
    /// Create a new request message
    pub fn request(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self::Request(JsonRpcRequest::new(method, params))
    }
    
    /// Create a new response message
    pub fn response(id: MessageId, result: serde_json::Value) -> Self {
        Self::Response(JsonRpcResponse::success(id, result))
    }
    
    /// Create a new notification message
    pub fn notification(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self::Notification {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
    
    /// Create a new batch message
    pub fn batch(messages: Vec<JsonRpcMessage>) -> Self {
        Self::Batch(messages)
    }
    
    /// Create a new stream message
    pub fn stream(id: MessageId, sequence: u64, data: serde_json::Value) -> Self {
        Self::Stream {
            id,
            sequence,
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if this is a request that expects a response
    pub fn expects_response(&self) -> bool {
        match self {
            Self::Request(req) => !req.is_notification(),
            Self::Batch(messages) => messages.iter().any(|m| m.expects_response()),
            _ => false,
        }
    }
    
    /// Get the message ID if present
    pub fn id(&self) -> Option<&MessageId> {
        match self {
            Self::Request(req) => req.id.as_ref(),
            Self::Response(resp) => Some(&resp.id),
            Self::Stream { id, .. } => Some(id),
            _ => None,
        }
    }
    
    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::Serialization { 
                message: format!("Failed to serialize message: {}", e),
                source: Some(Box::new(e)),
            })
    }
    
    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| Error::Serialization { 
                message: format!("Failed to deserialize message: {}", e),
                source: Some(Box::new(e)),
            })
    }
}

impl Message for JsonRpcMessage {
    fn jsonrpc(&self) -> &str {
        match self {
            Self::Request(req) => &req.jsonrpc,
            Self::Response(resp) => &resp.jsonrpc,
            Self::Notification { jsonrpc, .. } => jsonrpc,
            _ => "2.0",
        }
    }
    
    fn method(&self) -> &str {
        match self {
            Self::Request(req) => &req.method,
            Self::Notification { method, .. } => method,
            _ => "",
        }
    }
    
    fn is_notification(&self) -> bool {
        matches!(self, Self::Notification { .. })
    }
    
    fn id(&self) -> Option<&serde_json::Value> {
        self.id()
    }
    
    fn to_json(&self) -> Result<String> {
        self.to_json()
    }
}

/// Message framing types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FramingType {
    /// Length-prefixed framing
    LengthPrefixed,
    /// Newline-delimited framing
    LineDelimited,
    /// WebSocket frames
    WebSocketFrames,
    /// HTTP request/response
    Http,
    /// No framing (single message)
    None,
}

/// Transport statistics
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Active connections
    pub active_connections: usize,
    /// Connection errors
    pub connection_errors: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Connection ID
    pub id: String,
    /// Remote address
    pub remote_addr: Option<SocketAddr>,
    /// Local address
    pub local_addr: Option<SocketAddr>,
    /// Connection state
    pub state: ConnectionState,
    /// Connection established timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Messages sent on this connection
    pub messages_sent: u64,
    /// Messages received on this connection
    pub messages_received: u64,
}

/// Connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is active and ready
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Connection is closed
    Disconnected,
    /// Connection encountered an error
    Error(String),
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Read timeout
    pub read_timeout: Duration,
    /// Write timeout
    pub write_timeout: Duration,
    /// Idle timeout before closing connection
    pub idle_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// Connection limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection queue size
    pub queue_size: usize,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            max_message_size: 1024 * 1024, // 1MB
            queue_size: 1000,
        }
    }
}

/// Transport-specific error types
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },
    
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    
    #[error("Protocol error: {details}")]
    ProtocolError { details: String },
    
    #[error("Timeout occurred: {operation}")]
    Timeout { operation: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

/// Default message codec implementation
#[derive(Debug)]
pub struct DefaultMessageCodec {
    framing: FramingType,
}

impl DefaultMessageCodec {
    /// Create a new default codec with the specified framing
    pub fn new(framing: FramingType) -> Self {
        Self { framing }
    }
}

impl MessageCodec for DefaultMessageCodec {
    fn encode(&self, message: &JsonRpcMessage) -> Result<Vec<u8>> {
        let json = message.to_json()?;
        let bytes = json.as_bytes();
        
        match self.framing {
            FramingType::LengthPrefixed => {
                let len = bytes.len() as u32;
                let mut result = len.to_be_bytes().to_vec();
                result.extend_from_slice(bytes);
                Ok(result)
            }
            FramingType::LineDelimited => {
                let mut result = bytes.to_vec();
                result.push(b'\n');
                Ok(result)
            }
            _ => Ok(bytes.to_vec()),
        }
    }
    
    fn decode(&self, data: &[u8]) -> Result<JsonRpcMessage> {
        let json_data = match self.framing {
            FramingType::LengthPrefixed => {
                if data.len() < 4 {
                    return Err(Error::Transport { 
                        message: "Insufficient data for length prefix".to_string(),
                        source: None,
                    });
                }
                &data[4..]
            }
            FramingType::LineDelimited => {
                // Remove trailing newline if present
                if data.ends_with(b"\n") {
                    &data[..data.len() - 1]
                } else {
                    data
                }
            }
            _ => data,
        };
        
        let json_str = std::str::from_utf8(json_data)
            .map_err(|e| Error::Transport { 
                message: format!("Invalid UTF-8: {}", e),
                source: Some(Box::new(e)),
            })?;
        
        JsonRpcMessage::from_json(json_str)
    }
    
    fn framing(&self) -> FramingType {
        self.framing.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_json_rpc_message_creation() {
        let request = JsonRpcMessage::request("test_method", Some(json!({"param": "value"})));
        assert!(matches!(request, JsonRpcMessage::Request(_)));
        assert!(request.expects_response());
        
        let notification = JsonRpcMessage::notification("notify", None);
        assert!(matches!(notification, JsonRpcMessage::Notification { .. }));
        assert!(!notification.expects_response());
    }
    
    #[test]
    fn test_message_serialization() {
        let request = JsonRpcMessage::request("test", Some(json!({"a": 1})));
        let json = request.to_json().unwrap();
        let deserialized = JsonRpcMessage::from_json(&json).unwrap();
        assert_eq!(request, deserialized);
    }
    
    #[test]
    fn test_default_codec() {
        let codec = DefaultMessageCodec::new(FramingType::LineDelimited);
        let message = JsonRpcMessage::notification("test", None);
        
        let encoded = codec.encode(&message).unwrap();
        assert!(encoded.ends_with(b"\n"));
        
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(message, decoded);
    }
    
    #[test]
    fn test_length_prefixed_codec() {
        let codec = DefaultMessageCodec::new(FramingType::LengthPrefixed);
        let message = JsonRpcMessage::notification("test", None);
        
        let encoded = codec.encode(&message).unwrap();
        assert!(encoded.len() >= 4); // At least 4 bytes for length prefix
        
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(message, decoded);
    }
} 