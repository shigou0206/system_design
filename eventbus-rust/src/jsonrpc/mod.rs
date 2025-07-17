//! JSON-RPC integration for EventBus
//! 
//! This module provides JSON-RPC server and client implementations
//! for the EventBus service using the jsonrpc-rust framework.

pub mod methods;
pub mod server;
pub mod client;

// Re-export commonly used types
pub use methods::*;
pub use server::*;
pub use client::*; 