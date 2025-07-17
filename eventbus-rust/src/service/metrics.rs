//! Advanced metrics and monitoring for the event bus system
//! 
//! This module provides comprehensive metrics collection including:
//! - Throughput measurements
//! - Latency distributions  
//! - Error rates and types
//! - Resource utilization
//! - Custom business metrics

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Advanced metrics collector for the event bus
#[derive(Debug)]
pub struct AdvancedMetrics {
    /// Event processing metrics
    events_emitted: AtomicU64,
    events_processed: AtomicU64,
    events_failed: AtomicU64,
    
    /// Throughput tracking
    throughput_tracker: Arc<RwLock<ThroughputTracker>>,
    
    /// Latency measurements
    latency_tracker: Arc<RwLock<LatencyTracker>>,
    
    /// Error tracking
    error_tracker: Arc<RwLock<ErrorTracker>>,
    
    /// Topic-specific metrics
    topic_metrics: Arc<RwLock<HashMap<String, TopicMetrics>>>,
    
    /// Rule engine metrics
    rule_metrics: Arc<RwLock<RuleMetrics>>,
    
    /// Storage metrics
    storage_metrics: Arc<RwLock<StorageMetrics>>,
}

/// Throughput tracking with time windows
#[derive(Debug)]
pub struct ThroughputTracker {
    /// Events per second in different time windows
    current_minute: TimedCounter,
    last_minute: TimedCounter,
    current_hour: TimedCounter,
    
    /// Peak throughput records
    peak_events_per_second: f64,
    peak_timestamp: Option<Instant>,
}

/// Latency distribution tracking
#[derive(Debug)]
pub struct LatencyTracker {
    /// Histogram buckets for latency distribution
    buckets: Vec<LatencyBucket>,
    
    /// Summary statistics
    total_samples: u64,
    sum_duration_micros: u64,
    min_latency: Option<Duration>,
    max_latency: Option<Duration>,
    
    /// Percentile tracking
    p50: Duration,
    p95: Duration,
    p99: Duration,
}

/// Error tracking and categorization
#[derive(Debug)]
pub struct ErrorTracker {
    /// Error counts by type
    error_counts: HashMap<String, u64>,
    
    /// Recent errors for debugging
    recent_errors: Vec<ErrorSample>,
    
    /// Error rate calculations
    error_rate_1min: f64,
    error_rate_5min: f64,
    error_rate_15min: f64,
}

/// Per-topic metrics
#[derive(Debug, Clone)]
pub struct TopicMetrics {
    pub topic: String,
    pub events_count: u64,
    pub bytes_processed: u64,
    pub average_size: f64,
    pub last_event_time: Option<Instant>,
    pub subscribers_count: u32,
    pub processing_latency: Duration,
}

/// Rule engine performance metrics
#[derive(Debug)]
pub struct RuleMetrics {
    pub rules_evaluated: u64,
    pub rules_matched: u64,
    pub rules_executed: u64,
    pub rule_processing_time: Duration,
    pub failed_rule_executions: u64,
    pub active_rules_count: u32,
}

/// Storage layer metrics
#[derive(Debug)]
pub struct StorageMetrics {
    pub reads_per_second: f64,
    pub writes_per_second: f64,
    pub read_latency: Duration,
    pub write_latency: Duration,
    pub storage_size: u64,
    pub connection_pool_usage: f64,
    pub cache_hit_rate: f64,
}

/// Time-based counter for throughput tracking
#[derive(Debug)]
pub struct TimedCounter {
    count: u64,
    window_start: Instant,
    window_duration: Duration,
}

/// Latency bucket for histogram
#[derive(Debug)]
pub struct LatencyBucket {
    pub upper_bound_micros: u64,
    pub count: u64,
}

/// Error sample for debugging
#[derive(Debug, Clone)]
pub struct ErrorSample {
    pub error_type: String,
    pub message: String,
    pub timestamp: Instant,
    pub context: HashMap<String, String>,
}

