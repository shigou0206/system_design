//! Core traits for the JSON-RPC framework
//! 
//! This module defines the fundamental abstractions that all other components
//! build upon, including message handling, transport, and service interfaces.

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;
use futures::Stream;
use serde_json::Value;
use crate::core::error::{Error, Result};
use crate::core::types::{JsonRpcRequest, JsonRpcResponse, ServiceContext};
use crate::core::future::{JsonRpcFuture, ServiceStream};

/// Core message trait for JSON-RPC operations
/// 
/// This trait provides the fundamental interface for all message types
/// in the JSON-RPC framework.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use serde_json::json;
/// 
/// // Create a JSON-RPC request
/// let request = JsonRpcRequest::new("get_weather", Some(json!({"city": "Tokyo"})));
/// assert_eq!(request.method(), "get_weather");
/// assert_eq!(request.jsonrpc(), "2.0");
/// 
/// // Check if it's a notification
/// let notification = JsonRpcRequest::notification("log_event", None);
/// assert!(notification.is_notification());
/// ```
pub trait Message: Send + Sync + fmt::Debug {
    /// Get the JSON-RPC version (should be "2.0")
    fn jsonrpc(&self) -> &str;
    
    /// Get the method name
    fn method(&self) -> &str;
    
    /// Check if this is a notification (no response expected)
    fn is_notification(&self) -> bool;
    
    /// Get the message ID if present
    fn id(&self) -> Option<&Value>;
    
    /// Serialize the message to JSON
    fn to_json(&self) -> Result<String>;
    
