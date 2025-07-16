//! TRN CLI tool
//!
//! Command-line interface for TRN operations including validation,
//! parsing, conversion, and pattern matching.

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
use trn_rust::{Trn, generate_validation_report};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "trn")]
#[command(about = "TRN (Tool Resource Name) CLI utility")]
#[command(long_about = "A command-line tool for parsing, validating, and manipulating Tool Resource Names (TRN)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Parse and validate a single TRN
    Parse {
        /// TRN string to parse
        trn: String,
        /// Output format (json, yaml, or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
    /// Validate one or more TRNs
    Validate {
        /// TRN strings to validate
        trns: Vec<String>,
        /// Read from stdin instead of arguments
        #[arg(short, long)]
        stdin: bool,
        /// Output format (json, yaml, or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
    /// Convert TRN to URL format
    Convert {
        /// TRN string to convert
        trn: String,
        /// Base URL for HTTP conversion
        #[arg(short, long)]
        base_url: Option<String>,
        /// Output format (trn-url or http-url)
        #[arg(short, long, default_value = "trn-url")]
        target: String,
    },
    /// Show TRN components
    Info {
        /// TRN string to analyze
        trn: String,
        /// Output format (json, yaml, or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { trn, format } => {
            parse_command(&trn, &format)?;
        }
        Commands::Validate { trns, stdin, format } => {
            validate_command(trns, stdin, &format)?;
        }
        Commands::Convert { trn, base_url, target } => {
            convert_command(&trn, base_url.as_deref(), &target)?;
        }
        Commands::Info { trn, format } => {
            info_command(&trn, &format)?;
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Please build with --features cli");
    std::process::exit(1);
}

#[cfg(feature = "cli")]
fn parse_command(trn_str: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    match Trn::parse(trn_str) {
        Ok(trn) => {
            match format {
                "json" => {
                    let output = serde_json::json!({
                        "success": true,
                        "trn": trn.to_string(),
                        "components": {
                            "platform": trn.platform(),
                            "scope": trn.scope(),
                            "resource_type": trn.resource_type(),
                            "resource_id": trn.resource_id(),
                            "version": trn.version(),
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "yaml" => {
                    #[cfg(feature = "cli")]
                    {
                        let output = serde_yaml::to_string(&trn)?;
                        println!("{}", output);
                    }
                    #[cfg(not(feature = "cli"))]
                    {
                        eprintln!("YAML output requires CLI feature");
                        return Err("YAML not supported".into());
                    }
                }
                _ => {
                    println!("✅ TRN is valid");
                    println!("Platform: {}", trn.platform());
                    println!("Scope: {}", trn.scope());
                    println!("Resource Type: {}", trn.resource_type());
                    println!("Resource ID: {}", trn.resource_id());
                    println!("Version: {}", trn.version());
                    
                    // Display URLs
                    if let Ok(trn_url) = trn.to_url() {
                        println!("TRN URL: {}", trn_url);
                    }
                    
                    println!("Base TRN: {}", trn.base_trn().to_string());
                }
            }
        }
        Err(e) => {
            match format {
                "json" => {
                    let output = serde_json::json!({
                        "success": false,
                        "error": e.to_string(),
                        "input": trn_str
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                _ => {
                    eprintln!("❌ Invalid TRN: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "cli")]
fn validate_command(
    mut trns: Vec<String>,
    stdin: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if stdin {
        use std::io::{self, Read};
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        trns.extend(
            buffer
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string()),
        );
    }

    let report = generate_validation_report(&trns);

    match format {
        "json" => {
            let output = serde_json::json!({
                "total": report.total,
                "valid": report.valid,
                "invalid": report.invalid,
                "errors": report.errors,
                "duration_ms": report.stats.duration_ms,
                "rate_per_second": report.stats.rate_per_second
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("📊 Validation Report");
            println!("Total TRNs: {}", report.total);
            println!("✅ Valid: {}", report.valid);
            println!("❌ Invalid: {}", report.invalid);
            println!("⏱️  Duration: {}ms", report.stats.duration_ms);
            println!("🚀 Rate: {:.2} TRNs/second", report.stats.rate_per_second);

            if !report.errors.is_empty() {
                println!("\n🔍 Errors:");
                for (i, error) in report.errors.iter().enumerate() {
                    println!("  {}. {}", i + 1, error);
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn convert_command(
    trn_str: &str,
    base_url: Option<&str>,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let trn = Trn::parse(trn_str)?;

    match target {
        "trn-url" => {
            let url = trn.to_url()?;
            println!("{}", url);
        }
        "http-url" => {
            let base = base_url.unwrap_or("https://trn.example.com");
            let url = trn.to_http_url(base)?;
            println!("{}", url);
        }
        _ => {
            eprintln!("Invalid target format. Use 'trn-url' or 'http-url'");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn info_command(trn_str: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let trn = Trn::parse(trn_str)?;
    
    match format {
        "json" => {
            let info = serde_json::json!({
                "trn": trn.to_string(),
                "components": {
                    "platform": trn.platform(),
                    "scope": trn.scope(),
                    "resource_type": trn.resource_type(),
                    "resource_id": trn.resource_id(),
                    "version": trn.version(),
                },
                "urls": {
                    "trn_url": trn.to_url().ok(),
                    "base_trn": trn.base_trn().to_string(),
                },
                "validation": {
                    "is_valid": true,
                }
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        "yaml" => {
            #[cfg(feature = "cli")]
            {
                let info = serde_yaml::to_value(&trn)?;
                println!("{}", serde_yaml::to_string(&info)?);
            }
            #[cfg(not(feature = "cli"))]
            {
                eprintln!("YAML output requires CLI feature");
                return Err("YAML not supported".into());
            }
        }
        _ => {
            println!("🔍 TRN Information");
            println!("TRN: {}", trn.to_string());
            println!();
            println!("📋 Components:");
            println!("  Platform: {}", trn.platform());
            println!("  Scope: {}", trn.scope());
            println!("  Resource Type: {}", trn.resource_type());
            println!("  Resource ID: {}", trn.resource_id());
            println!("  Version: {}", trn.version());
            println!();
            
            if let Ok(trn_url) = trn.to_url() {
                println!("🔗 URLs:");
                println!("  TRN URL: {}", trn_url);
                if let Ok(http_url) = trn.to_http_url("https://platform.example.com") {
                    println!("  HTTP URL: {}", http_url);
                }
            }
            
            println!();
            println!("🎯 Base TRN: {}", trn.base_trn().to_string());
            println!("✅ Status: Valid");
        }
    }
    
    Ok(())
} 