/// Comprehensive metrics report
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsReport {
    /// Timestamp of report generation
    pub timestamp: u64,
    
    /// System overview
    pub system: SystemMetrics,
    
    /// Event processing metrics
    pub events: EventMetrics,
    
    /// Performance metrics
    pub performance: PerformanceMetrics,
    
    /// Error metrics
    pub errors: ErrorMetricsReport,
    
    /// Topic breakdown
    pub topics: Vec<TopicMetricsReport>,
    
    /// Storage metrics
    pub storage: StorageMetricsReport,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub active_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMetrics {
    pub total_emitted: u64,
    pub total_processed: u64,
    pub events_per_second_1min: f64,
    pub events_per_second_5min: f64,
    pub peak_events_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_processing_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMetricsReport {
    pub total_errors: u64,
    pub error_rate_percent: f64,
    pub errors_by_type: HashMap<String, u64>,
    pub recent_errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicMetricsReport {
    pub topic: String,
    pub event_count: u64,
    pub bytes_processed: u64,
    pub subscribers: u32,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageMetricsReport {
    pub reads_per_second: f64,
    pub writes_per_second: f64,
    pub avg_read_latency_ms: f64,
    pub avg_write_latency_ms: f64,
    pub storage_size_mb: f64,
    pub cache_hit_rate_percent: f64,
}

impl AdvancedMetrics {
    /// Create new advanced metrics collector
    pub fn new() -> Self {
        Self {
            events_emitted: AtomicU64::new(0),
            events_processed: AtomicU64::new(0),
            events_failed: AtomicU64::new(0),
            throughput_tracker: Arc::new(RwLock::new(ThroughputTracker::new())),
            latency_tracker: Arc::new(RwLock::new(LatencyTracker::new())),
            error_tracker: Arc::new(RwLock::new(ErrorTracker::new())),
            topic_metrics: Arc::new(RwLock::new(HashMap::new())),
            rule_metrics: Arc::new(RwLock::new(RuleMetrics::new())),
            storage_metrics: Arc::new(RwLock::new(StorageMetrics::new())),
        }
    }
    
    /// Record event emission
    pub async fn record_event_emitted(&self, topic: &str, size_bytes: usize) {
        self.events_emitted.fetch_add(1, Ordering::Relaxed);
        
        let mut throughput = self.throughput_tracker.write().await;
        throughput.record_event();
        
        let mut topics = self.topic_metrics.write().await;
        let topic_metric = topics.entry(topic.to_string()).or_insert_with(|| TopicMetrics::new(topic));
        topic_metric.record_event(size_bytes);
    }
    
    /// Record event processing with latency
    pub async fn record_event_processed(&self, topic: &str, processing_duration: Duration) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        let mut latency = self.latency_tracker.write().await;
        latency.record_latency(processing_duration);
        
        let mut topics = self.topic_metrics.write().await;
        if let Some(topic_metric) = topics.get_mut(topic) {
            topic_metric.update_latency(processing_duration);
        }
    }
    
    /// Record processing error
    pub async fn record_error(&self, error_type: &str, message: &str, context: HashMap<String, String>) {
        self.events_failed.fetch_add(1, Ordering::Relaxed);
        
        let mut errors = self.error_tracker.write().await;
        errors.record_error(error_type, message, context);
    }
    
    /// Generate comprehensive metrics report
    pub async fn generate_report(&self) -> MetricsReport {
        let throughput = self.throughput_tracker.read().await;
        let latency = self.latency_tracker.read().await;
        let errors = self.error_tracker.read().await;
        let topics = self.topic_metrics.read().await;
        let storage = self.storage_metrics.read().await;
        
        MetricsReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            system: SystemMetrics::collect().await,
            events: EventMetrics {
                total_emitted: self.events_emitted.load(Ordering::Relaxed),
                total_processed: self.events_processed.load(Ordering::Relaxed),
                events_per_second_1min: throughput.get_events_per_second_1min(),
                events_per_second_5min: throughput.get_events_per_second_5min(),
                peak_events_per_second: throughput.peak_events_per_second,
            },
            performance: PerformanceMetrics {
                avg_processing_latency_ms: latency.get_avg_latency().as_millis() as f64,
                p50_latency_ms: latency.p50.as_millis() as f64,
                p95_latency_ms: latency.p95.as_millis() as f64,
                p99_latency_ms: latency.p99.as_millis() as f64,
                max_latency_ms: latency.max_latency.unwrap_or_default().as_millis() as f64,
            },
            errors: ErrorMetricsReport {
                total_errors: self.events_failed.load(Ordering::Relaxed),
                error_rate_percent: errors.error_rate_1min * 100.0,
                errors_by_type: errors.error_counts.clone(),
                recent_errors: errors.recent_errors.iter()
                    .map(|e| format!("{}: {}", e.error_type, e.message))
                    .collect(),
            },
            topics: topics.values().map(|t| t.to_report()).collect(),
            storage: storage.to_report(),
        }
    }
    
    /// Reset all metrics (useful for testing)
    pub async fn reset(&self) {
        self.events_emitted.store(0, Ordering::Relaxed);
        self.events_processed.store(0, Ordering::Relaxed);
        self.events_failed.store(0, Ordering::Relaxed);
        
        let mut throughput = self.throughput_tracker.write().await;
        throughput.reset();
        
        let mut latency = self.latency_tracker.write().await;
        latency.reset();
        
        let mut errors = self.error_tracker.write().await;
        errors.reset();
        
        let mut topics = self.topic_metrics.write().await;
        topics.clear();
    }
}

// Implementation details for the various metric types would continue here...
// Each struct would have impl blocks with appropriate methods

impl ThroughputTracker {
    pub fn new() -> Self {
        Self {
            current_minute: TimedCounter::new(Duration::from_secs(60)),
            last_minute: TimedCounter::new(Duration::from_secs(60)),
            current_hour: TimedCounter::new(Duration::from_secs(3600)),
            peak_events_per_second: 0.0,
            peak_timestamp: None,
        }
    }
    
    pub fn record_event(&mut self) {
        self.current_minute.increment();
        self.current_hour.increment();
        
        // Update peak if necessary
        let current_rate = self.get_events_per_second_1min();
        if current_rate > self.peak_events_per_second {
            self.peak_events_per_second = current_rate;
            self.peak_timestamp = Some(Instant::now());
        }
    }
    
    pub fn get_events_per_second_1min(&self) -> f64 {
        self.current_minute.get_rate()
    }
    
    pub fn get_events_per_second_5min(&self) -> f64 {
        // Would implement a 5-minute sliding window
        self.current_minute.get_rate() // Simplified
    }
    
    pub fn reset(&mut self) {
        self.current_minute.reset();
        self.last_minute.reset();
        self.current_hour.reset();
        self.peak_events_per_second = 0.0;
        self.peak_timestamp = None;
    }
}

impl TimedCounter {
    pub fn new(window_duration: Duration) -> Self {
        Self {
            count: 0,
            window_start: Instant::now(),
            window_duration,
        }
    }
    
    pub fn increment(&mut self) {
        self.check_window();
        self.count += 1;
    }
    
    pub fn get_rate(&self) -> f64 {
        let elapsed = self.window_start.elapsed();
        if elapsed.as_secs_f64() > 0.0 {
            self.count as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        }
    }
    
    pub fn reset(&mut self) {
        self.count = 0;
        self.window_start = Instant::now();
    }
    
    fn check_window(&mut self) {
        if self.window_start.elapsed() >= self.window_duration {
            self.reset();
        }
    }
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            buckets: Self::create_buckets(),
            total_samples: 0,
            sum_duration_micros: 0,
            min_latency: None,
            max_latency: None,
            p50: Duration::from_micros(0),
            p95: Duration::from_micros(0),
            p99: Duration::from_micros(0),
        }
    }
    
    fn create_buckets() -> Vec<LatencyBucket> {
        vec![
            LatencyBucket { upper_bound_micros: 100, count: 0 },      // 0.1ms
            LatencyBucket { upper_bound_micros: 500, count: 0 },      // 0.5ms
            LatencyBucket { upper_bound_micros: 1000, count: 0 },     // 1ms
            LatencyBucket { upper_bound_micros: 5000, count: 0 },     // 5ms
            LatencyBucket { upper_bound_micros: 10000, count: 0 },    // 10ms
            LatencyBucket { upper_bound_micros: 50000, count: 0 },    // 50ms
            LatencyBucket { upper_bound_micros: 100000, count: 0 },   // 100ms
            LatencyBucket { upper_bound_micros: u64::MAX, count: 0 }, // infinity
        ]
    }
    
    pub fn record_latency(&mut self, duration: Duration) {
        let micros = duration.as_micros() as u64;
        
        // Update summary stats
        self.total_samples += 1;
        self.sum_duration_micros += micros;
        
        if self.min_latency.is_none() || duration < self.min_latency.unwrap() {
            self.min_latency = Some(duration);
        }
        
        if self.max_latency.is_none() || duration > self.max_latency.unwrap() {
            self.max_latency = Some(duration);
        }
        
        // Update buckets
        for bucket in &mut self.buckets {
            if micros <= bucket.upper_bound_micros {
                bucket.count += 1;
                break;
            }
        }
        
        // Update percentiles (simplified)
        self.update_percentiles();
    }
    
    pub fn get_avg_latency(&self) -> Duration {
        if self.total_samples > 0 {
            Duration::from_micros(self.sum_duration_micros / self.total_samples)
        } else {
            Duration::from_micros(0)
        }
    }
    
    pub fn reset(&mut self) {
        self.total_samples = 0;
        self.sum_duration_micros = 0;
        self.min_latency = None;
        self.max_latency = None;
        self.p50 = Duration::from_micros(0);
        self.p95 = Duration::from_micros(0);
        self.p99 = Duration::from_micros(0);
        
        for bucket in &mut self.buckets {
            bucket.count = 0;
        }
    }
    
    fn update_percentiles(&mut self) {
        // Simplified percentile calculation
        // In a real implementation, you'd use a more sophisticated algorithm
        if let Some(max) = self.max_latency {
            self.p50 = Duration::from_micros(max.as_micros() as u64 / 2);
            self.p95 = Duration::from_micros((max.as_micros() as u64 * 95) / 100);
            self.p99 = Duration::from_micros((max.as_micros() as u64 * 99) / 100);
        }
    }
}

