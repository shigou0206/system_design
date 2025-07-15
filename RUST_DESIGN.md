# TRN-Rust Library Design Document

## Overview

This document outlines the design for a complete Rust reimplementation of the TRN (Tool Resource Name) library. The Rust version will provide the same functionality as the Python version but with superior performance, memory safety, and zero-cost abstractions.

## Goals

### Primary Goals
- **Performance**: 10x faster parsing and validation compared to Python
- **Memory Safety**: Leverage Rust's ownership system for zero memory leaks
- **Type Safety**: Compile-time guarantees for TRN correctness
- **Zero-Copy**: Minimize allocations where possible
- **Concurrency**: Thread-safe operations without locks

### Secondary Goals
- **C FFI**: Export C-compatible API for other languages
- **WASM Support**: Compile to WebAssembly for browser usage
- **CLI Tools**: Provide command-line utilities
- **Benchmarking**: Performance comparison suite

## Architecture Design

### Module Structure

```
trn-rust/
├── Cargo.toml                 # Project configuration
├── src/
│   ├── lib.rs                 # Library root and public API
│   ├── types.rs               # Core TRN types and structures
│   ├── error.rs               # Error types and handling
│   ├── constants.rs           # Constants, patterns, and configs
│   ├── validation.rs          # TRN validation with caching
│   ├── parsing.rs             # TRN parsing with error recovery
│   ├── url.rs                 # URL conversion functionality
│   ├── utils.rs               # Utility functions and helpers
│   ├── builder.rs             # Builder pattern implementation
│   └── pattern.rs             # Pattern matching and filtering
├── benches/                   # Performance benchmarks
├── examples/                  # Usage examples
├── tests/                     # Integration tests
└── ffi/                       # C FFI bindings
```

### Core Type Design

#### Primary Types

```rust
// Core TRN structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Trn {
    platform: String,
    scope: Option<String>,
    resource_type: String,
    type_: String,
    subtype: Option<String>,
    instance_id: String,
    version: String,
    tag: Option<String>,
    hash: Option<String>,
}

// Components structure for zero-copy parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrnComponents<'a> {
    platform: &'a str,
    scope: Option<&'a str>,
    resource_type: &'a str,
    type_: &'a str,
    subtype: Option<&'a str>,
    instance_id: &'a str,
    version: &'a str,
    tag: Option<&'a str>,
    hash: Option<&'a str>,
}

// Builder for fluent construction
#[derive(Default)]
pub struct TrnBuilder {
    platform: Option<String>,
    scope: Option<String>,
    resource_type: Option<String>,
    type_: Option<String>,
    subtype: Option<String>,
    instance_id: Option<String>,
    version: Option<String>,
    tag: Option<String>,
    hash: Option<String>,
}
```

#### Enums for Type Safety

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    AiPlatform,
    User,
    Org,
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Tool,
    Dataset,
    Pipeline,
    Model,
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    OpenApi,
    Workflow,
    Python,
    Shell,
    System,
    AsyncApi,
    Custom(&'static str),
}
```

### Error Handling Strategy

#### Error Type Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum TrnError {
    #[error("TRN format error: {message}")]
    Format { message: String, trn: Option<String> },
    
    #[error("TRN validation error: {message}")]
    Validation { message: String, rule: String },
    
    #[error("TRN component error: {message}")]
    Component { message: String, component: String },
    
    #[error("TRN length error: {message}")]
    Length { message: String, length: usize, max_length: usize },
    
    #[error("TRN character error: {message}")]
    Character { message: String, invalid_char: char, position: usize },
    
    #[error("TRN hash error: {message}")]
    Hash { message: String, expected: Option<String>, actual: Option<String> },
    
    #[error("TRN URL error: {message}")]
    Url { message: String, url: Option<String> },
}

pub type Result<T> = std::result::Result<T, TrnError>;
```

### Performance Optimizations

#### Caching Strategy

```rust
use std::sync::Arc;
use dashmap::DashMap;

#[derive(Clone)]
pub struct ValidationCache {
    cache: Arc<DashMap<String, bool>>,
    max_size: usize,
}

impl ValidationCache {
    pub fn get(&self, key: &str) -> Option<bool> { /* ... */ }
    pub fn insert(&self, key: String, value: bool) { /* ... */ }
}
```

#### Zero-Copy Parsing

```rust
pub fn parse_components(input: &str) -> Result<TrnComponents<'_>> {
    // Use nom parser for zero-copy parsing
    // Return borrowed string slices instead of owned strings
}
```

#### Lazy Static Patterns

```rust
use once_cell::sync::Lazy;
use regex::Regex;

static TRN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^trn:([a-z][a-z0-9-]{1,31})...").unwrap()
});
```

## API Design

### Core API

```rust
impl Trn {
    // Construction
    pub fn new(platform: impl Into<String>, /* ... */) -> Result<Self>;
    pub fn parse(input: &str) -> Result<Self>;
    pub fn from_components(components: TrnComponents<'_>) -> Result<Self>;
    
    // Validation
    pub fn validate(&self) -> Result<()>;
    pub fn is_valid(&self) -> bool;
    
    // Conversion
    pub fn to_string(&self) -> String;
    pub fn to_url(&self) -> Result<String>;
    pub fn to_http_url(&self, base: &str) -> Result<String>;
    
    // Component access
    pub fn platform(&self) -> &str;
    pub fn scope(&self) -> Option<&str>;
    // ... other getters
    
    // Manipulation
    pub fn without_hash(&self) -> Self;
    pub fn without_tag(&self) -> Self;
    pub fn base_trn(&self) -> Self;
    
    // Comparison
    pub fn matches_pattern(&self, pattern: &str) -> bool;
    pub fn is_compatible_with(&self, other: &Self) -> bool;
}
```

