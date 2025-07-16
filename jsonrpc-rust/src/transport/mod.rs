//! Transport layer implementations for JSON-RPC framework
//! 
//! This module provides concrete implementations of transport protocols
//! for JSON-RPC communication, including TCP, WebSocket, HTTP, and mock
//! transports for testing.
//! 
//! # Architecture
//! 
//! The transport layer is built on top of the core traits defined in
//! `core::traits` and provides:
//! 
//! - **Abstraction Layer**: Common interfaces and utilities
//! - **Protocol Implementations**: TCP, WebSocket, HTTP transports
//! - **Registry**: Dynamic transport selection and management
//! - **Testing**: Mock implementations for unit tests
//! 
//! # Example
//! 
//! ```rust
//! use jsonrpc_rust::transport::prelude::*;
//! use jsonrpc_rust::transport::tcp::TcpTransport;
//! 
//! # async fn example() -> jsonrpc_rust::Result<()> {
//! // Create a TCP transport
//! let mut transport = TcpTransport::new("127.0.0.1:8080").await?;
//! 
//! // Send a JSON-RPC message
//! transport.send(r#"{"jsonrpc":"2.0","method":"ping","id":1}"#).await?;
//! 
//! // Receive response
//! let response = transport.receive().await?;
//! println!("Received: {}", response);
//! # Ok(())
//! # }
//! ```

// Core transport abstractions
pub mod abstraction;

// Protocol implementations
pub mod tcp;
pub mod mock;

// Transport registry
pub mod registry;

// Optional protocol implementations (feature-gated)
#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "http")]
pub mod http;

// Re-export commonly used types
pub use abstraction::*;
pub use tcp::*;
pub use mock::*;
pub use registry::*;

#[cfg(feature = "websocket")]
pub use websocket::*;

#[cfg(feature = "http")]
pub use http::*;

/// Prelude module for convenient imports
pub mod prelude {
    //! Common imports for transport layer usage
    
    // Core transport traits
    pub use super::abstraction::{
        TransportLayer, ConnectionManager, MessageCodec,
        TransportConfig, ConnectionInfo, TransportError
    };
    
    // Concrete implementations
    pub use super::tcp::{TcpTransport, TcpConnection, TcpConfig};
    pub use super::mock::{MockTransport, MockConnection, MockConfig};
    pub use super::registry::{TransportRegistry, TransportType, RegistryConfig};
    
    // Core traits from parent modules
    pub use crate::core::traits::{Transport, Connection, Message};
    pub use crate::core::types::{JsonRpcRequest, JsonRpcResponse, MessageId};
    pub use crate::core::error::{Error, Result};
    
    // Optional features
    #[cfg(feature = "websocket")]
    pub use super::websocket::{WebSocketTransport, WebSocketConnection, WebSocketConfig};
    
    #[cfg(feature = "http")]
    pub use super::http::{HttpTransport, HttpConnection, HttpConfig};
}

/// Transport layer version information
pub const TRANSPORT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported transport protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    /// TCP-based transport
    Tcp,
    /// WebSocket transport
    WebSocket,
    /// HTTP POST transport
    Http,
    /// Mock transport for testing
    Mock,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::WebSocket => write!(f, "websocket"),
            Protocol::Http => write!(f, "http"),
            Protocol::Mock => write!(f, "mock"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::Tcp),
            "websocket" | "ws" => Ok(Protocol::WebSocket),
            "http" | "https" => Ok(Protocol::Http),
            "mock" => Ok(Protocol::Mock),
            _ => Err(format!("Unknown protocol: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Tcp.to_string(), "tcp");
        assert_eq!(Protocol::WebSocket.to_string(), "websocket");
        assert_eq!(Protocol::Http.to_string(), "http");
        assert_eq!(Protocol::Mock.to_string(), "mock");
    }
    
    #[test]
    fn test_protocol_from_str() {
        assert_eq!("tcp".parse::<Protocol>().unwrap(), Protocol::Tcp);
        assert_eq!("websocket".parse::<Protocol>().unwrap(), Protocol::WebSocket);
        assert_eq!("ws".parse::<Protocol>().unwrap(), Protocol::WebSocket);
        assert_eq!("http".parse::<Protocol>().unwrap(), Protocol::Http);
        assert_eq!("https".parse::<Protocol>().unwrap(), Protocol::Http);
        assert_eq!("mock".parse::<Protocol>().unwrap(), Protocol::Mock);
        
        assert!("unknown".parse::<Protocol>().is_err());
    }
} 