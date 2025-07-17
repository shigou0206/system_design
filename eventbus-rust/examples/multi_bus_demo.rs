//! Multi-bus EventBus demonstration
//! 
//! This example shows how to:
//! 1. Set up multiple event bus instances
//! 2. Use the EventEnvelopeBuilder for easy event creation
//! 3. Handle rule-based event routing
//! 4. Manage different storage backends
//! 5. Monitor combined metrics

use eventbus_rust::{
    run_event_bus, 
    service::{MultiBusConfig, ServiceConfig, GlobalConfig, LoggingConfig},
    config::StorageConfig, 
    core::{EventEnvelopeBuilder, EventPriority, traits::EventBus},
};
use serde_json::json;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Multi-Bus EventBus Demo");

    // Create custom multi-bus configuration
    let config = create_custom_config();
    
    // Start the event bus system
    let bus_system = run_event_bus(config).await?;
    
    println!("✅ Multi-bus system started with buses: {:?}", bus_system.bus_names());

    // Demo 1: Basic event emission to different buses
    demo_basic_events(&bus_system).await?;
    
    // Demo 2: Using EventEnvelopeBuilder for different event types
    demo_event_builder(&bus_system).await?;
    
    // Demo 3: Subscribe to events from different buses
    demo_subscriptions(&bus_system).await?;
    
    // Demo 4: Monitor combined metrics
    demo_metrics(&bus_system).await?;
    
    println!("🎯 Demo completed successfully!");

    // Gracefully shutdown
    bus_system.stop().await?;
    
    Ok(())
}



/// Create a custom multi-bus configuration
fn create_custom_config() -> MultiBusConfig {
    let mut buses = HashMap::new();
    
    // Workflow bus with SQLite persistence
    buses.insert(
        "workflows".to_string(),
        ServiceConfig {
            instance_id: "workflows".to_string(),
            storage: StorageConfig::Sqlite { 
                path: "workflows.db".to_string() 
            },
            max_concurrent_emits: 50,
            max_events_per_second: Some(500),
            event_buffer_size: 5000,
            subscriber_buffer_size: 500,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
            ..ServiceConfig::default()
        }
    );
    
    // User events bus with memory storage (high performance)
    buses.insert(
        "users".to_string(),
        ServiceConfig {
            instance_id: "users".to_string(),
            storage: StorageConfig::Memory,
            max_concurrent_emits: 100,
            max_events_per_second: Some(1000),
            event_buffer_size: 10000,
            subscriber_buffer_size: 1000,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
            ..ServiceConfig::default()
        }
    );
    
    // System events bus with PostgreSQL (if available)
    buses.insert(
        "system".to_string(),
        ServiceConfig {
            instance_id: "system".to_string(),
            storage: StorageConfig::Memory, // Fallback to memory
            max_concurrent_emits: 25,
            max_events_per_second: Some(200),
            event_buffer_size: 2000,
            subscriber_buffer_size: 200,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
            ..ServiceConfig::default()
        }
    );

    MultiBusConfig {
        buses,
        global: GlobalConfig {
            rate_limit: None,
            metrics: None,
            logging: Some(LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                log_events: true,
                log_performance: true,
            }),
            shutdown_timeout_secs: 60,
        },
        default_bus: Some("workflows".to_string()),
    }
}

/// Demo basic event emission to different buses
async fn demo_basic_events(bus_system: &eventbus_rust::service::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📡 Demo 1: Basic Event Emission");
    
    // Get bus instances
    let workflows_bus = bus_system.get_bus("workflows").unwrap();
    let users_bus = bus_system.get_bus("users").unwrap();
    let system_bus = bus_system.get_bus("system").unwrap();
    
    // Emit workflow events
    let workflow_event = EventEnvelopeBuilder::new()
        .topic("workflow.started")
        .source_trn("trn:user:admin:service:workflow_manager")
        .metadata(json!({
            "workflow_id": "wf_12345",
            "user_id": "user_789",
            "type": "data_processing"
        }))
        .priority(EventPriority::High)
        .build()?;
    
    workflows_bus.emit(workflow_event).await?;
    println!("  ✅ Workflow event emitted");
    
    // Emit user events
    let user_event = EventEnvelopeBuilder::new()
        .topic("user.login")
        .source_trn("trn:user:alice:service:auth")
        .metadata(json!({
            "user_id": "alice",
            "session_id": "sess_456",
            "ip_address": "192.168.1.100"
        }))
        .build()?;
    
    users_bus.emit(user_event).await?;
    println!("  ✅ User event emitted");
    
    // Emit system events
    let system_event = EventEnvelopeBuilder::new()
        .topic("system.health")
        .source_trn("trn:system:monitor:service:health_check")
        .metadata(json!({
            "cpu_usage": 45.2,
            "memory_usage": 67.8,
            "disk_usage": 23.1
        }))
        .priority(EventPriority::Low)
        .build()?;
    
    system_bus.emit(system_event).await?;
    println!("  ✅ System event emitted");
    
    Ok(())
}