    /// Get message metadata
    fn metadata(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Transport abstraction for different communication protocols
/// 
/// This trait defines how messages are sent and received across different
/// transport mechanisms (TCP, WebSocket, HTTP, etc.).
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::Result;
/// use async_trait::async_trait;
/// 
/// struct MockTransport {
///     responses: Vec<String>,
/// }
/// 
/// #[async_trait]
/// impl Transport for MockTransport {
///     async fn send(&mut self, message: &str) -> Result<()> {
///         // Mock implementation
///         Ok(())
///     }
///     
///     async fn receive(&mut self) -> Result<String> {
///         self.responses.pop()
///             .ok_or_else(|| Error::Transport { message: "No more responses".to_string(), source: None })
///     }
///     
///     async fn close(&mut self) -> Result<()> {
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message through the transport
    async fn send(&mut self, message: &str) -> Result<()>;
    
    /// Receive a message from the transport
    async fn receive(&mut self) -> Result<String>;
    
    /// Close the transport connection
    async fn close(&mut self) -> Result<()>;
    
    /// Check if transport supports bidirectional communication
    fn is_bidirectional(&self) -> bool {
        true
    }
    
    /// Get transport-specific metadata
    fn metadata(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Connection abstraction for managing transport connections
/// 
/// This trait provides higher-level connection management on top of
/// the basic transport functionality.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::Result;
/// use async_trait::async_trait;
/// use std::collections::HashMap;
/// use serde_json::Value;
/// 
/// struct TcpConnection {
///     address: String,
///     closed: bool,
/// }
/// 
/// #[async_trait]
/// impl Connection for TcpConnection {
///     async fn connect(&mut self) -> Result<()> {
///         self.closed = false;
///         Ok(())
///     }
///     
///     async fn disconnect(&mut self) -> Result<()> {
///         self.closed = true;
///         Ok(())
///     }
///     
///     fn is_connected(&self) -> bool {
///         !self.closed
///     }
///     
///     fn is_closed(&self) -> bool {
///         self.closed
///     }
///     
///     fn connection_info(&self) -> HashMap<String, Value> {
///         let mut info = HashMap::new();
///         info.insert("address".to_string(), self.address.clone().into());
///         info.insert("protocol".to_string(), "tcp".into());
///         info
///     }
/// }
/// ```
#[async_trait]
pub trait Connection: Send + Sync {
    /// Establish the connection
    async fn connect(&mut self) -> Result<()>;
    
    /// Close the connection
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check if the connection is established
    fn is_connected(&self) -> bool;
    
    /// Check if the connection is closed
    /// 
    /// This method provides a clear way to determine connection state,
    /// improving safety in connection management.
    fn is_closed(&self) -> bool;
    
    /// Get connection information and metadata
    fn connection_info(&self) -> HashMap<String, Value>;
    
    /// Get the last error, if any
    fn last_error(&self) -> Option<&Error> {
        None
    }
    
    /// Test connection health
    async fn ping(&mut self) -> Result<()> {
        // Default implementation - can be overridden
        Ok(())
    }
}

/// Enhanced Service Stream with flow control capabilities
/// 
/// This trait extends the basic Stream functionality with advanced
/// flow control, pause/resume capabilities, and automatic cleanup.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::core::core_traits::EnhancedServiceStream;
/// use futures::StreamExt;
/// 
/// async fn handle_stream(mut stream: impl EnhancedServiceStream) {
///     // Pause processing temporarily
///     stream.pause().await.unwrap();
///     
///     // Resume when ready
///     stream.resume().await.unwrap();
///     
///     // Process messages
///     while let Some(response) = stream.next().await {
///         match response {
///             Ok(msg) => println!("Received: {:?}", msg),
///             Err(e) => eprintln!("Stream error: {}", e),
///         }
///     }
/// } // Stream automatically cancels on drop
/// ```
#[async_trait]
pub trait EnhancedServiceStream: Stream<Item = Result<JsonRpcResponse>> + Send + Unpin {
    /// Pause the stream processing
    /// 
    /// This allows for backpressure control and temporary halting
    /// of message processing without canceling the entire stream.
    async fn pause(&mut self) -> Result<()>;
    
    /// Resume the stream processing
    /// 
    /// Resumes a previously paused stream, allowing messages to
    /// flow again.
    async fn resume(&mut self) -> Result<()>;
    
    /// Check if the stream is currently paused
    fn is_paused(&self) -> bool;
    
    /// Cancel the stream and clean up resources
    async fn cancel(&mut self) -> Result<()>;
    
    /// Check if the stream is cancelled
    fn is_cancelled(&self) -> bool;
    
    /// Get stream metadata and statistics
    fn stream_info(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

// Note: Auto-cleanup for ServiceStream would need to be implemented
// on concrete types, not as a blanket implementation on the trait
// This would be part of the concrete implementations in Phase 2+

/// Method handler trait for individual JSON-RPC methods
/// 
/// This trait handles single method calls with request/response semantics.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::Result;
/// use async_trait::async_trait;
/// use serde_json::json;
/// 
/// struct WeatherHandler;
/// 
/// #[async_trait]
/// impl MethodHandler for WeatherHandler {
///     async fn handle_method(
///         &self,
///         request: &JsonRpcRequest,
///         context: &ServiceContext,
///     ) -> Result<JsonRpcResponse> {
///         if request.method == "get_weather" {
///             Ok(JsonRpcResponse::success(
///                 request.id.clone().unwrap_or(json!(null)),
///                 json!({"temperature": 25, "condition": "sunny"})
///             ))
///         } else {
///             Ok(JsonRpcResponse::error(
///                 request.id.clone().unwrap_or(json!(null)),
///                 JsonRpcError::new(JsonRpcErrorCode::MethodNotFound, "Method not found")
///             ))
///         }
///     }
///     
///     fn supported_methods(&self) -> Vec<String> {
///         vec!["get_weather".to_string()]
///     }
/// }
/// ```
#[async_trait]
pub trait MethodHandler: Send + Sync {
    /// Handle a single method call
    async fn handle_method(
        &self,
        request: &JsonRpcRequest,
        context: &ServiceContext,
    ) -> Result<JsonRpcResponse>;
    
    /// Get list of supported methods
    fn supported_methods(&self) -> Vec<String>;
    
    /// Check if a method is supported
    fn supports_method(&self, method: &str) -> bool {
        self.supported_methods().contains(&method.to_string())
    }
}

/// Stream handler trait for streaming JSON-RPC operations
/// 
/// This trait handles streaming operations where a single request
/// can generate multiple responses over time.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::Result;
/// use jsonrpc_rust::core::core_traits::EnhancedServiceStream;
/// use async_trait::async_trait;
/// use std::pin::Pin;
/// 
/// struct LogStreamHandler;
/// 
/// #[async_trait]
/// impl StreamHandler for LogStreamHandler {
///     async fn handle_stream(
///         &self,
///         request: &JsonRpcRequest,
///         context: &ServiceContext,
///     ) -> Result<Pin<Box<dyn EnhancedServiceStream>>> {
///         // Create and return a stream that emits log entries
///         todo!("Implement log streaming")
///     }
///     
///     fn supported_streams(&self) -> Vec<String> {
///         vec!["tail_logs".to_string(), "monitor_events".to_string()]
///     }
/// }
/// ```
#[async_trait]
pub trait StreamHandler: Send + Sync {
    /// Handle a streaming request
    async fn handle_stream(
        &self,
        request: &JsonRpcRequest,
        context: &ServiceContext,
    ) -> Result<Pin<Box<dyn EnhancedServiceStream>>>;
    
    /// Get list of supported streaming methods
    fn supported_streams(&self) -> Vec<String>;
    
    /// Check if a streaming method is supported
    fn supports_stream(&self, method: &str) -> bool {
        self.supported_streams().contains(&method.to_string())
    }
}

/// Service information and capabilities
/// 
/// This trait provides metadata about the service, including
/// supported methods, version information, and health status.
#[async_trait]
pub trait ServiceInfo: Send + Sync {
    /// Get service name
    fn service_name(&self) -> &str;
    
    /// Get service version
    fn service_version(&self) -> &str;
    
    /// Get all supported methods (both regular and streaming)
    fn supported_methods(&self) -> Vec<String>;
    
    /// Get service capabilities and metadata
    fn capabilities(&self) -> HashMap<String, Value>;
    
    /// Check service health
    async fn health_check(&self) -> Result<HashMap<String, Value>>;
}

/// Bidirectional stream trait for future expansion
/// 
/// This trait is a placeholder for future bidirectional streaming
/// capabilities, where both client and server can send messages
/// in either direction.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::core::traits::BidirectionalStream;
/// use jsonrpc_rust::Result;
/// use async_trait::async_trait;
/// use serde_json::json;
/// use std::collections::VecDeque;
/// 
/// // Mock bidirectional stream for testing
/// struct MockBidirectionalStream {
///     outbound: VecDeque<JsonRpcRequest>,
///     inbound: VecDeque<JsonRpcResponse>,
///     open: bool,
/// }
/// 
/// impl MockBidirectionalStream {
///     fn new() -> Self {
///         Self {
///             outbound: VecDeque::new(),
///             inbound: VecDeque::new(),
///             open: true,
///         }
///     }
///     
///     fn add_response(&mut self, response: JsonRpcResponse) {
///         self.inbound.push_back(response);
///     }
/// }
/// 
/// #[async_trait]
/// impl BidirectionalStream for MockBidirectionalStream {
///     async fn send(&mut self, message: JsonRpcRequest) -> Result<()> {
///         if !self.open {
///             return Err(Error::Transport { 
///                 message: "Stream is closed".to_string(), 
///                 source: None 
///             });
///         }
///         self.outbound.push_back(message);
///         Ok(())
///     }
///     
///     async fn receive(&mut self) -> Result<JsonRpcResponse> {
///         if !self.open {
///             return Err(Error::Transport { 
///                 message: "Stream is closed".to_string(), 
///                 source: None 
///             });
///         }
///         self.inbound.pop_front()
///             .ok_or_else(|| Error::Transport { 
///                 message: "No messages available".to_string(), 
///                 source: None 
///             })
///     }
///     
///     async fn close(&mut self) -> Result<()> {
///         self.open = false;
///         Ok(())
///     }
///     
///     fn is_open(&self) -> bool {
///         self.open
///     }
/// }
/// 
/// async fn handle_bidirectional() -> Result<()> {
///     let mut stream = MockBidirectionalStream::new();
///     
///     // Add a response to simulate server message
///     stream.add_response(JsonRpcResponse::success(
///         json!(1), 
///         json!({"status": "connected"})
///     ));
///     
///     // Send a request
///     let request = JsonRpcRequest::new("ping", Some(json!({})));
///     stream.send(request).await?;
///     
///     // Receive a response
///     let response = stream.receive().await?;
///     println!("Received: {:?}", response.result);
///     
///     // Close the stream
///     stream.close().await?;
///     assert!(!stream.is_open());
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait BidirectionalStream: Send + Sync {
    /// Send a message to the peer
    async fn send(&mut self, message: JsonRpcRequest) -> Result<()>;
    
    /// Receive a message from the peer
    async fn receive(&mut self) -> Result<JsonRpcResponse>;
    
    /// Close the bidirectional stream
    async fn close(&mut self) -> Result<()>;
    
    /// Check if the stream is still open
    fn is_open(&self) -> bool;
}

/// Legacy JSON-RPC Service trait (maintained for compatibility)
/// 
/// This is the original monolithic service trait. For new code,
/// consider using the more focused MethodHandler, StreamHandler,
/// and ServiceInfo traits instead.
#[deprecated(since = "0.1.1", note = "Use MethodHandler, StreamHandler, and ServiceInfo instead")]
#[async_trait]
pub trait JsonRpcService: Send + Sync {
    /// Handle a JSON-RPC request
    async fn handle_request(
        &self,
        request: JsonRpcRequest,
        _context: ServiceContext,
    ) -> JsonRpcFuture;
    
    /// Handle a streaming request (optional)
    async fn handle_stream(
        &self,
        request: JsonRpcRequest,
        _context: ServiceContext,
    ) -> Result<ServiceStream> {
        Err(Error::method_not_found(&request.method))
    }
    
    /// Get service information
    fn service_info(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Message serialization trait
/// 
/// This trait handles the conversion between Rust types and JSON-RPC
/// message formats, supporting different serialization strategies.
/// 
/// # Example
/// 
/// ```rust
/// use jsonrpc_rust::core::prelude::*;
/// use jsonrpc_rust::Result;
/// use serde_json::Value;
/// 
/// struct JsonSerializer;
/// 
/// impl MessageSerializer for JsonSerializer {
///     fn serialize_request(&self, request: &JsonRpcRequest) -> Result<String> {
///         serde_json::to_string(request)
///             .map_err(|e| Error::Serialization { message: format!("JSON serialization failed: {}", e), source: Some(Box::new(e)) })
///     }
///     
///     fn deserialize_request(&self, data: &str) -> Result<JsonRpcRequest> {
///         serde_json::from_str(data)
///             .map_err(|e| Error::Serialization { message: format!("JSON deserialization failed: {}", e), source: Some(Box::new(e)) })
///     }
///     
///     fn serialize_response(&self, response: &JsonRpcResponse) -> Result<String> {
///         serde_json::to_string(response)
///             .map_err(|e| Error::Serialization { message: format!("JSON serialization failed: {}", e), source: Some(Box::new(e)) })
///     }
///     
///     fn deserialize_response(&self, data: &str) -> Result<JsonRpcResponse> {
///         serde_json::from_str(data)
///             .map_err(|e| Error::Serialization { message: format!("JSON deserialization failed: {}", e), source: Some(Box::new(e)) })
///     }
/// }
/// ```
pub trait MessageSerializer: Send + Sync {
    /// Serialize a JSON-RPC request
    fn serialize_request(&self, request: &JsonRpcRequest) -> Result<String>;
    
    /// Deserialize a JSON-RPC request
    fn deserialize_request(&self, data: &str) -> Result<JsonRpcRequest>;
    
    /// Serialize a JSON-RPC response
    fn serialize_response(&self, response: &JsonRpcResponse) -> Result<String>;
    
    /// Deserialize a JSON-RPC response
    fn deserialize_response(&self, data: &str) -> Result<JsonRpcResponse>;
    
    /// Get serialization format name
    fn format_name(&self) -> &str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::*;
    use serde_json::json;

    struct MockMessage {
        method: String,
        id: Option<Value>,
    }

    impl fmt::Debug for MockMessage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MockMessage")
                .field("method", &self.method)
                .field("id", &self.id)
                .finish()
        }
    }

    impl Message for MockMessage {
        fn jsonrpc(&self) -> &str { "2.0" }
        fn method(&self) -> &str { &self.method }
        fn is_notification(&self) -> bool { self.id.is_none() }
        fn id(&self) -> Option<&Value> { self.id.as_ref() }
        fn to_json(&self) -> Result<String> { Ok("{}".to_string()) }
    }

    #[test]
    fn test_mock_message() {
        let msg = MockMessage {
            method: "test".to_string(),
            id: Some(json!(1)),
        };
        
        assert_eq!(msg.method(), "test");
        assert_eq!(msg.jsonrpc(), "2.0");
        assert!(!msg.is_notification());
        assert_eq!(msg.id(), Some(&json!(1)));
    }

    struct JsonSerializer;

    impl MessageSerializer for JsonSerializer {
        fn serialize_request(&self, request: &JsonRpcRequest) -> Result<String> {
            serde_json::to_string(request)
                .map_err(|e| Error::serialization(format!("Failed to serialize: {}", e)))
        }
        
        fn deserialize_request(&self, data: &str) -> Result<JsonRpcRequest> {
            serde_json::from_str(data)
                .map_err(|e| Error::serialization(format!("Failed to deserialize: {}", e)))
        }
        
        fn serialize_response(&self, response: &JsonRpcResponse) -> Result<String> {
            serde_json::to_string(response)
                .map_err(|e| Error::serialization(format!("Failed to serialize: {}", e)))
        }
        
        fn deserialize_response(&self, data: &str) -> Result<JsonRpcResponse> {
            serde_json::from_str(data)
                .map_err(|e| Error::serialization(format!("Failed to deserialize: {}", e)))
        }
    }

    #[test]
    fn test_json_serializer() {
        let serializer = JsonSerializer;
        let request = JsonRpcRequest::new("test", Some(json!({"param": "value"})));
        
        let serialized = serializer.serialize_request(&request).unwrap();
        let deserialized = serializer.deserialize_request(&serialized).unwrap();
        
        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    }
} 