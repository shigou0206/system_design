//! EventBus JSON-RPC Server Binary
//! 
//! This binary starts an EventBus JSON-RPC server that exposes EventBus
//! functionality over the network.

use std::sync::Arc;
use std::env;
use std::process;
use tokio;

use eventbus_rust::prelude::*;
use eventbus_rust::config::{EventBusConfig, StorageConfig};
use eventbus_rust::service::{EventBusService, ServiceConfig};
use eventbus_rust::jsonrpc::EventBusRpcServer;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let listen_addr = args.get(1)
        .map(|s| s.as_str())
        .unwrap_or("127.0.0.1:8080");

    println!("🚀 Starting EventBus JSON-RPC Server");
    println!("📡 Listen address: {}", listen_addr);

    // Create EventBus service configuration
    let service_config = ServiceConfig {
        instance_id: "eventbus-server".to_string(),
        max_memory_events: 10000,
        max_events_per_second: Some(1000),
        enable_metrics: true,
        ..Default::default()
    };

    // Create EventBus service
    println!("📦 Initializing EventBus service...");
    let event_bus_service = Arc::new(EventBusService::new(service_config));

    // Create JSON-RPC server
    println!("🔧 Setting up JSON-RPC server...");
    let rpc_server = EventBusRpcServer::new(Arc::clone(&event_bus_service));

    // Start the server
    println!("🌐 Starting JSON-RPC server on {}...", listen_addr);
    
    // Handle graceful shutdown
    tokio::select! {
        result = rpc_server.start(listen_addr) => {
            match result {
                Ok(_) => println!("✅ EventBus JSON-RPC server started successfully"),
                Err(e) => {
                    eprintln!("❌ Failed to start EventBus JSON-RPC server: {}", e);
                    process::exit(1);
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n🛑 Received shutdown signal, stopping server...");
        }
    }

    println!("👋 EventBus JSON-RPC server shutdown complete");
    Ok(())
}

fn print_usage() {
    println!("Usage: eventbus-server [listen_address]");
    println!();
    println!("Arguments:");
    println!("  listen_address    Address to listen on (default: 127.0.0.1:8080)");
    println!();
    println!("Examples:");
    println!("  eventbus-server                    # Listen on 127.0.0.1:8080");
    println!("  eventbus-server 0.0.0.0:9000      # Listen on 0.0.0.0:9000");
    println!("  eventbus-server localhost:8080     # Listen on localhost:8080");
} 