/// Demo EventEnvelopeBuilder for creating different event types
async fn demo_event_builder(bus_system: &eventbus_rust::service::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🏗️  Demo 2: EventEnvelopeBuilder");
    
    let workflows_bus = bus_system.get_bus("workflows").unwrap();
    
    // Business event with correlation
    let business_event = EventEnvelopeBuilder::new()
        .topic("order.processed")
        .source_trn("trn:user:customer:service:orders")
        .correlation_id("order_corr_123")
        .metadata(json!({
            "order_id": "ord_98765",
            "customer_id": "cust_555",
            "amount": 129.99,
            "items": [
                {"sku": "ITEM001", "quantity": 2},
                {"sku": "ITEM002", "quantity": 1}
            ]
        }))
        .priority(EventPriority::High)
        .build()?;
    
    workflows_bus.emit(business_event).await?;
    println!("  ✅ Business event with correlation emitted");
    
    // Error event with high priority
    let error_event = EventEnvelopeBuilder::new()
        .topic("system.error")
        .source_trn("trn:system:database:service:postgres")
        .metadata(json!({
            "error_code": "DB_CONNECTION_TIMEOUT",
            "error_message": "Failed to connect to database after 30 seconds",
            "retry_count": 3,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .priority(EventPriority::Critical)
        .build()?;
    
    workflows_bus.emit(error_event).await?;
    println!("  ✅ Critical error event emitted");
    
    // Batch of related events
    for i in 1..=5 {
        let batch_event = EventEnvelopeBuilder::new()
            .topic("data.batch_processed")
            .source_trn("trn:system:etl:service:data_processor")
            .correlation_id("batch_2024_001")
            .metadata(json!({
                "batch_id": format!("batch_2024_001_part_{}", i),
                "records_processed": 1000 * i,
                "processing_time_ms": 250 + (i * 50),
                "part_number": i,
                "total_parts": 5
            }))
            .build()?;
        
        workflows_bus.emit(batch_event).await?;
    }
    println!("  ✅ Batch processing events emitted");
    
    Ok(())
}

/// Demo subscription to events from different buses
async fn demo_subscriptions(bus_system: &eventbus_rust::service::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📻 Demo 3: Event Subscriptions");
    
    let workflows_bus = bus_system.get_bus("workflows").unwrap();
    let users_bus = bus_system.get_bus("users").unwrap();
    
    // Subscribe to workflow events
    let topic = "workflow.*";
    println!("  📡 Subscribing to '{}'", topic);
    
    // Note: In a real application, you would handle the subscription stream
    // For demo purposes, we'll just show that subscription is possible
    let _subscription_result = workflows_bus.subscribe(&topic).await;
    
    // Emit some test events for the subscription
    let workflow_complete_event = EventEnvelopeBuilder::new()
        .topic("workflow.completed")
        .source_trn("trn:user:system:service:workflow_engine")
        .metadata(json!({
            "workflow_id": "wf_12345",
            "execution_time_ms": 45000,
            "status": "success",
            "output_size_bytes": 2048
        }))
        .build()?;
    
    workflows_bus.emit(workflow_complete_event).await?;
    println!("  ✅ Workflow completion event emitted for subscription");
    
    // Subscribe to user events
    let user_topic = "user.login";
    println!("  📡 Subscribing to '{}'", user_topic);
    let _user_subscription = users_bus.subscribe(&user_topic).await;
    
    let user_login_event = EventEnvelopeBuilder::new()
        .topic("user.login")
        .source_trn("trn:user:bob:service:mobile_app")
        .metadata(json!({
            "user_id": "bob",
            "device_type": "mobile",
            "app_version": "2.1.4",
            "login_method": "oauth_google"
        }))
        .build()?;
    
    users_bus.emit(user_login_event).await?;
    println!("  ✅ User login event emitted for subscription");
    
    Ok(())
}

/// Demo metrics monitoring across multiple buses
async fn demo_metrics(bus_system: &eventbus_rust::service::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📊 Demo 4: Multi-Bus Metrics");
    
    // Get initial metrics
    let initial_metrics = bus_system.get_combined_metrics().await?;
    
    // Emit events to each bus
    for bus_name in ["workflows", "users", "system"] {
        if let Some(bus) = bus_system.get_bus(bus_name) {
            for i in 1..=10 {
                let event = EventEnvelopeBuilder::new()
                    .topic(&format!("{}.test_event", bus_name))
                    .source_trn(&format!("trn:test:{}:service:demo", bus_name))
                    .metadata(json!({
                        "test_id": i,
                        "bus": bus_name
                    }))
                    .build()?;
                
                bus.emit(event).await?;
            }
        }
    }
    
    // Wait a moment for processing
    sleep(Duration::from_millis(100)).await;
    
    // Get updated metrics
    let updated_metrics = bus_system.get_combined_metrics().await?;
    
    println!("  📈 Metrics Comparison:");
    println!("     Initial events processed: {}", 
             initial_metrics.total_events_processed());
    println!("     Updated events processed: {}", 
             updated_metrics.total_events_processed());
    
    for bus_name in ["workflows", "users", "system"] {
        if let Some(metrics) = updated_metrics.get_bus_metrics(bus_name) {
            println!("  📋 {}: {} events processed", bus_name, metrics.events_processed());
        }
    }
    
    Ok(())
} 