impl ErrorTracker {
    pub fn new() -> Self {
        Self {
            error_counts: HashMap::new(),
            recent_errors: Vec::new(),
            error_rate_1min: 0.0,
            error_rate_5min: 0.0,
            error_rate_15min: 0.0,
        }
    }
    
    pub fn record_error(&mut self, error_type: &str, message: &str, context: HashMap<String, String>) {
        // Update error counts
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
        
        // Add to recent errors (keep only last 100)
        self.recent_errors.push(ErrorSample {
            error_type: error_type.to_string(),
            message: message.to_string(),
            timestamp: Instant::now(),
            context,
        });
        
        if self.recent_errors.len() > 100 {
            self.recent_errors.remove(0);
        }
        
        // Update error rates (simplified)
        self.update_error_rates();
    }
    
    pub fn reset(&mut self) {
        self.error_counts.clear();
        self.recent_errors.clear();
        self.error_rate_1min = 0.0;
        self.error_rate_5min = 0.0;
        self.error_rate_15min = 0.0;
    }
    
    fn update_error_rates(&mut self) {
        // Simplified error rate calculation
        let recent_count = self.recent_errors.len() as f64;
        self.error_rate_1min = recent_count / 60.0; // Simplified
        self.error_rate_5min = recent_count / 300.0;
        self.error_rate_15min = recent_count / 900.0;
    }
}