### Utility Functions

```rust
// Pattern matching
pub fn find_matching_trns<'a>(
    trns: &'a [String], 
    pattern: &str
) -> Vec<&'a String>;

// Version comparison
pub fn compare_versions(v1: &str, v2: &str, op: VersionOp) -> bool;

// Batch operations
pub fn batch_validate(trns: &[String]) -> ValidationReport;

// Normalization
pub fn normalize_trn(input: &str) -> String;
```

### Builder Pattern

```rust
impl TrnBuilder {
    pub fn new() -> Self;
    pub fn platform(mut self, platform: impl Into<String>) -> Self;
    pub fn scope(mut self, scope: impl Into<String>) -> Self;
    // ... other setters
    pub fn build(self) -> Result<Trn>;
}
```

## Dependencies Selection

### Core Dependencies

```toml
[dependencies]
# Error handling
thiserror = "1.0"        # Derive error types
anyhow = "1.0"           # Error context

# Parsing
nom = "7.0"              # Parser combinators
regex = "1.0"            # Regular expressions

# Caching and performance
once_cell = "1.0"        # Lazy static initialization
dashmap = "5.0"          # Concurrent hashmap

# Serialization
serde = { version = "1.0", features = ["derive"] }

# URL handling
url = "2.0"              # URL parsing and building

# String utilities
unicode-normalization = "0.1"  # Unicode handling
```

### Optional Dependencies

```toml
[dependencies]
# CLI support
clap = { version = "4.0", optional = true }

# C FFI
libc = { version = "0.2", optional = true }

[features]
default = []
cli = ["clap"]
ffi = ["libc"]
```

## Memory Management Strategy

### Owned vs Borrowed

- **Owned Types**: For public API and long-lived objects
- **Borrowed Types**: For parsing and temporary operations
- **Cow Types**: For conditional ownership

### String Handling

```rust
use std::borrow::Cow;

pub enum TrnString<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> From<&'a str> for TrnString<'a> {
    fn from(s: &'a str) -> Self {
        TrnString::Borrowed(s)
    }
}
```

## Concurrency Design

### Thread Safety

- All types implement `Send + Sync`
- Validation cache uses `DashMap` for lock-free access
- Regex patterns are compiled once and shared

### Async Support

```rust
#[cfg(feature = "async")]
impl Trn {
    pub async fn validate_async(&self) -> Result<()>;
    pub async fn resolve_aliases(&mut self) -> Result<()>;
}
```

## Testing Strategy

### Unit Tests

- Each module has comprehensive unit tests
- Property-based testing with `proptest`
- Fuzzing with `cargo-fuzz`

### Integration Tests

- Cross-language compatibility tests
- Performance regression tests
- Memory leak detection

### Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parsing(c: &mut Criterion) {
    c.bench_function("parse_simple_trn", |b| {
        b.iter(|| Trn::parse(black_box("trn:user:alice:tool:openapi:github-api:v1.0")))
    });
}
```

## Platform Support

### Target Platforms

- **Linux**: Primary development platform
- **macOS**: Full support
- **Windows**: Full support
- **WebAssembly**: Browser and Node.js support

### Architecture Support

- **x86_64**: Primary architecture
- **ARM64**: Full support for Apple Silicon and servers
- **RISC-V**: Basic support

## FFI Design

### C API

```c
// C header file
typedef struct trn_t trn_t;

trn_t* trn_parse(const char* input, char** error);
void trn_free(trn_t* trn);
const char* trn_to_string(const trn_t* trn);
bool trn_validate(const trn_t* trn);
```

### Python Bindings

Using `pyo3` for seamless Python integration:

```rust
use pyo3::prelude::*;

#[pyclass]
struct PyTrn {
    inner: Trn,
}

#[pymethods]
impl PyTrn {
    #[new]
    fn new(input: &str) -> PyResult<Self> {
        Ok(PyTrn {
            inner: Trn::parse(input).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
        })
    }
}
```

## Documentation Strategy

### API Documentation

- Comprehensive rustdoc comments
- Usage examples for all public functions
- Link to design decisions

### User Guide

- Getting started tutorial
- Migration guide from Python version
- Performance tuning guide

### Integration Examples

- Web framework integration
- CLI application examples
- FFI usage examples

## Release Strategy

### Versioning

- Semantic versioning (SemVer)
- Feature flags for breaking changes
- Long-term support versions

### Distribution

- crates.io for Rust ecosystem
- GitHub releases with precompiled binaries
- Docker images for containerized usage

## Migration from Python

### API Compatibility

Provide similar API surface where possible:

```rust
// Rust equivalent of Python API
impl Trn {
    pub fn parse(input: &str) -> Result<Self>;        // TRN.parse()
    pub fn to_url(&self) -> Result<String>;           // trn.to_url()
    pub fn matches_pattern(&self, pattern: &str) -> bool;  // trn.matches_pattern()
}
```

### Performance Comparison

Target performance improvements:
- Parsing: 10-50x faster
- Validation: 5-20x faster
- Memory usage: 2-5x lower
- Startup time: 10-100x faster

This design provides a solid foundation for implementing a high-performance, memory-safe TRN library in Rust that maintains API compatibility with the Python version while leveraging Rust's unique strengths. 