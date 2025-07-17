//! EventBus JSON-RPC Client Demo
//!
//! This example demonstrates how to use the EventBus JSON-RPC client
//! to interact with a remote EventBus service.

use std::time::Duration;
use tokio::time::sleep;

use eventbus_rust::prelude::*;
use eventbus_rust::core::{EventEnvelope, EventQuery, EventEnvelopeBuilder};
use eventbus_rust::jsonrpc::{EventBusRpcClient, connect_to_eventbus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 EventBus JSON-RPC Client Demo");
    
    // Connect to the EventBus JSON-RPC server
    let server_addr = "127.0.0.1:8080";
    println!("📡 Connecting to EventBus server at {}", server_addr);
    
    let client = connect_to_eventbus(server_addr).await
        .map_err(|e| format!("Failed to connect to EventBus server: {}", e))?;
    
    println!("✅ Connected to EventBus server");

    // Example 1: Emit a single event
    println!("\n📤 Example 1: Emitting a single event");
    let event = EventEnvelopeBuilder::new()
        .topic("user.login")
        .source_trn()
        .metadata(serde_json::json!({
            "user_id": "user123",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "ip_address": "192.168.1.100"
        }))
        .build()?;

    match client.emit(event).await {
        Ok(success) => println!("✅ Event emitted successfully: {}", success),
        Err(e) => println!("❌ Failed to emit event: {}", e),
    }

    // Example 2: Emit multiple events in batch
    println!("\n📦 Example 2: Emitting events in batch");
    let events = vec![
        EventEnvelopeBuilder::new()
            .topic("system.startup")
            .source_trn()
            .metadata(serde_json::json!({"service": "api-gateway"}))
            .build()?,
        EventEnvelopeBuilder::new()
            .topic("system.startup")
            .source_trn()
            .metadata(serde_json::json!({"service": "user-service"}))
            .build()?,
        EventEnvelopeBuilder::new()
            .topic("system.startup")
            .source_trn()
            .metadata(serde_json::json!({"service": "order-service"}))
            .build()?,
    ];

    match client.emit_batch(events).await {
        Ok(count) => println!("✅ Batch emitted successfully: {} events", count),
        Err(e) => println!("❌ Failed to emit batch: {}", e),
    }

    // Example 3: List available topics
    println!("\n📋 Example 3: Listing available topics");
    match client.list_topics().await {
        Ok(topics) => {
            if topics.is_empty() {
                println!("📭 No topics available");
            } else {
                println!("📚 Available topics:");
                for topic in topics {
                    println!("  - {}", topic);
                }
            }
        },
        Err(e) => println!("❌ Failed to list topics: {}", e),
    }

    // Example 4: Query events
    println!("\n🔍 Example 4: Querying events");
    let query = EventQuery::new().with_topic("user.login");
    
    match client.poll(query).await {
        Ok(events) => {
            println!("📬 Found {} events:", events.len());
            for (i, event) in events.iter().enumerate() {
                println!("  {}. Topic: {}, ID: {}", i + 1, event.topic, event.id);
            }
        },
        Err(e) => println!("❌ Failed to query events: {}", e),
    }

    // Example 5: Subscribe to a topic
    println!("\n🔔 Example 5: Subscribing to events");
    let subscription = match client.subscribe("user.login", Some("demo-client".to_string())).await {
        Ok(handle) => {
            println!("✅ Subscribed to topic 'user.login' with ID: {}", handle.subscription_id);
            handle
        },
        Err(e) => {
            println!("❌ Failed to subscribe: {}", e);
            return Ok(());
        }
    };

    // Example 6: Poll for subscription events
    println!("\n📨 Example 6: Polling subscription for events");
    for i in 1..=3 {
        println!("  Polling attempt {}...", i);
        
        match client.get_subscription_events(&subscription, Some(10), Some(2000)).await {
            Ok(events) => {
                if events.is_empty() {
                    println!("    📭 No new events");
                } else {
                    println!("    📬 Received {} events:", events.len());
                    for event in events {
                        println!("      - Topic: {}, ID: {}", event.topic, event.id);
                    }
                }
            },
            Err(e) => println!("    ❌ Failed to get subscription events: {}", e),
        }
        
        if i < 3 {
            sleep(Duration::from_secs(1)).await;
        }
    }

    // Example 7: Get server statistics
    println!("\n📊 Example 7: Getting server statistics");
    match client.get_stats().await {
        Ok(stats) => {
            println!("📈 EventBus Server Statistics:");
            println!("  - Total events: {}", stats.total_events);
            println!("  - Active topics: {}", stats.active_topics);
            println!("  - Active subscriptions: {}", stats.active_subscriptions);
            println!("  - Events per second: {:.2}", stats.events_per_second);
            println!("  - Uptime: {} seconds", stats.uptime_seconds);
            println!("  - Memory usage: {} events, ~{} bytes", 
                stats.memory_usage.events_in_memory, 
                stats.memory_usage.estimated_bytes
            );
        },
        Err(e) => println!("❌ Failed to get stats: {}", e),
    }

    // Example 8: Unsubscribe
    println!("\n🔕 Example 8: Unsubscribing from topic");
    match client.unsubscribe(&subscription).await {
        Ok(success) => {
            if success {
                println!("✅ Successfully unsubscribed from topic");
            } else {
                println!("⚠️ Unsubscribe returned false");
            }
        },
        Err(e) => println!("❌ Failed to unsubscribe: {}", e),
    }

    // Example 9: List active subscriptions
    println!("\n📋 Example 9: Listing active subscriptions");
    let subscriptions = client.list_subscriptions().await;
    if subscriptions.is_empty() {
        println!("📭 No active subscriptions");
    } else {
        println!("📚 Active subscriptions:");
        for sub in subscriptions {
            println!("  - ID: {}, Topic: {}", sub.subscription_id, sub.topic);
        }
    }

    println!("\n🎉 EventBus JSON-RPC Client Demo completed!");
    Ok(())
} 