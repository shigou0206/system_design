/// Advanced TRN Usage Example
/// 
/// This example demonstrates advanced features and real-world usage patterns:
/// - Batch operations and validation reports
/// - Performance testing
/// - Pattern matching scenarios
/// - Error handling patterns

use std::time::Instant;
use trn_rust::{Trn, TrnBuilder, generate_validation_report};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 TRN Rust Library - Advanced Usage Example");
    println!("==============================================\n");

    // 1. Batch Operations and Validation
    println!("1. Batch Operations and Validation:");
    let batch_trns = vec![
        "trn:user:alice:tool:weather-api:v1.0".to_string(),
        "trn:org:openai:model:gpt-4:v1.0".to_string(),
        "trn:aiplatform:huggingface:dataset:common-crawl:latest".to_string(),
        "invalid-trn-format".to_string(),  // This will fail
        "trn:user:bob:pipeline:data-prep:v2.1".to_string(),
        "trn:org:anthropic:model:claude-3:v3.0".to_string(),
    ];
    
    println!("   Validating {} TRNs...", batch_trns.len());
    let report = generate_validation_report(&batch_trns);
    
    println!("   Validation Report:");
    println!("     Total: {}", report.total);
    println!("     Valid: {}", report.valid);
    println!("     Invalid: {}", report.invalid);
    println!("     Duration: {}ms", report.stats.duration_ms);
    println!("     Rate: {:.2} TRNs/second", report.stats.rate_per_second);
    
    if !report.errors.is_empty() {
        println!("   Errors found:");
        for (i, error) in report.errors.iter().enumerate() {
            println!("     {}. {}", i + 1, error);
        }
    }
    println!();

    // 2. Performance Testing
    println!("2. Performance Testing:");
    
    // Generate test data
    let test_data: Vec<String> = (0..5000)
        .map(|i| format!("trn:user:user{}:tool:api{}:v1.0", i, i))
        .collect();
    
    // Benchmark parsing
    let start = Instant::now();
    let mut parsed_count = 0;
    for trn_str in &test_data {
        if Trn::parse(trn_str).is_ok() {
            parsed_count += 1;
        }
    }
    let parse_duration = start.elapsed();
    
    println!("   Parsed {} TRNs in {:?}", parsed_count, parse_duration);
    println!("   Parse rate: {:.2} TRNs/ms", parsed_count as f64 / parse_duration.as_millis() as f64);
    
    // Benchmark validation
    let valid_trns: Vec<Trn> = test_data.iter()
        .filter_map(|s| Trn::parse(s).ok())
        .collect();
    
    let start = Instant::now();
    let valid_count = valid_trns.iter()
        .filter(|trn| trn.is_valid())
        .count();
    let validation_duration = start.elapsed();
    
    println!("   Validated {} TRNs in {:?}", valid_trns.len(), validation_duration);
    println!("   Valid TRNs: {}", valid_count);
    println!();

    // 3. Pattern Matching Scenarios
    println!("3. Pattern Matching Scenarios:");
    
    let api_resources = create_sample_resources()?;
    println!("   Created {} sample resources", api_resources.len());
    
    let patterns = vec![
        ("trn:user:*:*:*:*", "All user resources"),
        ("trn:*:*:model:*:*", "All models"),
        ("trn:*:*:*:*:v1.0", "Version 1.0 resources"),
        ("trn:org:*:*:*:*", "All organization resources"),
    ];
    
    for (pattern, description) in patterns {
        let matches: Vec<_> = api_resources.iter()
            .filter(|trn| trn.matches_pattern(pattern))
            .collect();
        
        println!("   Pattern '{}' ({}):", pattern, description);
        for trn in matches.iter().take(3) {  // Show first 3 matches
            println!("     - {}", trn);
        }
        if matches.len() > 3 {
            println!("     ... and {} more", matches.len() - 3);
        }
        println!("     Total matches: {}", matches.len());
        println!();
    }

    // 4. Builder Pattern Scenarios
    println!("4. Advanced Builder Patterns:");
    
    // Template-based building
    let template = TrnBuilder::new()
        .platform("org")
        .scope("mycompany");
    
    let services = vec![
        ("auth-service", "v2.0"),
        ("user-service", "v1.5"),
        ("payment-service", "v3.0"),
    ];
    
    println!("   Creating microservices using template:");
    for (service, version) in services {
        let service_trn = template.clone()
            .resource_type("tool")
            .resource_id(service)
            .version(version)
            .build()?;
        
        println!("     - {}", service_trn);
    }
    println!();

    // 5. Error Handling Patterns
    println!("5. Error Handling Patterns:");
    
    let problematic_inputs = vec![
        "",
        "not-a-trn",
        "trn:user:alice",
        "trn:user:alice:tool:myapi:v1.0:extra",
    ];
    
    for input in problematic_inputs {
        match Trn::parse(input) {
            Ok(trn) => {
                match trn.validate() {
                    Ok(_) => println!("   ✓ '{}' -> Valid TRN", input),
                    Err(e) => println!("   ⚠ '{}' -> Parsed but invalid: {}", input, e),
                }
            }
            Err(e) => println!("   ✗ '{}' -> Parse error: {}", input, e),
        }
    }
    println!();

    // 6. Real-world Application: Resource Discovery
    println!("6. Real-world Application: Resource Discovery");
    
    // Find all resources from a specific organization
    let org_resources: Vec<_> = api_resources.iter()
        .filter(|trn| trn.platform() == "org")
        .collect();
    
    println!("   Organization resources: {}", org_resources.len());
    
    // Find all latest versions
    let latest_resources: Vec<_> = api_resources.iter()
        .filter(|trn| trn.version() == "latest")
        .collect();
    
    println!("   Latest version resources: {}", latest_resources.len());
    
    // Find compatible resources with a base TRN
    let base_trn = Trn::new("user", "alice", "tool", "weather-api", "v1.0")?;
    let compatible: Vec<_> = api_resources.iter()
        .filter(|trn| base_trn.is_compatible_with(trn))
        .collect();
    
    println!("   Resources compatible with {}: {}", base_trn, compatible.len());
    
    for trn in compatible.iter().take(3) {
        println!("     - {}", trn);
    }
    println!();

    // 7. URL Conversion Workflows
    println!("7. URL Conversion Workflows:");
    
    let sample_trn = &api_resources[0];
    
    // Convert to different URL formats
    let trn_url = sample_trn.to_url()?;
    println!("   TRN URL: {}", trn_url);
    
    let http_url = sample_trn.to_http_url("https://registry.example.com")?;
    println!("   HTTP URL: {}", http_url);
    
    // Test roundtrip conversion
    let from_url = trn_rust::url_to_trn(&trn_url)?;
    println!("   Roundtrip successful: {}", sample_trn.to_string() == from_url.to_string());
    println!();

    println!("✨ Advanced example completed successfully!");
    Ok(())
}

