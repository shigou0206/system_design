//! # jsonrpc-rust: Universal JSON-RPC Framework
//! 
//! A high-performance, feature-rich JSON-RPC 2.0 framework designed for building
//! scalable, production-ready services with advanced features like streaming,
//! priority scheduling, TRN integration, and comprehensive error handling.
//!
//! ## Key Features
//!
//! - **🚀 High Performance**: Async-first design with zero-cost abstractions
//! - **🔄 Streaming Support**: Built-in streaming and Server-Sent Events
//! - **🎯 Priority Scheduling**: Advanced priority-based execution control
//! - **🔒 Type Safety**: Comprehensive error handling with source location tracking
//! - **🏢 Multi-Tenant**: Deep TRN (Tool Resource Name) integration
//! - **📊 Monitoring**: Built-in metrics, tracing, and diagnostics
//! - **🔧 Extensible**: Modular architecture with plugin system
//!
//! ## Quick Start
//!
//! ```rust
//! use jsonrpc_rust::prelude::*;
//! use serde_json::json;
//!
//! // Create a simple JSON-RPC request
//! let request = JsonRpcRequest::new("get_weather", Some(json!({"city": "Tokyo"})));
//!
//! // Create a successful response
//! let response = JsonRpcResponse::success(
//!     request.id().cloned().unwrap(),
//!     json!({"temperature": 25, "condition": "sunny"})
//! );
//!
//! // Create service context with authentication
//! let context = ServiceContext::new("req-123")
//!     .with_auth_context(
//!         AuthContext::new("user-456", "bearer")
//!             .with_permission("weather:read")
//!             .with_role("user")
//!     );
//! ```
//!
//! ## TRN Integration Example
//!
//! ```rust
//! # #[cfg(feature = "trn-integration")]
//! # {
//! use jsonrpc_rust::prelude::*;
//! use serde_json::json;
//!
//! // Create TRN context for resource identification
//! let trn_context = TrnContext::new("user", "alice", "tool", "weather-api", "v1.0")
//!     .with_tenant_id("acme-corp")
//!     .with_namespace("production");
//!
//! // Integrate TRN into service context
//! let context = ServiceContext::new("req-789")
//!     .with_trn_context(trn_context);
//!
//! println!("TRN: {}", context.trn_context.unwrap().to_trn_string());
//! // Output: trn:user:alice:tool:weather-api:v1.0
//! # }
//! ```
//!
//! ## Streaming Example
//!
//! ```rust
//! use jsonrpc_rust::prelude::*;
//! use jsonrpc_rust::core::future::{SpawnPolicy, Priority};
//! use futures::StreamExt;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a stream of responses
//! let responses = vec![
//!     Ok(JsonRpcResponse::success(json!(1), json!({"step": 1}))),
//!     Ok(JsonRpcResponse::success(json!(2), json!({"step": 2}))),
//!     Ok(JsonRpcResponse::success(json!(3), json!({"step": 3}))),
//! ];
//!
//! let mut stream = JsonRpcStream::from_iter(responses)
//!     .set_policy(SpawnPolicy::new().with_priority(Priority::High));
//!
//! // Process stream items
//! while let Some(response) = stream.next().await {
//!     match response {
//!         Ok(resp) => println!("Received: {:?}", resp.result),
//!         Err(e) => eprintln!("Stream error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling Example
//!
//! ```rust
//! use jsonrpc_rust::prelude::*;
//!
//! // JSON-RPC specific errors
//! let not_found = JsonRpcError::method_not_found("unknown_method");
//! let invalid_params = JsonRpcError::invalid_params("Missing required parameter");
//!
//! // Framework errors with automatic conversion
//! let transport_error = Error::transport("Connection failed");
//! let timeout_error = Error::timeout("operation", std::time::Duration::from_secs(30));
//!
//! // Check if error is retryable
//! assert!(transport_error.is_retryable());
//! assert!(!Error::method_not_found("test").is_retryable());
//!
//! // Convert to JSON-RPC error for client response
//! let jsonrpc_error = transport_error.to_jsonrpc_error();
//! ```
//!
//! ## Architecture Overview
//!
//! The framework is organized into several layers:
//!
//! - **Core Layer (L1)**: Basic types, traits, error handling, futures
//! - **Transport Layer (L2)**: Protocol adapters (TCP, WebSocket, HTTP)  
//! - **Protocol Layer (L3)**: JSON-RPC 2.0 implementation, message routing
//! - **Extension Layer (L4)**: Streaming, events, advanced features
//! - **Convenience Layer (L5)**: Macros, builders, runtime wrappers
//!
//! ## Feature Flags
//!
//! - `std` - Standard library support (enabled by default)
//! - `tcp` - TCP transport support (enabled by default)
//! - `websocket` - WebSocket transport support
//! - `http` - HTTP transport support
//! - `sse` - Server-Sent Events support
//! - `trn-integration` - TRN (Tool Resource Name) integration
//! - `debug-location` - Source location tracking for debugging
//! - `mock` - Mock implementations for testing
//! - `benchmarks` - Benchmark support
//! - `fuzz` - Fuzzing support
//! - `prometheus` - Prometheus metrics integration

/// JSON-RPC version constant
pub const JSONRPC_VERSION: &str = "2.0";

