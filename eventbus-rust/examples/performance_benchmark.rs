//! Performance benchmark for the event bus system
//! 
//! This example demonstrates the performance improvements with optimized
//! batch processing, storage backends, and monitoring capabilities.

use eventbus_rust::prelude::*;
use eventbus_rust::service::{ServiceConfig, AdvancedMetrics};
use eventbus_rust::storage::{StorageConfig, sqlite::SqliteStorage, postgres::PostgresStorage};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for performance monitoring
    tracing_subscriber::fmt::init();
    
    println!("🚀 EventBus Performance Benchmark Starting...");
    
    // Test configuration parameters
    let event_counts = vec![100, 1000, 10000, 50000];
    let batch_sizes = vec![1, 10, 100, 1000];
    
    // Run SQLite benchmarks
    println!("\n📊 SQLite Performance Tests");
    for &event_count in &event_counts {
        for &batch_size in &batch_sizes {
            if event_count >= batch_size {
                run_sqlite_benchmark(event_count, batch_size).await?;
            }
        }
    }
    
    // Run PostgreSQL benchmarks if available
    if std::env::var("POSTGRES_URL").is_ok() {
        println!("\n📊 PostgreSQL Performance Tests");
        for &event_count in &event_counts {
            for &batch_size in &batch_sizes {
                if event_count >= batch_size {
                    run_postgres_benchmark(event_count, batch_size).await?;
                }
            }
        }
    } else {
        println!("\n⚠️ PostgreSQL benchmarks skipped (POSTGRES_URL not set)");
    }
    
    // Run memory vs storage comparison
    println!("\n🔄 Memory vs Storage Comparison");
    run_storage_comparison().await?;
    
    // Run throughput stress test
    println!("\n⚡ Throughput Stress Test");
    run_throughput_stress_test().await?;
    
    // Run latency measurement test
    println!("\n⏱️ Latency Measurement Test");
    run_latency_test().await?;
    
    println!("\n✅ Performance benchmarks completed!");
    
    Ok(())
}

/// Run SQLite performance benchmark
async fn run_sqlite_benchmark(event_count: usize, batch_size: usize) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let db_path = format!("benchmark_sqlite_{}_{}.db", event_count, batch_size);
    
    // Clean up previous test database
    let _ = std::fs::remove_file(&db_path);
    
    // Create service with SQLite storage
    let config = ServiceConfig {
        instance_id: format!("sqlite_bench_{}_{}", event_count, batch_size),
        storage: StorageConfig::Sqlite { 
            database_url: format!("sqlite:{}", db_path) 
        },
        max_concurrent_emits: 100,
        batch_size: batch_size,
        enable_metrics: true,
        ..Default::default()
    };
    
    let service = EventBusService::with_config(config).await?;
    
    // Generate test events
    let events = generate_test_events(event_count);
    
    // Measure batch processing performance
    let start = Instant::now();
    
    if batch_size == 1 {
        // Individual emit
        for event in events {
            service.emit(event).await?;
        }
    } else {
        // Batch emit
        for batch in events.chunks(batch_size) {
            service.emit_batch(batch.to_vec()).await?;
        }
    }
    
    let duration = start.elapsed();
    
    // Measure query performance
    let query_start = Instant::now();
    let results = service.poll(EventQuery::new()).await?;
    let query_duration = query_start.elapsed();
    
    let events_per_second = event_count as f64 / duration.as_secs_f64();
    
    println!(
        "SQLite | Events: {:>6} | Batch: {:>4} | Insert: {:>8.2}ms | Query: {:>6.2}ms | Rate: {:>8.1} evt/s | Retrieved: {}",
        event_count,
        batch_size,
        duration.as_millis(),
        query_duration.as_millis(),
        events_per_second,
        results.len()
    );
    
    // Clean up
    service.shutdown().await?;
    let _ = std::fs::remove_file(&db_path);
    
    Ok(())
}