fn create_sample_resources() -> Result<Vec<Trn>, Box<dyn std::error::Error>> {
    let mut resources = Vec::new();
    
    // User tools
    resources.push(Trn::new("user", "alice", "tool", "weather-api", "v1.0")?);
    resources.push(Trn::new("user", "alice", "tool", "weather-api", "v2.0")?);
    resources.push(Trn::new("user", "alice", "tool", "weather-api", "latest")?);
    resources.push(Trn::new("user", "bob", "tool", "chat-bot", "v1.0")?);
    
    // Organization models
    resources.push(Trn::new("org", "openai", "model", "gpt-3", "v1.0")?);
    resources.push(Trn::new("org", "openai", "model", "gpt-4", "v1.0")?);
    resources.push(Trn::new("org", "anthropic", "model", "claude-2", "v2.0")?);
    resources.push(Trn::new("org", "anthropic", "model", "claude-3", "v3.0")?);
    
    // Platform datasets
    resources.push(Trn::new("aiplatform", "huggingface", "dataset", "common-crawl", "latest")?);
    resources.push(Trn::new("aiplatform", "kaggle", "dataset", "titanic", "v1.0")?);
    
    // Pipelines
    resources.push(Trn::new("user", "alice", "pipeline", "ml-training", "v1.0")?);
    resources.push(Trn::new("org", "netflix", "pipeline", "recommendation", "v3.0")?);
    
    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sample_resources() {
        let resources = create_sample_resources().unwrap();
        assert!(!resources.is_empty());
        
        // All should be valid
        for resource in resources {
            assert!(resource.is_valid());
        }
    }
    
    #[test]
    fn test_example_runs() {
        // Test that the example code runs without panicking
        main().expect("Example should run successfully");
    }
} 