/// Core module containing fundamental types and traits
pub mod core;

// Transport layer abstractions (Phase 2) - will be implemented in future phases
// pub mod transport;

// Protocol layer implementation (Phase 3) - will be implemented in future phases  
// pub mod protocol;

// Extension layer for advanced features (Phase 4) - will be implemented in future phases
// pub mod extensions;

// Convenience layer with macros and builders (Phase 5) - will be implemented in future phases
// pub mod convenience;

/// Prelude module for convenient imports
/// 
/// This module re-exports the most commonly used types and traits.
/// Import everything you need with:
///
/// ```rust
/// use jsonrpc_rust::prelude::*;
/// ```
pub mod prelude {
    //! Convenient re-exports of commonly used types and traits
    
    // Core types
    pub use crate::core::prelude::*;
    
    // Version constant
    pub use crate::JSONRPC_VERSION;
    
         // Future extensions (will be available in later phases)
     // #[cfg(feature = "tcp")]
     // pub use crate::transport::*;
     
     // pub use crate::protocol::*;
     // pub use crate::extensions::*;
     // pub use crate::convenience::*;
}

// Modern modular exports (recommended)
pub use core::prelude::*;

// Legacy flat exports (deprecated but maintained for compatibility)
#[deprecated(since = "0.1.1", note = "Use `jsonrpc_rust::prelude::*` or specific module imports instead")]
pub use core::{
    error::{Error, Result, ErrorKind, JsonRpcError, JsonRpcErrorCode},
    types::{
        JsonRpcRequest, JsonRpcResponse, ServiceResponse, ServiceContext,
        AuthContext, ClientInfo, MessageId
    },
    traits::{Message, Transport, Connection, MessageSerializer, MethodHandler, StreamHandler},
    future::{JsonRpcFuture, JsonRpcStream, ServiceStream, Priority, SpawnPolicy},
};

#[cfg(feature = "trn-integration")]
#[deprecated(since = "0.1.1", note = "Use `jsonrpc_rust::core::trn::*` instead")]
pub use core::types::{TrnContext, MessageMetadata};

/// Current version of the framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Framework information
pub mod info {
    /// Get framework version
    pub fn version() -> &'static str {
        crate::VERSION
    }
    
    /// Get JSON-RPC version supported
    pub fn jsonrpc_version() -> &'static str {
        crate::JSONRPC_VERSION
    }
    
    /// Get build information
    pub fn build_info() -> BuildInfo {
        BuildInfo {
            version: version(),
            jsonrpc_version: jsonrpc_version(),
            features: build_features(),
            build_timestamp: option_env!("JSONRPC_BUILD_TIMESTAMP"),
            git_hash: option_env!("JSONRPC_GIT_HASH"),
        }
    }
    
    /// Build features that are enabled
    fn build_features() -> Vec<&'static str> {
        let mut features = vec!["std"];
        
        #[cfg(feature = "tcp")]
        features.push("tcp");
        
        #[cfg(feature = "websocket")]
        features.push("websocket");
        
        #[cfg(feature = "http")]
        features.push("http");
        
        #[cfg(feature = "sse")]
        features.push("sse");
        
        #[cfg(feature = "trn-integration")]
        features.push("trn-integration");
        
        #[cfg(feature = "debug-location")]
        features.push("debug-location");
        
        #[cfg(feature = "mock")]
        features.push("mock");
        
        #[cfg(feature = "benchmarks")]
        features.push("benchmarks");
        
        #[cfg(feature = "fuzz")]
        features.push("fuzz");
        
        #[cfg(feature = "prometheus")]
        features.push("prometheus");
        
        features
    }
    
    /// Build information structure
    #[derive(Debug, Clone)]
    pub struct BuildInfo {
        /// Framework version
        pub version: &'static str,
        /// JSON-RPC version supported
        pub jsonrpc_version: &'static str,
        /// Enabled features
        pub features: Vec<&'static str>,
        /// Build timestamp (if available)
        pub build_timestamp: Option<&'static str>,
        /// Git commit hash (if available)
        pub git_hash: Option<&'static str>,
    }
}

// Placeholder modules for future phases (empty implementations for now)
#[cfg(feature = "tcp")]
pub mod transport {
    //! Transport layer abstractions (Phase 2)
    //! 
    //! This module will provide concrete implementations of transport protocols
    //! including TCP, WebSocket, and HTTP transports.
}

pub mod protocol {
    //! Protocol layer implementation (Phase 3)
    //! 
    //! This module will provide the core JSON-RPC 2.0 protocol implementation,
    //! message routing, and request/response handling.
}

pub mod extensions {
    //! Extension layer for advanced features (Phase 4)
    //! 
    //! This module will provide streaming support, event systems,
    //! and other advanced functionality.
}

pub mod convenience {
    //! Convenience layer with macros and builders (Phase 5)
    //! 
    //! This module will provide procedural macros, builder patterns,
    //! and runtime wrappers for easier usage.
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_info() {
        let build_info = info::build_info();
        assert!(!build_info.version.is_empty());
        assert_eq!(build_info.jsonrpc_version, "2.0");
        assert!(!build_info.features.is_empty());
        assert!(build_info.features.contains(&"std"));
    }
    
    #[test]
    fn test_jsonrpc_version() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }
}
