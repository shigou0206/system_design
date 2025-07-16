//! Core module for jsonrpc-rust
//! 
//! This module provides the foundational types, traits, and functionality
//! for the JSON-RPC framework.

// Core module exports and organization

// Module declarations
pub mod error;
pub mod types;
pub mod traits;
pub mod future;

// Organized public exports
pub mod core_types {
    //! Core type definitions
    pub use super::types::{
        JsonRpcRequest, JsonRpcResponse, ServiceResponse, ResponseType,
        ServiceContext, AuthContext, MessageId, MessageMetadata,
        StreamMessage, ResponsePayload, ResponseMetaInfo
    };
    
    // Import ServiceInfo struct with alias to avoid conflicts
    pub use super::types::ServiceInfo as ServiceInfoStruct;
}

pub mod core_traits {
    //! Core trait definitions
    pub use super::traits::{
        Message, Transport, Connection, MessageSerializer,
        MethodHandler, StreamHandler, EnhancedServiceStream,
        BidirectionalStream
    };
    
    // Import ServiceInfo trait with alias to avoid conflicts
    pub use super::traits::ServiceInfo as ServiceInfoTrait;
}

pub mod errors {
    //! Error handling types
    pub use super::error::{Error, ErrorKind, JsonRpcError, JsonRpcErrorCode, RetryPolicy};
    
    #[cfg(feature = "debug-location")]
    pub use super::error::SourceLocation;
}

pub mod streaming {
    //! Streaming and future types
    pub use super::future::{
        JsonRpcFuture, JsonRpcStream, ServiceStream, StreamControl, BackpressureSignal
    };
}

// TRN integration (conditional)
#[cfg(feature = "trn-integration")]
pub mod trn {
    //! TRN (Tool Resource Name) integration
    pub use super::types::{TrnContext, MessageMetadata};
}

// Prelude module for common imports
pub mod prelude {
    //! Common imports for jsonrpc-rust
    //! 
    //! This module re-exports the most commonly used types and traits,
    //! allowing users to import everything they need with a single use statement:
    //! 
    //! ```rust
    //! use jsonrpc_rust::core::prelude::*;
    //! ```
    
    // Essential types
    pub use super::types::{
        JsonRpcRequest, JsonRpcResponse, ServiceResponse, ResponseType,
        ServiceContext, AuthContext, MessageId, 
        ChannelBidirectionalStream, ChannelBidirectionalStreamPeer
    };
    
    // Core traits (using new trait design)
    pub use super::traits::{
        Message, Transport, Connection, MessageSerializer,
        MethodHandler, StreamHandler
    };
    
    // Import ServiceInfo with clear names
    pub use super::types::ServiceInfo as ServiceInfoStruct;
    pub use super::traits::ServiceInfo as ServiceInfoTrait;
    
    // Error handling
    pub use super::error::{Error, ErrorKind, JsonRpcError, JsonRpcErrorCode};
    
    // Futures and streams
    pub use super::future::{JsonRpcFuture, JsonRpcStream, ServiceStream};
    
    // TRN integration (conditional)
    #[cfg(feature = "trn-integration")]
    pub use super::types::TrnContext;
}

// Modern recommended exports (no glob imports to avoid conflicts)
pub use self::prelude::*; 