impl TopicMetrics {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            events_count: 0,
            bytes_processed: 0,
            average_size: 0.0,
            last_event_time: None,
            subscribers_count: 0,
            processing_latency: Duration::from_micros(0),
        }
    }
    
    pub fn record_event(&mut self, size_bytes: usize) {
        self.events_count += 1;
        self.bytes_processed += size_bytes as u64;
        self.average_size = self.bytes_processed as f64 / self.events_count as f64;
        self.last_event_time = Some(Instant::now());
    }
    
    pub fn update_latency(&mut self, latency: Duration) {
        // Simple moving average
        let old_latency = self.processing_latency.as_micros() as f64;
        let new_latency = latency.as_micros() as f64;
        let avg_latency = (old_latency + new_latency) / 2.0;
        self.processing_latency = Duration::from_micros(avg_latency as u64);
    }
    
    pub fn to_report(&self) -> TopicMetricsReport {
        TopicMetricsReport {
            topic: self.topic.clone(),
            event_count: self.events_count,
            bytes_processed: self.bytes_processed,
            subscribers: self.subscribers_count,
            avg_latency_ms: self.processing_latency.as_millis() as f64,
        }
    }
}

impl RuleMetrics {
    pub fn new() -> Self {
        Self {
            rules_evaluated: 0,
            rules_matched: 0,
            rules_executed: 0,
            rule_processing_time: Duration::from_micros(0),
            failed_rule_executions: 0,
            active_rules_count: 0,
        }
    }
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self {
            reads_per_second: 0.0,
            writes_per_second: 0.0,
            read_latency: Duration::from_micros(0),
            write_latency: Duration::from_micros(0),
            storage_size: 0,
            connection_pool_usage: 0.0,
            cache_hit_rate: 0.0,
        }
    }
    
    pub fn to_report(&self) -> StorageMetricsReport {
        StorageMetricsReport {
            reads_per_second: self.reads_per_second,
            writes_per_second: self.writes_per_second,
            avg_read_latency_ms: self.read_latency.as_millis() as f64,
            avg_write_latency_ms: self.write_latency.as_millis() as f64,
            storage_size_mb: self.storage_size as f64 / (1024.0 * 1024.0),
            cache_hit_rate_percent: self.cache_hit_rate * 100.0,
        }
    }
}

impl SystemMetrics {
    pub async fn collect() -> Self {
        // Simplified system metrics collection
        Self {
            uptime_seconds: 0, // Would get actual uptime
            memory_usage_bytes: 0, // Would get actual memory usage
            cpu_usage_percent: 0.0, // Would get actual CPU usage
            active_connections: 0, // Would get actual connection count
        }
    }
}

// Additional implementation details would continue... 