/// Run PostgreSQL performance benchmark
async fn run_postgres_benchmark(event_count: usize, batch_size: usize) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let postgres_url = std::env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgresql://localhost/eventbus_benchmark".to_string());
    
    // Create service with PostgreSQL storage
    let config = ServiceConfig {
        instance_id: format!("postgres_bench_{}_{}", event_count, batch_size),
        storage: StorageConfig::Postgres { 
            database_url: postgres_url,
            max_connections: 20,
            enable_partitioning: false,
        },
        max_concurrent_emits: 100,
        batch_size: batch_size,
        enable_metrics: true,
        ..Default::default()
    };
    
    let service = EventBusService::with_config(config).await?;
    
    // Generate test events
    let events = generate_test_events(event_count);
    
    // Measure batch processing performance
    let start = Instant::now();
    
    if batch_size == 1 {
        // Individual emit
        for event in events {
            service.emit(event).await?;
        }
    } else {
        // Batch emit
        for batch in events.chunks(batch_size) {
            service.emit_batch(batch.to_vec()).await?;
        }
    }
    
    let duration = start.elapsed();
    
    // Measure query performance
    let query_start = Instant::now();
    let results = service.poll(EventQuery::new()).await?;
    let query_duration = query_start.elapsed();
    
    let events_per_second = event_count as f64 / duration.as_secs_f64();
    
    println!(
        "PostgreSQL | Events: {:>6} | Batch: {:>4} | Insert: {:>8.2}ms | Query: {:>6.2}ms | Rate: {:>8.1} evt/s | Retrieved: {}",
        event_count,
        batch_size,
        duration.as_millis(),
        query_duration.as_millis(),
        events_per_second,
        results.len()
    );
    
    // Clean up
    service.shutdown().await?;
    
    Ok(())
}

/// Compare memory vs persistent storage performance
async fn run_storage_comparison() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let event_count = 10000;
    let batch_size = 100;
    
    // Memory storage test
    let memory_config = ServiceConfig {
        instance_id: "memory_comparison".to_string(),
        storage: StorageConfig::Memory { max_events: 50000 },
        batch_size,
        ..Default::default()
    };
    
    let memory_service = EventBusService::with_config(memory_config).await?;
    let events = generate_test_events(event_count);
    
    let memory_start = Instant::now();
    for batch in events.chunks(batch_size) {
        memory_service.emit_batch(batch.to_vec()).await?;
    }
    let memory_duration = memory_start.elapsed();
    memory_service.shutdown().await?;
    
    // SQLite storage test
    let sqlite_config = ServiceConfig {
        instance_id: "sqlite_comparison".to_string(),
        storage: StorageConfig::Sqlite { 
            database_url: "sqlite:comparison_test.db".to_string() 
        },
        batch_size,
        ..Default::default()
    };
    
    let sqlite_service = EventBusService::with_config(sqlite_config).await?;
    let events = generate_test_events(event_count);
    
    let sqlite_start = Instant::now();
    for batch in events.chunks(batch_size) {
        sqlite_service.emit_batch(batch.to_vec()).await?;
    }
    let sqlite_duration = sqlite_start.elapsed();
    sqlite_service.shutdown().await?;
    
    println!(
        "Storage Comparison | Events: {} | Memory: {:.2}ms | SQLite: {:.2}ms | Ratio: {:.2}x",
        event_count,
        memory_duration.as_millis(),
        sqlite_duration.as_millis(),
        sqlite_duration.as_secs_f64() / memory_duration.as_secs_f64()
    );
    
    // Clean up
    let _ = std::fs::remove_file("comparison_test.db");
    
    Ok(())
}

