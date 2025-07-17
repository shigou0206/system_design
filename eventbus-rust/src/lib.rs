//! # eventbus-rust: High-performance Event Bus Service
//! 
//! A distributed event bus service built on top of jsonrpc-rust and trn-rust,
//! providing pub/sub messaging, event persistence, rule-based triggers, and TRN integration.
//!
//! ## Key Features
//!
//! - **🚀 High Performance**: Built on async-first jsonrpc-rust foundation
//! - **📡 Pub/Sub Messaging**: Publish and subscribe to events with topic-based routing
//! - **🔄 Real-time Streaming**: Live event streams with backpressure control
//! - **💾 Event Persistence**: SQLite/PostgreSQL storage with event replay
//! - **⚡ Rule Engine**: Event-driven rule execution and tool invocation
//! - **🏷️ TRN Integration**: Deep integration with Tool Resource Name system
//! - **🔒 Type Safety**: Strongly typed events and comprehensive error handling
//! - **📊 Multi-Instance**: Support for multiple event bus instances

/// Core event bus types, traits and data structures
pub mod core;

/// Event storage and persistence implementations
pub mod storage;

/// Event routing and rule engine
pub mod routing;

/// JSON-RPC service implementation
pub mod service;

/// Configuration management
pub mod config;

/// Utilities and helpers
pub mod utils;

/// Prelude module for convenient imports
pub mod prelude {
    // Core types
    pub use crate::core::*;
    
    // Service types
    pub use crate::service::EventBusService;
    
    // Storage types
    // EventStorage is re-exported from core
    
    // Routing types
    pub use crate::routing::{EventRouter, RuleEngine};
    
    // Configuration
    pub use crate::config::{EventBusConfig, EventBusInstance};
    
    // Re-export from dependencies
    pub use jsonrpc_rust::prelude::*;
    
    // TRN integration (will be properly implemented later)
    #[cfg(feature = "trn-integration")]
    pub use trn_rust::*;
}

// Re-export the main modules (already defined above)

// Core types and traits
pub use core::{
    types::*,
    traits::*,
    error::*,
};

// Storage implementations
pub use storage::{
    create_storage,
    memory::MemoryStorage,
};

// Configuration
pub use config::{
    StorageConfig,
};

// Service types
pub use service::{
    EventBusService,
    ServiceConfig,
    ServiceMetrics,
    MultiBusConfig,
    MultiBusManager,
    GlobalConfig,
    RateLimitConfig,
    MetricsConfig,
    LoggingConfig,
    CombinedMetrics,
};

// Utility functions
pub use utils::{
    validate_trn,
    normalize_topic,
    extract_run_id,
    trn_matches,
};

/// Current version of the event bus
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Async entry point to run the event bus system
/// 
/// This function creates and starts multiple event bus instances based on the provided configuration.
/// It handles initialization, startup, and graceful shutdown of all buses.
/// 
/// # Arguments
/// * `config` - Multi-bus configuration defining which buses to create
/// 
/// # Returns
/// A handle to the running event bus system that can be used to interact with buses
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use eventbus_rust::{run_event_bus, service::MultiBusConfig};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Use default configuration (creates "workflows" and "global" buses)
///     let config = MultiBusConfig::default();
///     
///     // Start the event bus system
///     let mut bus_system = run_event_bus(config).await?;
///     
///     // The system is now running and ready to handle events
///     // You can interact with buses through the returned handle
///     
///     // Gracefully shutdown when done
///     bus_system.stop().await?;
///     
///     Ok(())
/// }
/// ```
pub async fn run_event_bus(
    config: service::MultiBusConfig,
) -> Result<service::MultiBusManager, Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging if configured
    if let Some(ref logging_config) = config.global.logging {
        init_logging(logging_config)?;
    }

    tracing::info!("Starting EventBus system with {} buses", config.buses.len());

    // Create the multi-bus manager
    let mut manager = service::MultiBusManager::new(config).await?;

    // Start all buses
    manager.start().await?;

    tracing::info!("EventBus system started successfully");

    Ok(manager)
}

/// Initialize logging based on configuration
fn init_logging(config: &service::LoggingConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false));

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Logging initialized with level: {}", config.level);
    Ok(())
}

/// Run event bus with default configuration
/// 
/// This is a convenience function that creates a default multi-bus configuration
/// and starts the event bus system.
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use eventbus_rust::run_event_bus_default;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let mut bus_system = run_event_bus_default().await?;
///     
///     // System is running with default configuration
///     // - "workflows" bus for workflow events
///     // - "global" bus for general events
///     
///     bus_system.stop().await?;
///     Ok(())
/// }
/// ```
pub async fn run_event_bus_default() -> Result<service::MultiBusManager, Box<dyn std::error::Error + Send + Sync>> {
    run_event_bus(service::MultiBusConfig::default()).await
}

/// Create a single event bus instance with default configuration
/// 
/// This is useful when you only need one event bus instance rather than a multi-bus setup.
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use eventbus_rust::create_single_event_bus;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let bus = create_single_event_bus().await?;
///     
///     // Use the bus for event operations
///     // let event = EventEnvelope::new("test.topic", json!({"message": "hello"}));
///     // bus.emit_event(event).await?;
///     
///     bus.shutdown().await?;
///     Ok(())
/// }
/// ```
pub async fn create_single_event_bus() -> Result<service::EventBusService, Box<dyn std::error::Error + Send + Sync>> {
    let config = service::ServiceConfig::default();
    service::EventBusService::with_config(config).await
}

/// Create a single event bus with custom configuration
/// 
/// # Arguments
/// * `config` - Service configuration for the event bus
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use eventbus_rust::{create_event_bus_with_config, service::ServiceConfig, storage::StorageConfig};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let config = ServiceConfig {
///         storage: StorageConfig::Memory { max_events: 10000 },
///         max_concurrent_emits: 50,
///         max_events_per_second: 500.0,
///         ..Default::default()
///     };
///     
///     let bus = create_event_bus_with_config(config).await?;
///     bus.shutdown().await?;
///     Ok(())
/// }
/// ```
pub async fn create_event_bus_with_config(
    config: service::ServiceConfig,
) -> Result<service::EventBusService, Box<dyn std::error::Error + Send + Sync>> {
    service::EventBusService::with_config(config).await
}
