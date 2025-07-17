//! Performance benchmark for the event bus system
//! 
//! This example demonstrates the performance improvements with optimized
//! batch processing, storage backends, and monitoring capabilities.

use eventbus_rust::prelude::*;
use eventbus_rust::service::ServiceConfig;
use eventbus_rust::config::StorageConfig;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    
    // Run PostgreSQL benchmarks (if available)
    println!("\n🐘 PostgreSQL Performance Tests");
    if let Ok(postgres_url) = std::env::var("DATABASE_URL") {
        run_postgres_benchmark(&postgres_url).await?;
    } else {
        println!("Skipping PostgreSQL tests - DATABASE_URL not set");
    }
    
    // Run comparative tests
    println!("\n⚡ Storage Comparison Tests");
    run_storage_comparison_test().await?;
    
    // Run stress test
    println!("\n🔥 High-Load Stress Test");
    run_stress_test().await?;
    
    // Run latency tests
    println!("\n⏱️  Latency Performance Tests");
    run_latency_tests().await?;
    
    println!("\n✅ All benchmark tests completed!");
    Ok(())
}

/// Run SQLite-specific performance benchmarks
async fn run_sqlite_benchmark(event_count: usize, batch_size: usize) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join(format!("bench_{}_{}_{}.db", event_count, batch_size, chrono::Utc::now().timestamp()));
    
    let config = ServiceConfig {
        instance_id: format!("sqlite-bench-{}-{}", event_count, batch_size),
        batch_size,
        storage: StorageConfig::Sqlite { 
            path: db_path.to_string_lossy().to_string()
        },
        enable_metrics: true,
        ..ServiceConfig::default()
    };
    
    let start = Instant::now();
    let service = EventBusService::with_config(config).await?;
    let setup_time = start.elapsed();
    
    // Emit events
    let emit_start = Instant::now();
    for i in 0..event_count {
        let event = EventEnvelopeBuilder::new()
            .topic("com.example.benchmark.test")
            .source_trn("trn:test:source:benchmark")
            .metadata(json!({
                "batch": batch_size,
                "index": i,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
            .build()?;
        
        service.emit(event).await?;
        
        if i % 1000 == 0 && i > 0 {
            print!(".");
        }
    }
    let emit_time = emit_start.elapsed();
    
    // Query events
    let query_start = Instant::now();
    let query = EventQuery::new()
        .with_topic("com.example.benchmark.test");
    let events = service.poll(query).await?;
    let query_time = query_start.elapsed();
    
    let total_time = start.elapsed();
    let events_per_second = (event_count as f64) / emit_time.as_secs_f64();
    
    println!("\n📈 SQLite Benchmark Results (events: {}, batch: {})", event_count, batch_size);
    println!("  Setup time: {:?}", setup_time);
    println!("  Emit time: {:?} ({:.2} events/sec)", emit_time, events_per_second);
    println!("  Query time: {:?} ({} events retrieved)", query_time, events.len());
    println!("  Total time: {:?}", total_time);
    
    Ok(())
}

/// Run PostgreSQL-specific performance benchmarks
async fn run_postgres_benchmark(postgres_url: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ServiceConfig {
        instance_id: "postgres-bench".to_string(),
        batch_size: 100,
        storage: StorageConfig::Postgres { 
            url: postgres_url.to_string(),
            pool_size: 20,
        },
        enable_metrics: true,
        ..ServiceConfig::default()
    };
    
    let start = Instant::now();
    let service = EventBusService::with_config(config).await?;
    let setup_time = start.elapsed();
    
    let event_count = 10000;
    
    // Emit events
    let emit_start = Instant::now();
    for i in 0..event_count {
        let event = EventEnvelopeBuilder::new()
            .topic("com.example.postgres.test")
            .source_trn("trn:test:source:postgres")
            .metadata(json!({
                "index": i,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
            .build()?;
        
        service.emit(event).await?;
    }
    let emit_time = emit_start.elapsed();
    
    let events_per_second = (event_count as f64) / emit_time.as_secs_f64();
    
    println!("\n🐘 PostgreSQL Benchmark Results");
    println!("  Setup time: {:?}", setup_time);
    println!("  Emit time: {:?} ({:.2} events/sec)", emit_time, events_per_second);
    
    Ok(())
}

/// Compare performance across different storage backends
async fn run_storage_comparison_test() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event_count = 5000;
    
    // Test memory storage
    let memory_config = ServiceConfig {
        instance_id: "memory-comparison".to_string(),
        storage: StorageConfig::Memory,
        enable_metrics: true,
        ..ServiceConfig::default()
    };
    
    let memory_service = EventBusService::with_config(memory_config).await?;
    
    let memory_start = Instant::now();
    for i in 0..event_count {
        let event = EventEnvelopeBuilder::new()
            .topic("com.example.comparison.test")
            .source_trn("trn:test:source:comparison")
            .metadata(json!({"index": i}))
            .build()?;
        memory_service.emit(event).await?;
    }
    let memory_time = memory_start.elapsed();
    
    // Test SQLite storage
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("comparison_test.db");
    
    let sqlite_config = ServiceConfig {
        instance_id: "sqlite-comparison".to_string(),
        storage: StorageConfig::Sqlite { 
            path: db_path.to_string_lossy().to_string()
        },
        enable_metrics: true,
        ..ServiceConfig::default()
    };
    
    let sqlite_service = EventBusService::with_config(sqlite_config).await?;
    
    let sqlite_start = Instant::now();
    for i in 0..event_count {
        let event = EventEnvelopeBuilder::new()
            .topic("com.example.comparison.test")
            .source_trn("trn:test:source:comparison")
            .metadata(json!({"index": i}))
            .build()?;
        sqlite_service.emit(event).await?;
    }
    let sqlite_time = sqlite_start.elapsed();
    
    let memory_eps = (event_count as f64) / memory_time.as_secs_f64();
    let sqlite_eps = (event_count as f64) / sqlite_time.as_secs_f64();
    
    println!("\n⚡ Storage Comparison Results ({} events)", event_count);
    println!("  Memory: {:?} ({:.2} events/sec)", memory_time, memory_eps);
    println!("  SQLite: {:?} ({:.2} events/sec)", sqlite_time, sqlite_eps);
    println!("  Speed ratio: {:.2}x (memory vs sqlite)", memory_eps / sqlite_eps);
    
    Ok(())
}

/// Run high-load stress test
async fn run_stress_test() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ServiceConfig {
        instance_id: "stress-test".to_string(),
        max_concurrent_emits: 200,
        storage: StorageConfig::Memory,
        enable_metrics: true,
        ..ServiceConfig::default()
    };
    
    let service = EventBusService::with_config(config).await?;
    let service = Arc::new(service);
    
    let start = Instant::now();
    let concurrent_tasks = 50;
    let events_per_task = 1000;
    
    let mut handles = Vec::new();
    
    for task_id in 0..concurrent_tasks {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            for i in 0..events_per_task {
                let event = EventEnvelopeBuilder::new()
                    .topic("com.example.stress.test")
                    .source_trn(&format!("trn:test:source:stress:task:{}", task_id))
                    .metadata(json!({
                        "task_id": task_id,
                        "event_id": i
                    }))
                    .build();
                
                match event {
                    Ok(event) => {
                        if let Err(e) = service_clone.emit(event).await {
                            eprintln!("Error in task {}: {}", task_id, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error building event in task {}: {}", task_id, e);
                    }
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    let total_time = start.elapsed();
    let total_events = concurrent_tasks * events_per_task;
    let total_eps = (total_events as f64) / total_time.as_secs_f64();
    
    println!("\n🔥 Stress Test Results");
    println!("  Concurrent tasks: {}", concurrent_tasks);
    println!("  Events per task: {}", events_per_task);
    println!("  Total events: {}", total_events);
    println!("  Total time: {:?}", total_time);
    println!("  Throughput: {:.2} events/sec", total_eps);
    
    Ok(())
}

/// Test latency performance with different payload sizes
async fn run_latency_tests() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ServiceConfig {
        instance_id: "latency-test".to_string(),
        storage: StorageConfig::Memory,
        enable_metrics: true,
        ..ServiceConfig::default()
    };
    
    let service = EventBusService::with_config(config).await?;
    
    let payload_sizes = vec![100, 1000, 10000, 100000]; // bytes
    let iterations = 100;
    
    for &payload_size in &payload_sizes {
        let payload = "x".repeat(payload_size);
        let mut latencies = Vec::new();
        
        for _ in 0..iterations {
            let start = Instant::now();
            
            let event = EventEnvelopeBuilder::new()
                .topic("com.example.latency.test")
                .source_trn("trn:test:source:latency")
                .metadata(json!({"payload": payload}))
                .build()?;
            
            service.emit(event).await?;
            
            let latency = start.elapsed();
            latencies.push(latency);
        }
        
        // Calculate statistics
        latencies.sort();
        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let p50_latency = latencies[latencies.len() / 2];
        let p95_latency = latencies[(latencies.len() as f64 * 0.95) as usize];
        let p99_latency = latencies[(latencies.len() as f64 * 0.99) as usize];
        
        println!("\n⏱️  Latency Test Results (payload: {} bytes)", payload_size);
        println!("  Average: {:?}", avg_latency);
        println!("  P50: {:?}", p50_latency);
        println!("  P95: {:?}", p95_latency);
        println!("  P99: {:?}", p99_latency);
    }
    
    Ok(())
} 