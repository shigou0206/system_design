//! # TRN-Rust: High-Performance Tool Resource Name Library
//!
//! A high-performance Rust library for parsing, validating, and manipulating
//! Tool Resource Names (TRN) in AI Agent platforms.
//!
//! ## Overview
//!
//! TRN (Tool Resource Name) is a standardized identifier format for tools and resources
//! in AI Agent platforms. This library provides comprehensive functionality for:
//!
//! - Parsing and validating TRN strings
//! - Converting between TRN and URL formats
//! - Pattern matching and filtering
//! - Version comparison and management
//! - Builder pattern for TRN construction
//!
//! ## TRN Format
//!
//! ```text
//! trn:platform[:scope]:resource_type:type[:subtype]:instance_id:version[:tag][@hash]
//! ```
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use trn_rust::{Trn, TrnBuilder};
//!
//! // Parse a TRN string
//! let trn = Trn::parse("trn:user:alice:tool:openapi::getUserById:v1.0:")?;
//! println!("Platform: {}", trn.platform());
//! println!("Scope: {:?}", trn.scope());
//! println!("Type: {}", trn.type_());
//!
//! // Create using builder pattern
//! let trn = TrnBuilder::new()
//!     .platform("user")
//!     .scope("alice")
//!     .resource_type("tool")
//!     .type_("openapi")
//!     .subtype("")
//!     .instance_id("getUserById")
//!     .version("v1.0")
//!     .tag("")
//!     .build()?;
//!
//! // Convert to string
//! println!("TRN: {}", trn.to_string());
//! # Ok::<(), trn_rust::TrnError>(())
//! ```
//!
//! ### URL Conversion
//!
//! ```rust
//! use trn_rust::Trn;
//!
//! let trn = Trn::parse("trn:user:alice:tool:openapi::getUserById:v1.0:")?;
//!
//! // Convert to TRN URL
//! let trn_url = trn.to_url()?;
//! println!("TRN URL: {}", trn_url);
//!
//! // Convert to HTTP URL
//! let http_url = trn.to_http_url("https://platform.example.com")?;
//! println!("HTTP URL: {}", http_url);
//! # Ok::<(), trn_rust::TrnError>(())
//! ```
//!
//! ### Pattern Matching
//!
//! ```rust
//! use trn_rust::utils::find_matching_trns;
//!
//! let trns = vec![
//!     "trn:user:alice:tool:openapi::getUserById:v1.0:".to_string(),
//!     "trn:user:alice:tool:openapi::sendMessage:v2.0:".to_string(),
//!     "trn:user:bob:tool:python::processData:v1.5:".to_string(),
//! ];
//!
//! let alice_tools = find_matching_trns(&trns, "trn:user:alice:*");
//! println!("Alice's tools: {:#?}", alice_tools);
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Core modules
mod constants;
mod error;
mod types;

// Main functionality modules
mod builder;
mod parsing;
mod pattern;
mod url;
mod utils;
mod validation;

// Re-export public API
pub use builder::TrnBuilder;
pub use error::{TrnError, TrnResult};
pub use types::{Platform, ResourceType, ToolType, Trn, TrnComponents};

// Re-export utility functions
pub use utils::*;

// Re-export URL conversion functions
pub use url::url_to_trn;

// Re-export validation functions
pub use validation::{
    batch_validate, is_valid_trn, is_valid_identifier, is_valid_scope, is_valid_version,
    normalize_trn, TrnValidator, TrnStats, ValidationReport
};

// Note: Validate trait is defined in this module, not re-exported

// Re-export pattern matching
pub use pattern::{find_matching_trns, TrnMatcher};

// Feature-gated modules
#[cfg(feature = "cli")]
#[cfg_attr(docsrs, doc(cfg(feature = "cli")))]
pub mod cli;

#[cfg(feature = "ffi")]
#[cfg_attr(docsrs, doc(cfg(feature = "ffi")))]
pub mod ffi;

#[cfg(feature = "python")]
#[cfg_attr(docsrs, doc(cfg(feature = "python")))]
mod python;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Convenience function to parse a TRN string
///
/// This is equivalent to [`Trn::parse`] but can be used as a free function.
///
/// # Examples
///
/// ```rust
/// use trn_rust::parse;
///
/// let trn = parse("trn:user:alice:tool:openapi::getUserById:v1.0:")?;
/// assert_eq!(trn.platform(), "user");
/// assert_eq!(trn.scope(), Some("alice"));
/// # Ok::<(), trn_rust::TrnError>(())
/// ```
pub fn parse(input: &str) -> TrnResult<Trn> {
    Trn::parse(input)
}

/// Trait for types that can be validated
pub trait Validate {
    /// Validate the item
    fn validate(&self) -> TrnResult<()>;
}

impl Validate for str {
    fn validate(&self) -> TrnResult<()> {
        validation::validate_trn_string(self)
    }
}

impl Validate for Trn {
    fn validate(&self) -> TrnResult<()> {
        validation::validate_trn_struct(self)
    }
}

/// Convenience function for TRN validation
///
/// Works with both string slices and Trn objects.
///
/// # Examples
///
/// ```rust
/// use trn_rust::{validate, Trn};
///
/// assert!(validate("trn:user:alice:tool:openapi::getUserById:v1.0:").is_ok());
/// assert!(validate("invalid-trn").is_err());
///
/// let trn = Trn::parse("trn:user:alice:tool:openapi::getUserById:v1.0:").unwrap();
/// assert!(validate(&trn).is_ok());
/// ```
pub fn validate<T: Validate + ?Sized>(input: &T) -> TrnResult<()> {
    input.validate()
}

/// Convenience function to create a TRN builder
///
/// This is equivalent to [`TrnBuilder::new`] but can be used as a free function.
///
/// # Examples
///
/// ```rust
/// use trn_rust::builder;
///
/// let trn = builder()
///     .platform("user")
///     .scope("alice")
///     .resource_type("tool")
///     .type_("openapi")
///     .subtype("")
///     .instance_id("getUserById")
///     .version("v1.0")
///     .tag("")
///     .build()?;
/// # Ok::<(), trn_rust::TrnError>(())
/// ```
pub fn builder() -> TrnBuilder {
    TrnBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_convenience_function() {
        let trn = parse("trn:user:alice:tool:openapi::getUserById:v1.0:").unwrap();
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
        assert_eq!(trn.instance_id(), "getUserById");
    }

    #[test]
    fn test_validate_convenience_function() {
        assert!(validate("trn:user:alice:tool:openapi::getUserById:v1.0:").is_ok());
        assert!(validate("invalid-trn-format").is_err());
        assert!(validate("").is_err());
    }

    #[test]
    fn test_builder_convenience_function() {
        let trn = builder()
            .platform("user")
            .scope("alice")
            .resource_type("tool")
            .type_("openapi")
            .subtype("")
            .instance_id("getUserById")
            .version("v1.0")
            .tag("")
            .build()
            .unwrap();

        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
    }

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.chars().any(|c| c.is_ascii_digit()));
    }
} 