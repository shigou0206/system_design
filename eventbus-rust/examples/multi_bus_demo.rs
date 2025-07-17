//! Multi-bus EventBus demonstration
//! 
//! This example shows how to:
//! 1. Set up multiple event bus instances
//! 2. Use the EventEnvelopeBuilder for easy event creation
//! 3. Handle rule-based event routing
//! 4. Manage different storage backends
//! 5. Monitor combined metrics

use eventbus_rust::{
    run_event_bus, MultiBusConfig, ServiceConfig, GlobalConfig,
    StorageConfig, EventEnvelopeBuilder, Rule, RuleAction,
    EventPriority, LoggingConfig,
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
    let mut bus_system = run_event_bus(config).await?;
    
    println!("✅ Multi-bus system started with buses: {:?}", bus_system.bus_names());

    // Demo 1: Basic event emission to different buses
    demo_basic_events(&bus_system).await?;
    
    // Demo 2: Using EventEnvelopeBuilder for different event types
    demo_event_builder(&bus_system).await?;
    
    // Demo 3: Subscribe to events from different buses
    demo_subscriptions(&bus_system).await?;
    
    // Demo 4: Monitor combined metrics
    demo_metrics(&bus_system).await?;
    
    // Allow some time for event processing
    sleep(Duration::from_secs(2)).await;
    
    // Get final metrics
    let metrics = bus_system.get_combined_metrics().await?;
    println!("\n📊 Final Combined Metrics:");
    println!("Total events processed: {}", metrics.totals.events_processed);
    println!("Total EPS: {:.2}", metrics.totals.events_per_second);
    println!("Total active subscriptions: {}", metrics.totals.active_subscriptions);
    
    for (bus_name, bus_metrics) in &metrics.buses {
        println!("  {}: {} events, {:.2} EPS", 
                 bus_name, 
                 bus_metrics.events_processed, 
                 bus_metrics.events_per_second);
    }
    
    // Graceful shutdown
    println!("\n🛑 Shutting down event bus system...");
    bus_system.stop().await?;
    println!("✅ System shut down successfully");
    
    Ok(())
}

