//! Core types and traits for the event bus system
//! 
//! This module provides the foundational data structures and abstractions
//! that all other modules build upon.

pub mod types;
pub mod traits;
pub mod error;

// Re-export all public items
pub use types::*;
pub use traits::*;
pub use error::*; 