/// Run throughput stress test with high concurrency
async fn run_throughput_stress_test() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = ServiceConfig {
        instance_id: "throughput_stress".to_string(),
        storage: StorageConfig::Memory { max_events: 100000 },
        max_concurrent_emits: 1000,
        max_events_per_second: None, // Remove rate limiting
        batch_size: 50,
        ..Default::default()
    };
    
    let service = EventBusService::with_config(config).await?;
    
    // Run concurrent producers
    let num_producers = 10;
    let events_per_producer = 1000;
    
    let start = Instant::now();
    
    let mut handles = Vec::new();
    for i in 0..num_producers {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            for j in 0..events_per_producer {
                let event = EventEnvelope::new(
                    format!("stress.producer.{}", i),
                    json!({
                        "producer_id": i,
                        "event_num": j,
                        "timestamp": chrono::Utc::now().timestamp(),
                        "data": format!("test_data_{}", j)
                    })
                );
                
                if let Err(e) = service_clone.emit(event).await {
                    eprintln!("Failed to emit event: {}", e);
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all producers to complete
    for handle in handles {
        handle.await?;
    }
    
    let duration = start.elapsed();
    let total_events = num_producers * events_per_producer;
    let events_per_second = total_events as f64 / duration.as_secs_f64();
    
    println!(
        "Stress Test | Producers: {} | Events: {} | Duration: {:.2}s | Throughput: {:.1} evt/s",
        num_producers,
        total_events,
        duration.as_secs_f64(),
        events_per_second
    );
    
    service.shutdown().await?;
    
    Ok(())
}

/// Measure end-to-end latency with different event sizes
async fn run_latency_test() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = ServiceConfig {
        instance_id: "latency_test".to_string(),
        storage: StorageConfig::Memory { max_events: 10000 },
        max_concurrent_emits: 10,
        ..Default::default()
    };
    
    let service = EventBusService::with_config(config).await?;
    
    let payload_sizes = vec![
        ("Small", 100),   // ~100 bytes
        ("Medium", 1000), // ~1KB
        ("Large", 10000), // ~10KB
    ];
    
    for (size_name, payload_size) in payload_sizes {
        let mut latencies = Vec::new();
        
        for _ in 0..100 {
            let large_payload = "x".repeat(payload_size);
            let event = EventEnvelope::new(
                "latency.test",
                json!({
                    "data": large_payload,
                    "timestamp": chrono::Utc::now().timestamp()
                })
            );
            
            let start = Instant::now();
            service.emit(event).await?;
            let latency = start.elapsed();
            
            latencies.push(latency);
        }
        
        latencies.sort();
        let p50 = latencies[latencies.len() / 2];
        let p95 = latencies[(latencies.len() * 95) / 100];
        let p99 = latencies[(latencies.len() * 99) / 100];
        let avg: Duration = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        
        println!(
            "Latency {} | Avg: {:.2}ms | P50: {:.2}ms | P95: {:.2}ms | P99: {:.2}ms",
            size_name,
            avg.as_micros() as f64 / 1000.0,
            p50.as_micros() as f64 / 1000.0,
            p95.as_micros() as f64 / 1000.0,
            p99.as_micros() as f64 / 1000.0,
        );
    }
    
    service.shutdown().await?;
    
    Ok(())
}

/// Generate test events with varied content
fn generate_test_events(count: usize) -> Vec<EventEnvelope> {
    let topics = vec![
        "user.login", "user.logout", "order.created", "order.updated", 
        "payment.processed", "notification.sent", "system.health", "metrics.cpu"
    ];
    
    (0..count)
        .map(|i| {
            let topic = &topics[i % topics.len()];
            let mut event = EventEnvelope::new(
                *topic,
                json!({
                    "event_id": i,
                    "timestamp": chrono::Utc::now().timestamp(),
                    "user_id": format!("user_{}", i % 1000),
                    "session_id": format!("session_{}", i % 100),
                    "data": {
                        "action": format!("action_{}", i),
                        "details": format!("Event number {} for testing performance", i),
                        "metadata": {
                            "source": "benchmark",
                            "version": "1.0"
                        }
                    }
                })
            );
            
            // Add TRN information to some events
            if i % 3 == 0 {
                event.source_trn = Some(format!("trn:user:user_{}:service:benchmark:v1.0", i % 100));
                event.target_trn = Some(format!("trn:resource:{}:event_{}:v1.0", topic.replace('.', "_"), i));
            }
            
            // Add correlation IDs to some events
            if i % 5 == 0 {
                event.correlation_id = Some(format!("correlation_{}", i / 5));
            }
            
            // Vary priority
            event.priority = match i % 4 {
                0 => 50,   // Low
                1 => 100,  // Normal
                2 => 150,  // High
                3 => 200,  // Critical
                _ => 100,
            };
            
            event
        })
        .collect()
} 