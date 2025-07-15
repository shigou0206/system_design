//! TRN CLI tool
//!
//! Command-line interface for TRN operations including validation,
//! parsing, conversion, and pattern matching.

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
use trn_rust::{parse, validate, ValidationReport};

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
            handle_parse(&trn, &format)?;
        }
        Commands::Validate { trns, stdin, format } => {
            if stdin {
                handle_validate_stdin(&format)?;
            } else {
                handle_validate(&trns, &format)?;
            }
        }
        Commands::Convert { trn, base_url, target } => {
            handle_convert(&trn, base_url.as_deref(), &target)?;
        }
        Commands::Info { trn, format } => {
            handle_info(&trn, &format)?;
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn handle_parse(trn_str: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    match parse(trn_str) {
        Ok(trn) => {
            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&trn)?;
                    println!("{}", json);
                }
                "yaml" => {
                    let yaml = serde_yaml::to_string(&trn)?;
                    println!("{}", yaml);
                }
                _ => {
                    println!("✓ Valid TRN: {}", trn);
                    println!("  Platform: {}", trn.platform());
                    if let Some(scope) = trn.scope() {
                        println!("  Scope: {}", scope);
                    }
                    println!("  Resource Type: {}", trn.resource_type());
                    println!("  Type: {}", trn.type_());
                    if let Some(subtype) = trn.subtype() {
                        println!("  Subtype: {}", subtype);
                    }
                    println!("  Instance ID: {}", trn.instance_id());
                    println!("  Version: {}", trn.version());
                    if let Some(tag) = trn.tag() {
                        println!("  Tag: {}", tag);
                    }
                    if let Some(hash) = trn.hash() {
                        println!("  Hash: {}", hash);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Invalid TRN: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

#[cfg(feature = "cli")]
fn handle_validate(trns: &[String], format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let report = trn_rust::batch_validate(trns);
    
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        "yaml" => {
            let yaml = serde_yaml::to_string(&report)?;
            println!("{}", yaml);
        }
        _ => {
            println!("Validation Report:");
            println!("  Total: {}", report.total);
            println!("  Valid: {} ({}%)", report.valid, 
                     (report.valid as f64 / report.total as f64 * 100.0) as u32);
            println!("  Invalid: {} ({}%)", report.invalid, 
                     (report.invalid as f64 / report.total as f64 * 100.0) as u32);
            println!("  Duration: {}ms", report.stats.duration_ms);
            println!("  Rate: {:.1} TRNs/sec", report.stats.rate_per_second);
            
            if !report.errors.is_empty() {
                println!("\nErrors:");
                for error in &report.errors {
                    println!("  ✗ {}: {}", error.trn, error.error);
                    if let Some(suggestion) = &error.suggestion {
                        println!("    💡 {}", suggestion);
                    }
                }
            }
        }
    }
    
    if report.invalid > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

#[cfg(feature = "cli")]
fn handle_validate_stdin(format: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, BufRead};
    
    let stdin = io::stdin();
    let trns: Vec<String> = stdin.lock()
        .lines()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect();
    
    handle_validate(&trns, format)
}

#[cfg(feature = "cli")]
fn handle_convert(trn_str: &str, base_url: Option<&str>, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let trn = parse(trn_str)?;
    
    match target {
        "trn-url" => {
            let url = trn.to_url()?;
            println!("{}", url);
        }
        "http-url" => {
            if let Some(base) = base_url {
                let url = trn.to_http_url(base)?;
                println!("{}", url);
            } else {
                eprintln!("Error: --base-url is required for http-url conversion");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Error: Invalid target format. Use 'trn-url' or 'http-url'");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(feature = "cli")]
fn handle_info(trn_str: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let trn = parse(trn_str)?;
    
    match format {
        "json" => {
            let info = serde_json::json!({
                "trn": trn.to_string(),
                "components": {
                    "platform": trn.platform(),
                    "scope": trn.scope(),
                    "resource_type": trn.resource_type(),
                    "type": trn.type_(),
                    "subtype": trn.subtype(),
                    "instance_id": trn.instance_id(),
                    "version": trn.version(),
                    "tag": trn.tag(),
                    "hash": trn.hash(),
                },
                "urls": {
                    "trn_url": trn.to_url().ok(),
                    "base_trn": trn.base_trn().to_string(),
                },
                "validation": {
                    "is_valid": trn.is_valid(),
                }
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        "yaml" => {
            let info = serde_yaml::to_value(&trn)?;
            println!("{}", serde_yaml::to_string(&info)?);
        }
        _ => {
            println!("TRN Information:");
            println!("  Full TRN: {}", trn);
            println!("  Base TRN: {}", trn.base_trn());
            if let Ok(url) = trn.to_url() {
                println!("  TRN URL: {}", url);
            }
            println!("  Valid: {}", if trn.is_valid() { "✓" } else { "✗" });
            
            println!("\nComponents:");
            println!("  Platform: {}", trn.platform());
            if let Some(scope) = trn.scope() {
                println!("  Scope: {}", scope);
            }
            println!("  Resource Type: {}", trn.resource_type());
            println!("  Type: {}", trn.type_());
            if let Some(subtype) = trn.subtype() {
                println!("  Subtype: {}", subtype);
            }
            println!("  Instance ID: {}", trn.instance_id());
            println!("  Version: {}", trn.version());
            if let Some(tag) = trn.tag() {
                println!("  Tag: {}", tag);
            }
            if let Some(hash) = trn.hash() {
                println!("  Hash: {}", hash);
            }
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature is not enabled. Please compile with --features cli");
    std::process::exit(1);
} 