fn create_custom_config() -> MultiBusConfig {
    let mut buses = HashMap::new();
    
    // Workflow bus with SQLite persistence
    buses.insert(
        "workflows".to_string(),
        ServiceConfig {
            storage: StorageConfig::Sqlite { 
                database_url: "workflows.db".to_string() 
            },
            max_concurrent_emits: 50,
            max_events_per_second: 500.0,
            event_buffer_size: 5000,
            subscriber_buffer_size: 500,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
        }
    );
    
    // User events bus with memory storage (high performance)
    buses.insert(
        "users".to_string(),
        ServiceConfig {
            storage: StorageConfig::Memory { max_events: 20000 },
            max_concurrent_emits: 100,
            max_events_per_second: 1000.0,
            event_buffer_size: 10000,
            subscriber_buffer_size: 1000,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
        }
    );
    
    // System events bus with PostgreSQL (if available)
    buses.insert(
        "system".to_string(),
        ServiceConfig {
            storage: StorageConfig::Memory { max_events: 10000 }, // Fallback to memory
            max_concurrent_emits: 25,
            max_events_per_second: 200.0,
            event_buffer_size: 2000,
            subscriber_buffer_size: 200,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
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

async fn demo_basic_events(bus_system: &eventbus_rust::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🔥 Demo 1: Basic Event Emission");
    
    // Emit to default bus (workflows)
    let event = EventEnvelopeBuilder::new()
        .topic("workflow.start")
        .payload_json(json!({
            "workflow_id": "wf_123",
            "user_id": "user_456",
            "action": "deployment"
        }))
        .metadata("source", "api")
        .priority(EventPriority::High)
        .now()
        .build()?;
    
    bus_system.emit(event).await?;
    println!("✅ Emitted workflow event to default bus");
    
    // Emit to specific bus (users)
    let user_event = EventEnvelopeBuilder::user_event(
        "alice",
        "login",
        json!({
            "timestamp": chrono::Utc::now(),
            "ip": "192.168.1.100",
            "user_agent": "Mozilla/5.0..."
        })
    )?.build()?;
    
    bus_system.emit_to_bus("users", user_event).await?;
    println!("✅ Emitted user event to users bus");
    
    // Emit system event
    let system_event = EventEnvelopeBuilder::system_event(
        "health",
        json!({
            "status": "healthy",
            "memory_usage": 85.2,
            "cpu_usage": 23.1
        })
    )?.build()?;
    
    bus_system.emit_to_bus("system", system_event).await?;
    println!("✅ Emitted system event to system bus");
    
    Ok(())
}

async fn demo_event_builder(bus_system: &eventbus_rust::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🏗️ Demo 2: EventEnvelopeBuilder Usage");
    
    // Workflow event with full metadata
    let workflow_event = EventEnvelopeBuilder::workflow_event(
        "wf_789",
        "step.completed",
        json!({
            "step_id": "step_001",
            "duration_ms": 1234,
            "result": "success"
        })
    )?
    .source_trn("trn:user:alice:workflow:data-pipeline:v1.0")
    .target_trn("trn:system:scheduler:tool:task-manager:v2.0")
    .correlation_id("corr_xyz_789")
    .sequence_number(42)
    .metadata("environment", "production")
    .metadata("region", "us-west-2")
    .build()?;
    
    bus_system.emit_to_bus("workflows", workflow_event).await?;
    println!("✅ Emitted complex workflow event");
    
    // Error event
    let error_event = EventEnvelopeBuilder::error_event(
        "database",
        json!({
            "error_type": "connection_timeout",
            "message": "Failed to connect to database after 30s",
            "stack_trace": "...",
            "recovery_action": "retry_with_backoff"
        })
    )?
    .correlation_id("corr_error_001")
    .critical_priority()
    .build()?;
    
    bus_system.emit_to_bus("system", error_event).await?;
    println!("✅ Emitted critical error event");
    
    // Metric event
    let metric_event = EventEnvelopeBuilder::metric_event(
        "response_time",
        json!({
            "value": 156.7,
            "unit": "ms",
            "endpoint": "/api/users",
            "status_code": 200
        })
    )?
    .metadata("service", "user-api")
    .low_priority()
    .build()?;
    
    bus_system.emit_to_bus("system", metric_event).await?;
    println!("✅ Emitted metric event");
    
    Ok(())
}

async fn demo_subscriptions(bus_system: &eventbus_rust::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📡 Demo 3: Event Subscriptions");
    
    // Subscribe to workflow events
    let mut workflow_receiver = bus_system.subscribe_to_bus("workflows", "workflow.*".to_string()).await?;
    
    // Subscribe to user events  
    let mut user_receiver = bus_system.subscribe_to_bus("users", "user.*".to_string()).await?;
    
    // Subscribe to system errors
    let mut error_receiver = bus_system.subscribe_to_bus("system", "error.*".to_string()).await?;
    
    println!("✅ Set up subscriptions to multiple buses");
    
    // Emit some test events
    let test_events = vec![
        ("workflows", EventEnvelopeBuilder::new()
            .topic("workflow.test")
            .payload_json(json!({"test": "workflow"}))
            .build()?),
        ("users", EventEnvelopeBuilder::new()
            .topic("user.test") 
            .payload_json(json!({"test": "user"}))
            .build()?),
        ("system", EventEnvelopeBuilder::new()
            .topic("error.test")
            .payload_json(json!({"test": "error"}))
            .build()?),
    ];
    
    for (bus_name, event) in test_events {
        bus_system.emit_to_bus(bus_name, event).await?;
    }
    
    // Try to receive events (with timeout)
    println!("🔍 Listening for events...");
    
    let timeout_duration = Duration::from_millis(500);
    
    tokio::select! {
        result = workflow_receiver.recv() => {
            if let Ok(event) = result {
                println!("📨 Received workflow event: {}", event.topic);
            }
        }
        _ = sleep(timeout_duration) => {}
    }
    
    tokio::select! {
        result = user_receiver.recv() => {
            if let Ok(event) = result {
                println!("📨 Received user event: {}", event.topic);
            }
        }
        _ = sleep(timeout_duration) => {}
    }
    
    tokio::select! {
        result = error_receiver.recv() => {
            if let Ok(event) = result {
                println!("📨 Received error event: {}", event.topic);
            }
        }
        _ = sleep(timeout_duration) => {}
    }
    
    Ok(())
}

async fn demo_metrics(bus_system: &eventbus_rust::MultiBusManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n📊 Demo 4: Metrics Monitoring");
    
    // Get initial metrics
    let initial_metrics = bus_system.get_combined_metrics().await?;
    println!("📈 Initial metrics collected at: {}", initial_metrics.collected_at);
    
    // Emit a burst of events to see metrics change
    for i in 0..10 {
        let event = EventEnvelopeBuilder::new()
            .topic(format!("test.batch.{}", i))
            .payload_json(json!({"batch_id": i, "timestamp": chrono::Utc::now()}))
            .build()?;
        
        // Distribute across buses
        let bus_name = match i % 3 {
            0 => "workflows",
            1 => "users", 
            _ => "system",
        };
        
        bus_system.emit_to_bus(bus_name, event).await?;
    }
    
    // Wait a bit for processing
    sleep(Duration::from_millis(100)).await;
    
    // Get updated metrics
    let updated_metrics = bus_system.get_combined_metrics().await?;
    println!("📈 Updated metrics collected at: {}", updated_metrics.collected_at);
    println!("📊 Metrics comparison:");
    println!("  Events processed: {} -> {}", 
             initial_metrics.totals.events_processed,
             updated_metrics.totals.events_processed);
    
    // Show per-bus breakdown
    for (bus_name, metrics) in &updated_metrics.buses {
        println!("  📋 {}: {} events processed", bus_name, metrics.events_processed);
    }
    
    Ok(())
} 