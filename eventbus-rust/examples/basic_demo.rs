//! Basic EventBus demonstration
//! 
//! This example shows how to:
//! 1. Create an event bus service
//! 2. Emit events
//! 3. Query events
//! 4. Work with TRN-enabled events

use eventbus_rust::prelude::*;
use eventbus_rust::service::ServiceConfig;
use serde_json::json;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 EventBus Demo Starting...");
    
    // Create event bus service
    let mut config = ServiceConfig::default();
    config.instance_id = "demo".to_string();
    config.max_memory_events = 100;
    config.enable_rules = false;
    config.allowed_sources = vec!["*".to_string()]; // Allow all sources
    
    let service = EventBusService::new(config);
    
    // Create some test events
    println!("\n📤 Emitting events...");
    
    // Basic event
    let event1 = EventEnvelope::new("user.login", json!({
        "user_id": "alice",
        "timestamp": "2024-01-15T10:00:00Z"
    }));
    
    service.emit(event1).await?;
    println!("✅ Emitted user.login event");
    
    // Event with TRN information
    let event2 = EventEnvelope::new("order.created", json!({
        "order_id": "order_123",
        "customer_id": "alice",
        "amount": 99.99
    }))
    .set_trn(
        Some("trn:user:alice:service:order-service".to_string()),
        Some("trn:resource:order:order_123".to_string())
    )
    .with_correlation_id("req-456")
    .with_priority(200);
    
    service.emit(event2).await?;
    println!("✅ Emitted order.created event with TRN");
    
    // Event with metadata
    let event3 = EventEnvelope::new("analytics.page_view", json!({
        "page": "/dashboard",
        "user_agent": "Mozilla/5.0...",
        "ip": "192.168.1.1"
    }))
    .with_metadata(json!({
        "source": "web",
        "version": "1.0"
    }));
    
    service.emit(event3).await?;
    println!("✅ Emitted analytics.page_view event with metadata");
    
    // Query events
    println!("\n📥 Querying events...");
    
    // Query all events
    let all_events = service.poll(EventQuery::new()).await?;
    println!("📊 Found {} total events", all_events.len());
    
    for (i, event) in all_events.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, event.topic, 
                 event.payload.get("user_id")
                     .or(event.payload.get("order_id"))
                     .or(event.payload.get("page"))
                     .unwrap_or(&json!("N/A")));
    }
    
    // Query specific topic
    let user_events = service.poll(
        EventQuery::new().with_topic("user.*")
    ).await?;
    println!("\n🔍 Found {} user events", user_events.len());
    
    // Query by TRN
    let mut trn_query = EventQuery::new();
    trn_query.source_trn = Some("trn:user:alice:service:order-service".to_string());
    let trn_events = service.poll(trn_query).await?;
    println!("🏷️ Found {} events with TRN source", trn_events.len());
    
    // Get service statistics
    println!("\n📈 Service Statistics:");
    let stats = service.get_stats().await?;
    println!("  - Events processed: {}", stats.events_processed);
    println!("  - Topic count: {}", stats.topic_count);
    println!("  - Active subscriptions: {}", stats.active_subscriptions);
    
    // List all topics
    let topics = service.list_topics().await?;
    println!("\n📋 Available topics:");
    for topic in topics {
        println!("  - {}", topic);
    }
    
    println!("\n✅ Demo completed successfully!");
    
    Ok(())
} 