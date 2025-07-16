/// Basic TRN Usage Example
/// 
/// This example demonstrates the fundamental operations of the TRN library:
/// - Creating TRNs using different methods
/// - Parsing and validating TRN strings
/// - Converting between different formats
/// - Basic pattern matching

use trn_rust::{Trn, TrnBuilder, is_valid_trn, url_to_trn};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TRN Rust Library - Basic Usage Example");
    println!("==========================================\n");

    // 1. Creating TRNs using the constructor
    println!("1. Creating TRNs using constructor:");
    let trn1 = Trn::new("user", "alice", "tool", "myapi", "v1.0")?;
    println!("   Created: {}", trn1);
    println!("   Platform: {}", trn1.platform());
    println!("   Scope: {}", trn1.scope());
    println!("   Resource Type: {}", trn1.resource_type());
    println!("   Resource ID: {}", trn1.resource_id());
    println!("   Version: {}", trn1.version());
    println!();

    // 2. Creating TRNs using the builder pattern
    println!("2. Creating TRNs using builder pattern:");
    let trn2 = TrnBuilder::new()
        .platform("org")
        .scope("company")
        .resource_type("model")
        .resource_id("bert-large")
        .version("v2.1")
        .build()?;
    println!("   Created: {}", trn2);
    println!();

    // 3. Parsing TRN strings
    println!("3. Parsing TRN from string:");
    let trn_string = "trn:aiplatform:system:dataset:training:latest";
    let trn3 = Trn::parse(trn_string)?;
    println!("   Parsed '{}' to: {}", trn_string, trn3);
    println!();

    // 4. Validation
    println!("4. TRN Validation:");
    let valid_trns = vec![
        "trn:user:alice:tool:myapi:v1.0",
        "trn:org:company:model:bert:v2.1",
        "trn:aiplatform:system:dataset:training:latest",
    ];
    
    let invalid_trns = vec![
        "invalid-format",
        "trn:user:alice",           // Too few components
        "trn:user:alice:tool:myapi:v1.0:extra", // Too many components
        "trn::alice:tool:myapi:v1.0", // Empty platform
    ];
    
    println!("   Valid TRNs:");
    for trn_str in &valid_trns {
        let is_valid = is_valid_trn(trn_str);
        println!("     {} -> {}", trn_str, if is_valid { "✓ Valid" } else { "✗ Invalid" });
    }
    
    println!("   Invalid TRNs:");
    for trn_str in &invalid_trns {
        let is_valid = is_valid_trn(trn_str);
        println!("     {} -> {}", trn_str, if is_valid { "✓ Valid" } else { "✗ Invalid" });
    }
    println!();

    // 5. URL Conversion
    println!("5. URL Conversion:");
    let trn_url = trn1.to_url()?;
    println!("   TRN URL: {}", trn_url);
    
    let http_url = trn1.to_http_url("https://platform.example.com")?;
    println!("   HTTP URL: {}", http_url);
    
    // Convert back from URL
    let from_url = url_to_trn(&trn_url)?;
    println!("   Converted back: {}", from_url);
    println!();

    // 6. Pattern Matching
    println!("6. Pattern Matching:");
    let test_trns = vec![
        Trn::new("user", "alice", "tool", "api1", "v1.0")?,
        Trn::new("user", "alice", "tool", "api2", "v1.0")?,
        Trn::new("user", "bob", "tool", "api1", "v1.0")?,
        Trn::new("org", "company", "model", "bert", "v2.0")?,
    ];
    
    let patterns = vec![
        ("trn:user:alice:*:*:*", "Alice's resources"),
        ("trn:*:*:tool:*:*", "All tools"),
        ("trn:*:*:*:*:v1.0", "Version 1.0 resources"),
    ];
    
    for (pattern, description) in patterns {
        let matches: Vec<_> = test_trns.iter()
            .filter(|trn| trn.matches_pattern(pattern))
            .collect();
        
        println!("   Pattern '{}' ({}):", pattern, description);
        for matched_trn in matches {
            println!("     - {}", matched_trn);
        }
    }
    println!();

    // 7. TRN Comparison and Compatibility
    println!("7. TRN Comparison and Compatibility:");
    let base_trn = Trn::new("user", "alice", "tool", "myapi", "v1.0")?;
    let version_variant = Trn::new("user", "alice", "tool", "myapi", "v2.0")?;
    let different_trn = Trn::new("org", "alice", "tool", "myapi", "v1.0")?;
    
    println!("   Base TRN: {}", base_trn);
    println!("   Version variant: {}", version_variant);
    println!("   Different platform: {}", different_trn);
    
    println!("   Compatibility checks:");
    println!("     Base ↔ Version variant: {}", 
             if base_trn.is_compatible_with(&version_variant) { "✓ Compatible" } else { "✗ Incompatible" });
    println!("     Base ↔ Different platform: {}", 
             if base_trn.is_compatible_with(&different_trn) { "✓ Compatible" } else { "✗ Incompatible" });
    
    // Base TRN (version becomes wildcard)
    let base = base_trn.base_trn();
    println!("   Base TRN (wildcard version): {}", base);
    println!();

    // 8. JSON Serialization
    println!("8. JSON Serialization:");
    let json_string = trn1.to_json()?;
    println!("   JSON: {}", json_string);
    
    let from_json = Trn::from_json(&json_string)?;
    println!("   Deserialized: {}", from_json);
    println!();

    // 9. Real-world Examples
    println!("9. Real-world TRN Examples:");
    let examples = vec![
        ("trn:user:alice:tool:weather-api:v1.0", "Alice's weather API tool"),
        ("trn:org:openai:model:gpt-4:v1.0", "OpenAI's GPT-4 model"),
        ("trn:aiplatform:huggingface:dataset:common-crawl:latest", "HuggingFace's Common Crawl dataset"),
        ("trn:user:bob:pipeline:data-preprocessing:v2.1", "Bob's data preprocessing pipeline"),
        ("trn:org:anthropic:model:claude-3:v3.0", "Anthropic's Claude-3 model"),
    ];
    
    for (trn_str, description) in examples {
        let trn = Trn::parse(trn_str)?;
        println!("   {} -> {}", description, trn);
    }
    println!();

    println!("✨ Example completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_runs() {
        // Test that the example code runs without panicking
        main().expect("Example should run successfully");
    }
} 