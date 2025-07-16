# TRN Rust Library - Examples

This directory contains comprehensive examples demonstrating how to use the TRN (Tool Resource Name) Rust library in various scenarios.

## Overview

The TRN Rust library provides high-performance parsing, validation, and manipulation of Tool Resource Names for AI Agent platforms. These examples show practical usage patterns and best practices.

## TRN Format

```
trn:platform:scope:resource_type:resource_id:version
```

### Components

- **platform**: `user`, `org`, or `aiplatform`
- **scope**: User/organization identifier (required for all platforms)
- **resource_type**: `tool`, `model`, `dataset`, `pipeline`, etc.
- **resource_id**: Unique identifier for the resource
- **version**: Version string (semantic versioning recommended)

## Examples

### 1. Basic Usage (`basic_usage.rs`)

Demonstrates fundamental TRN operations:

```bash
cargo run --example basic_usage
```

**Features covered:**
- Parsing TRN strings
- Building TRNs with the builder pattern
- Component access and validation
- URL conversion (trn:// and https://)
- Format conversion (JSON, YAML)
- Error handling patterns
- Creating variations from existing TRNs

**Sample TRNs used:**
```
trn:user:alice:tool:github-api:v1.0
trn:org:company:tool:user-onboarding:v2.1
trn:aiplatform:system:model:bert-base:latest
trn:user:bob:tool:data-processor:v1.5
```

### 2. Advanced Patterns (`advanced_patterns.rs`)

Shows sophisticated pattern matching and filtering:

```bash
cargo run --example advanced_patterns
```

**Features covered:**
- Wildcard pattern matching
- Complex filtering with multiple criteria
- Grouping by platform, tool type, and scope
- Version analysis and comparison
- Batch URL operations
- Validation analysis across collections
- Custom filtering functions
- Statistical analysis

**Use cases:**
- Finding all tools by a specific user
- Grouping tools by type or platform
- Analyzing version distributions
- Finding production-ready tools
- Identifying critical infrastructure components

### 3. CLI Tool Usage (`cli_usage.rs`)

Demonstrates command-line application patterns:

```bash
# Show usage
cargo run --example cli_usage

# Parse a TRN
cargo run --example cli_usage -- parse "trn:user:alice:tool:github-api:v1.0"

# Validate a TRN
cargo run --example cli_usage -- validate "trn:user:alice:tool:github-api:v1.0"

# Convert to URL
cargo run --example cli_usage -- convert "trn:user:alice:tool:github-api:v1.0" url

# Interactive builder
cargo run --example cli_usage -- build

# Run demonstration
cargo run --example cli_usage -- demo

# Process TRNs from file
cargo run --example cli_usage -- batch sample_trns.txt

# Analyze patterns in file
cargo run --example cli_usage -- analyze sample_trns.txt
```

**Features covered:**
- Command-line argument parsing
- Interactive TRN building
- Batch file processing
- Pattern analysis and statistics
- Error reporting and validation
- Multiple output formats

### 4. Performance Testing (`performance_testing.rs`)

Benchmarks and performance analysis:

```bash
cargo run --example performance_testing --release
```

**Performance metrics:**
- Parsing performance (operations/second)
- Building performance with builder pattern
- Validation speed with caching
- URL conversion throughput
- Batch operation efficiency
- Memory usage analysis
- Concurrent operation performance

**Typical results on modern hardware:**
- Parsing: 100K+ TRNs/second
- Building: 50K+ TRNs/second
- Validation: 200K+ validations/second
- URL conversion: 150K+ conversions/second

## Running Examples

### Prerequisites

```bash
# Clone the repository
git clone <repository-url>
cd trn-rust

# Build the project
cargo build --release
```

### Basic Examples

```bash
# Run basic usage examples
cargo run --example basic_usage

# Run advanced pattern examples
cargo run --example advanced_patterns

# Run performance tests (use release mode for accurate results)
cargo run --example performance_testing --release
```

### CLI Examples

```bash
# Show CLI help
cargo run --example cli_usage

# Parse and display TRN components
cargo run --example cli_usage -- parse "trn:user:alice:tool:github-api:v1.0"

# Validate TRN format and business rules
cargo run --example cli_usage -- validate "trn:user:alice:tool:github-api:v1.0"

# Convert TRN to different formats
cargo run --example cli_usage -- convert "trn:user:alice:tool:github-api:v1.0" json
cargo run --example cli_usage -- convert "trn:user:alice:tool:github-api:v1.0" url
cargo run --example cli_usage -- convert "trn:user:alice:tool:github-api:v1.0" yaml
```

### Creating Sample Data

Create a file `sample_trns.txt` with TRNs for batch processing:

```
# User tools
trn:user:alice:tool:github-api:v1.0
trn:user:bob:tool:user-flow:v2.0
trn:user:charlie:tool:data-processor:v1.5

# Organization tools
trn:org:company:tool:hr-system:v3.0
trn:org:startup:tool:customer-api:v2.5

# AI Platform models
trn:aiplatform:system:model:bert-base:latest
trn:aiplatform:system:dataset:training-data:v1.0
```

Then process it:

```bash
cargo run --example cli_usage -- batch sample_trns.txt
cargo run --example cli_usage -- analyze sample_trns.txt
```

## Integration Patterns

### Library Usage

```rust
use trn_rust::{Trn, TrnBuilder};

// Parse existing TRN
let trn = Trn::parse("trn:user:alice:tool:github-api:v1.0")?;

// Build new TRN
let trn = TrnBuilder::new()
    .platform(Platform::User)
    .scope("alice")
    .resource_type(ResourceType::Tool)
    .tool_type(ToolType::OpenApi)
    .instance_id("github-api")
    .version("v1.0")
    .build()?;

// Validate and convert
trn.validate()?;
let url = trn.to_url()?;
```

### Error Handling

```rust
match Trn::parse(trn_string) {
    Ok(trn) => {
        match trn.validate() {
            Ok(()) => println!("Valid TRN: {}", trn),
            Err(e) => eprintln!("Validation error: {}", e),
        }
    }
    Err(e) => eprintln!("Parse error: {}", e),
}
```

### Batch Processing

```rust
let trn_strings = vec![/* ... */];
let mut valid_trns = Vec::new();
let mut errors = Vec::new();

for trn_str in trn_strings {
    match Trn::parse(&trn_str) {
        Ok(trn) if trn.validate().is_ok() => valid_trns.push(trn),
        Ok(_) => errors.push(format!("Invalid TRN: {}", trn_str)),
        Err(e) => errors.push(format!("Parse error for {}: {}", trn_str, e)),
    }
}
```

## Performance Tips

1. **Use release mode** for performance testing:
   ```bash
   cargo run --example performance_testing --release
   ```

2. **Pre-validate** when processing large batches:
   ```rust
   let valid_trns: Vec<_> = trn_strings
       .iter()
       .filter_map(|s| Trn::parse(s).ok())
       .filter(|trn| trn.validate().is_ok())
       .collect();
   ```

3. **Reuse builders** for similar TRNs:
   ```rust
   let base_builder = TrnBuilder::new()
       .platform(Platform::User)
       .scope("alice")
       .resource_type(ResourceType::Tool);
   
   // Clone and customize for each tool
   let trn1 = base_builder.clone()
       .instance_id("tool1")
       .version("v1.0")
       .build()?;
   ```

4. **Use concurrent processing** for large datasets:
   ```rust
   use rayon::prelude::*;
   
   let results: Vec<_> = trn_strings
       .par_iter()
       .map(|s| Trn::parse(s))
       .collect();
   ```

## Best Practices

1. **Always validate** TRNs after parsing
2. **Handle errors gracefully** in production code
3. **Use semantic versioning** for version fields
4. **Cache validation results** for repeated operations
5. **Profile performance** for high-throughput scenarios
6. **Use appropriate platforms** (user vs org vs aiplatform)
7. **Follow naming conventions** for instance IDs and scopes

## Troubleshooting

### Common Issues

1. **Parse Errors**
   - Check TRN format against specification
   - Ensure all required components are present
   - Verify component character restrictions

2. **Validation Errors**
   - User/Org platforms require scope
   - AiPlatform should not have scope
   - Check reserved word usage

3. **Performance Issues**
   - Use release mode for benchmarks
   - Consider caching for repeated operations
   - Profile with appropriate tools

### Getting Help

- Check the main library documentation
- Review error messages for specific guidance
- Run examples with verbose output
- Compare with working examples in this directory

## Contributing

When adding new examples:

1. Focus on practical use cases
2. Include comprehensive error handling
3. Add performance considerations
4. Document expected outputs
5. Test with various TRN formats
6. Update this README with new examples 