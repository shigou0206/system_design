//! JSON-RPC service implementation for the event bus

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{Semaphore, broadcast};
use tokio::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::core::{
    EventEnvelope, EventQuery, EventTriggerRule,
    traits::{EventBus, EventStorage, RuleEngine, EventBusResult},
    EventBusError
};
use crate::storage::MemoryStorage;

/// Main event bus service that implements JSON-RPC interface
pub struct EventBusService {
    /// Storage backend for persistence
    storage: Option<Arc<dyn EventStorage>>,
    
    /// Rule engine for automated responses
    rule_engine: Option<Arc<dyn RuleEngine>>,
    
    /// In-memory event distribution (for subscriptions)
    memory_storage: Arc<MemoryStorage>,
    
    /// Service configuration
    config: ServiceConfig,
    
    /// Concurrency control for emit operations
    emit_semaphore: Arc<Semaphore>,
    
    /// Broadcast channel for real-time subscriptions
    event_sender: broadcast::Sender<EventEnvelope>,
    
    /// Performance metrics
    metrics: ServiceMetrics,
}

/// Configuration for the event bus service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service instance ID
    pub instance_id: String,
    
    /// Maximum number of events to keep in memory for subscriptions
    pub max_memory_events: usize,
    
    /// Whether to enable rule processing
    pub enable_rules: bool,
    
    /// Allowed source TRN patterns
    pub allowed_sources: Vec<String>,
    
    /// Maximum concurrent emit operations
    pub max_concurrent_emits: usize,
    
    /// Rate limiting: max events per second
    pub max_events_per_second: Option<u32>,
    
    /// Batch size for storage operations
    pub batch_size: usize,
    
    /// Grace period for shutdown
    #[serde(with = "duration_serde")]
    pub shutdown_grace_period: Duration,
    
    /// Storage configuration
    pub storage: crate::config::StorageConfig,
    
    /// Event buffer size for processing
    pub event_buffer_size: usize,
    
    /// Subscriber buffer size
    pub subscriber_buffer_size: usize,
    
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Enable graceful shutdown
    pub enable_graceful_shutdown: bool,
    
    /// Shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            instance_id: "default".to_string(),
            max_memory_events: 1000,
            enable_rules: false,
            allowed_sources: vec!["*".to_string()],
            max_concurrent_emits: 100,
            max_events_per_second: None,
            batch_size: 50,
            shutdown_grace_period: Duration::from_secs(30),
            storage: crate::config::StorageConfig::Memory,
            event_buffer_size: 10000,
            subscriber_buffer_size: 1000,
            enable_metrics: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_secs: 30,
        }
    }
}

/// Service performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Total events processed
    #[serde(skip)]
    events_processed: AtomicU64,
    
    /// Events processed in the last second - snapshot value for serialization
    events_last_second_count: u64,
    
    /// Active subscription count
    #[serde(skip)]
    active_subscriptions: AtomicU64,
    
    /// Current concurrent operations
    #[serde(skip)]
    current_operations: AtomicU64,
    
    /// Error count
    #[serde(skip)]
    error_count: AtomicU64,
    
    /// Non-atomic fields for serialization
    #[serde(skip)]
    events_last_second: parking_lot::RwLock<Vec<Instant>>,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            events_processed: AtomicU64::new(0),
            events_last_second_count: 0,
            active_subscriptions: AtomicU64::new(0),
            current_operations: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            events_last_second: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl ServiceMetrics {
    /// Record an event being processed
    fn record_event(&self) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        
        let now = Instant::now();
        let mut last_second = self.events_last_second.write();
        
        // Remove events older than 1 second
        last_second.retain(|&instant| now.duration_since(instant) < Duration::from_secs(1));
        last_second.push(now);
    }
    
    /// Get events per second
    fn get_events_per_second(&self) -> f64 {
        let last_second = self.events_last_second.read();
        last_second.len() as f64
    }
    
    /// Record an error
    fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increment operation counter
    fn start_operation(&self) {
        self.current_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Decrement operation counter
    fn end_operation(&self) {
        self.current_operations.fetch_sub(1, Ordering::Relaxed);
    }
    
    /// Get the total number of events processed
    pub fn events_processed(&self) -> u64 {
        self.events_processed.load(Ordering::Relaxed)
    }
    
    /// Get the number of events processed in the last second
    pub fn events_per_second(&self) -> f64 {
        self.get_events_per_second()
    }
    
    /// Get the number of active subscriptions
    pub fn active_subscriptions(&self) -> u64 {
        self.active_subscriptions.load(Ordering::Relaxed)
    }
    
    /// Get the number of current operations
    pub fn current_operations(&self) -> u64 {
        self.current_operations.load(Ordering::Relaxed)
    }
    
    /// Get the total error count
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
}

impl EventBusService {
    /// Create a new event bus service
    pub fn new(config: ServiceConfig) -> Self {
        let (event_sender, _) = broadcast::channel(config.max_memory_events);
        
        Self {
            storage: None,
            rule_engine: None,
            memory_storage: Arc::new(MemoryStorage::new()),
            emit_semaphore: Arc::new(Semaphore::new(config.max_concurrent_emits)),
            event_sender,
            metrics: ServiceMetrics::default(),
            config,
        }
    }
    
    /// Create a new event bus service with async initialization
    pub async fn with_config(config: ServiceConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self::new(config))
    }
    
    /// Set the storage backend
    pub fn with_storage(mut self, storage: Arc<dyn EventStorage>) -> Self {
        self.storage = Some(storage);
        self
    }
    
    /// Set the rule engine
    pub fn with_rule_engine(mut self, rule_engine: Arc<dyn RuleEngine>) -> Self {
        self.rule_engine = Some(rule_engine);
        self.config.enable_rules = true;
        self
    }
    
    /// Start the event bus service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize storage if configured
        if let Some(storage) = &self.storage {
            storage.initialize().await?;
        }
        Ok(())
    }
    
    /// Emit a single event (wrapper around handle_emit_event)
    pub async fn emit_event(&self, event: EventEnvelope) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.handle_emit_event(event).await.map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
    
    /// Get service metrics
    pub async fn get_metrics(&self) -> Result<ServiceMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Create a snapshot of current metrics
        let events_processed = self.metrics.events_processed.load(Ordering::Relaxed);
        let active_subscriptions = self.metrics.active_subscriptions.load(Ordering::Relaxed);
        let current_operations = self.metrics.current_operations.load(Ordering::Relaxed);
        let error_count = self.metrics.error_count.load(Ordering::Relaxed);
        
        // Calculate events in last second
        let last_second_count = {
            let events = self.metrics.events_last_second.read();
            let cutoff = tokio::time::Instant::now() - Duration::from_secs(1);
            events.iter().filter(|&&instant| instant > cutoff).count() as u64
        };
        
        Ok(ServiceMetrics {
            events_processed: AtomicU64::new(events_processed),
            events_last_second_count: last_second_count,
            active_subscriptions: AtomicU64::new(active_subscriptions),
            current_operations: AtomicU64::new(current_operations),
            error_count: AtomicU64::new(error_count),
            events_last_second: parking_lot::RwLock::new(Vec::new()),
        })
    }
    
    /// Check if source TRN is allowed
    fn is_source_allowed(&self, source_trn: Option<&String>) -> bool {
        // If no restrictions, allow all
        if self.config.allowed_sources.contains(&"*".to_string()) {
            return true;
        }
        
        // If no source TRN provided, check if empty sources are allowed
        let source = match source_trn {
            Some(s) => s,
            None => return self.config.allowed_sources.is_empty(),
        };
        
        // Check against patterns
        for pattern in &self.config.allowed_sources {
            if pattern == "*" || source.starts_with(pattern.trim_end_matches('*')) {
                return true;
            }
        }
        
        false
    }
    
    /// Check rate limiting
    async fn check_rate_limit(&self) -> EventBusResult<()> {
        if let Some(max_eps) = self.config.max_events_per_second {
            let current_eps = self.metrics.get_events_per_second();
            if current_eps >= max_eps as f64 {
                return Err(EventBusError::rate_limited(
                    format!("Rate limit exceeded: {:.1} EPS", current_eps)
                ));
            }
        }
        Ok(())
    }
    
    /// Emit multiple events in batch
    pub async fn emit_batch(&self, events: Vec<EventEnvelope>) -> EventBusResult<()> {
        // Check rate limiting for batch
        self.check_rate_limit().await?;
        
        // Acquire semaphore permits for batch
        let _permits = self.emit_semaphore.acquire_many(events.len() as u32).await
            .map_err(|_| EventBusError::internal("Failed to acquire semaphore permits"))?;
        
        self.metrics.start_operation();
        
        let result = async {
            // Validate all events first
            for event in &events {
                if !self.is_source_allowed(event.source_trn.as_ref()) {
                    return Err(EventBusError::permission_denied(
                        format!("Source TRN not allowed: {:?}", event.source_trn)
                    ));
                }
            }
            
            // Store in persistent storage if available (batch operation)
            if let Some(ref storage) = self.storage {
                // TODO: Implement batch store method
                for event in &events {
                    storage.store(event).await?;
                }
            }
            
            // Store in memory for real-time subscriptions
            for event in &events {
                self.memory_storage.store(event).await?;
                
                // Broadcast to subscribers
                let _ = self.event_sender.send(event.clone());
                
                // Record metrics
                self.metrics.record_event();
            }
            
            // Process rules if enabled
            if self.config.enable_rules {
                if let Some(ref rule_engine) = self.rule_engine {
                    for event in &events {
                        let _invocations = rule_engine.process_event(event).await?;
                        // TODO: Execute tool invocations
                    }
                }
            }
            
            Ok(())
        }.await;
        
        self.metrics.end_operation();
        
        if result.is_err() {
            self.metrics.record_error();
        }
        
        result
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&self) -> EventBusResult<()> {
        // Wait for ongoing operations to complete
        let start = Instant::now();
        while self.metrics.current_operations.load(Ordering::Relaxed) > 0 {
            if start.elapsed() > self.config.shutdown_grace_period {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Close broadcast channel
        // Note: broadcast channels don't have explicit close
        
        Ok(())
    }
}

#[async_trait]
impl EventBus for EventBusService {
    async fn emit(&self, event: EventEnvelope) -> EventBusResult<()> {
        // Validate source TRN
        if !self.is_source_allowed(event.source_trn.as_ref()) {
            return Err(EventBusError::permission_denied(
                format!("Source TRN not allowed: {:?}", event.source_trn)
            ));
        }
        
        // Check rate limiting for single emit
        self.check_rate_limit().await?;
        
        // Acquire semaphore permit for single emit
        let _permit = self.emit_semaphore.acquire().await
            .map_err(|_| EventBusError::internal("Failed to acquire semaphore permit"))?;
        
        self.metrics.start_operation();
        
        let result = async {
            // Store in persistent storage if available
            if let Some(ref storage) = self.storage {
                storage.store(&event).await?;
            }
            
            // Store in memory for real-time subscriptions
            self.memory_storage.store(&event).await?;
            
            // Broadcast to subscribers
            let _ = self.event_sender.send(event.clone());
            
            // Record metrics
            self.metrics.record_event();
            
            // Process rules if enabled
            if self.config.enable_rules {
                if let Some(ref rule_engine) = self.rule_engine {
                    let _invocations = rule_engine.process_event(&event).await?;
                    // TODO: Execute tool invocations
                }
            }
            
            Ok(())
        }.await;
        
        self.metrics.end_operation();
        
        if result.is_err() {
            self.metrics.record_error();
        }
        
        result
    }
    
    async fn poll(&self, query: EventQuery) -> EventBusResult<Vec<EventEnvelope>> {
        // Query persistent storage first, fall back to memory
        if let Some(ref storage) = self.storage {
            storage.query(&query).await
        } else {
            self.memory_storage.query(&query).await
        }
    }
    
    async fn subscribe(&self, topic: &str) -> EventBusResult<std::pin::Pin<Box<dyn futures::Stream<Item = EventEnvelope> + Send>>> {
        use futures::stream::StreamExt;
        use tokio_stream::wrappers::BroadcastStream;
        
        let receiver = self.event_sender.subscribe();
        let topic_filter = topic.to_string();
        
        // Increment subscription counter
        self.metrics.active_subscriptions.fetch_add(1, Ordering::Relaxed);
        
        let stream = BroadcastStream::new(receiver)
            .filter_map(move |result| {
                let topic_filter = topic_filter.clone();
                async move {
                    match result {
                        Ok(event) => {
                            // Filter by topic (support wildcards)
                            if topic_filter == "*" || event.topic == topic_filter || 
                               (topic_filter.ends_with('*') && 
                                event.topic.starts_with(topic_filter.trim_end_matches('*'))) {
                                Some(event)
                            } else {
                                None
                            }
                        }
                        Err(_) => None, // Skip broadcast errors
                    }
                }
            });
        
        Ok(Box::pin(stream))
    }
    
    async fn list_topics(&self) -> EventBusResult<Vec<String>> {
        // Get topics from storage or memory
        let storage: &dyn EventStorage = self.storage.as_ref()
            .map(|s| s.as_ref())
            .unwrap_or(self.memory_storage.as_ref());
        
        // Query all events to extract topics
        let query = EventQuery::new();
        let events = storage.query(&query).await?;
        
        let mut topics: Vec<String> = events
            .into_iter()
            .map(|e| e.topic)
            .collect();
        
        topics.sort();
        topics.dedup();
        
        Ok(topics)
    }
    
    async fn get_stats(&self) -> EventBusResult<crate::core::traits::BusStats> {
        let memory_stats = self.memory_storage.get_stats().await?;
        
        Ok(crate::core::traits::BusStats {
            events_processed: self.metrics.events_processed.load(Ordering::Relaxed),
            active_subscriptions: self.metrics.active_subscriptions.load(Ordering::Relaxed) as u32,
            topic_count: memory_stats.topics_count,
            events_per_second: self.metrics.get_events_per_second(),
        })
    }
}

/// JSON-RPC method implementations
impl EventBusService {
    /// Handle emit_event method
    pub async fn handle_emit_event(&self, event: EventEnvelope) -> EventBusResult<serde_json::Value> {
        self.emit(event).await?;
        Ok(serde_json::json!({"status": "success"}))
    }
    
    /// Handle poll_events method
    pub async fn handle_poll_events(&self, query: EventQuery) -> EventBusResult<Vec<EventEnvelope>> {
        self.poll(query).await
    }
    
    /// Handle register_rule method
    pub async fn handle_register_rule(&self, rule: EventTriggerRule) -> EventBusResult<serde_json::Value> {
        if let Some(ref rule_engine) = self.rule_engine {
            rule_engine.register_rule(rule).await?;
            Ok(serde_json::json!({"status": "success"}))
        } else {
            Err(EventBusError::configuration("Rule engine not enabled"))
        }
    }
    
    /// Handle list_topics method
    pub async fn handle_list_topics(&self) -> EventBusResult<Vec<String>> {
        self.list_topics().await
    }
    
    /// Handle get_stats method (for monitoring)
    pub async fn handle_get_stats(&self) -> EventBusResult<serde_json::Value> {
        let stats = self.get_stats().await?;
        Ok(serde_json::json!({
            "events_processed": stats.events_processed,
            "active_subscriptions": stats.active_subscriptions,
            "topic_count": stats.topic_count,
            "events_per_second": stats.events_per_second
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_event_bus_service_basic() {
        let config = ServiceConfig::default();
        let service = EventBusService::new(config);
        
        // Test emitting an event
        let event = EventEnvelope::new("test.topic", json!({"message": "hello"}));
        assert!(service.emit(event).await.is_ok());
        
        // Test polling events
        let query = EventQuery::new().with_topic("test.topic");
        let events = service.poll(query).await.unwrap();
        assert_eq!(events.len(), 1);
        
        // Test listing topics
        let topics = service.list_topics().await.unwrap();
        assert!(topics.contains(&"test.topic".to_string()));
    }
    
    #[tokio::test]
    async fn test_source_trn_validation() {
        let mut config = ServiceConfig::default();
        config.allowed_sources = vec!["trn:user:alice:*".to_string()];
        let service = EventBusService::new(config);
        
        // Test allowed source
        let event = EventEnvelope::new("test", json!({}))
            .set_trn(Some("trn:user:alice:tool:test".to_string()), None);
        assert!(service.emit(event).await.is_ok());
        
        // Test disallowed source
        let event = EventEnvelope::new("test", json!({}))
            .set_trn(Some("trn:user:bob:tool:test".to_string()), None);
        assert!(service.emit(event).await.is_err());
    }
} 

/// Configuration for multiple event bus instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiBusConfig {
    /// Individual bus configurations
    pub buses: HashMap<String, ServiceConfig>,
    /// Global settings that apply to all buses
    pub global: GlobalConfig,
    /// Default bus name to use when none specified
    pub default_bus: Option<String>,
}

/// Global configuration shared across all event bus instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Global rate limiting settings
    pub rate_limit: Option<RateLimitConfig>,
    /// Global metrics configuration
    pub metrics: Option<MetricsConfig>,
    /// Global logging configuration
    pub logging: Option<LoggingConfig>,
    /// Shutdown timeout for all buses
    pub shutdown_timeout_secs: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum events per second across all buses
    pub global_max_eps: Option<f64>,
    /// Per-bus maximum events per second
    pub per_bus_max_eps: Option<f64>,
    /// Burst capacity
    pub burst_capacity: Option<u32>,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether to enable metrics collection
    pub enabled: bool,
    /// Metrics export endpoint
    pub endpoint: Option<String>,
    /// Export interval in seconds
    pub export_interval_secs: u64,
    /// Custom metric labels
    pub labels: HashMap<String, String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format (json, text)
    pub format: String,
    /// Whether to log events
    pub log_events: bool,
    /// Whether to log performance metrics
    pub log_performance: bool,
}

impl Default for MultiBusConfig {
    fn default() -> Self {
        let mut buses = HashMap::new();
        
        // Default workflow bus
        buses.insert(
            "workflows".to_string(),
            ServiceConfig {
                storage: crate::config::StorageConfig::Memory,
                max_concurrent_emits: 100,
                max_events_per_second: Some(1000),
                ..Default::default()
            }
        );
        
        // Default global bus
        buses.insert(
            "global".to_string(),
            ServiceConfig {
                storage: crate::config::StorageConfig::Memory,
                max_concurrent_emits: 200,
                max_events_per_second: Some(2000),
                ..Default::default()
            }
        );

        Self {
            buses,
            global: GlobalConfig::default(),
            default_bus: Some("global".to_string()),
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            rate_limit: Some(RateLimitConfig::default()),
            metrics: Some(MetricsConfig::default()),
            logging: Some(LoggingConfig::default()),
            shutdown_timeout_secs: 60,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            global_max_eps: Some(5000.0),
            per_bus_max_eps: Some(2000.0),
            burst_capacity: Some(1000),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: Some("/metrics".to_string()),
            export_interval_secs: 10,
            labels: HashMap::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            log_events: false,
            log_performance: true,
        }
    }
}

/// Multi-bus manager for handling multiple EventBus instances
pub struct MultiBusManager {
    /// Individual bus services
    buses: HashMap<String, EventBusService>,
    /// Configuration
    config: MultiBusConfig,
    /// Shutdown signal
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl MultiBusManager {
    /// Create a new multi-bus manager
    pub async fn new(config: MultiBusConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut buses = HashMap::new();
        
        for (name, bus_config) in &config.buses {
            let service = EventBusService::with_config(bus_config.clone()).await?;
            buses.insert(name.clone(), service);
        }
        
        Ok(Self {
            buses,
            config,
            shutdown_tx: None,
        })
    }

    /// Start all bus instances
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        for (name, bus) in &self.buses {
            tracing::info!("Starting event bus: {}", name);
            bus.start().await?;
        }

        tracing::info!("All event buses started successfully");
        Ok(())
    }

    /// Stop all bus instances gracefully
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        let timeout = std::time::Duration::from_secs(self.config.global.shutdown_timeout_secs);
        
        for (name, bus) in &self.buses {
            tracing::info!("Stopping event bus: {}", name);
            tokio::time::timeout(timeout, bus.shutdown()).await
                .map_err(|_| format!("Timeout stopping bus: {}", name))?
                .map_err(|e| format!("Error stopping bus {}: {}", name, e))?;
        }

        tracing::info!("All event buses stopped successfully");
        Ok(())
    }

    /// Get a specific bus by name
    pub fn get_bus(&self, name: &str) -> Option<&EventBusService> {
        self.buses.get(name)
    }

    /// Get the default bus
    pub fn get_default_bus(&self) -> Option<&EventBusService> {
        let default_name = self.config.default_bus.as_ref()?;
        self.buses.get(default_name)
    }

    /// Get all bus names
    pub fn bus_names(&self) -> Vec<String> {
        self.buses.keys().cloned().collect()
    }

    /// Emit event to a specific bus
    pub async fn emit_to_bus(
        &self,
        bus_name: &str,
        event: EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bus = self.buses.get(bus_name)
            .ok_or_else(|| format!("Bus '{}' not found", bus_name))?;
        
        bus.emit_event(event).await
    }

    /// Emit event to default bus
    pub async fn emit(
        &self,
        event: EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let default_name = self.config.default_bus.as_ref()
            .ok_or("No default bus configured")?;
        
        self.emit_to_bus(default_name, event).await
    }

    /// Subscribe to events from a specific bus
    pub async fn subscribe_to_bus(
        &self,
        bus_name: &str,
        topic: String,
    ) -> Result<tokio::sync::broadcast::Receiver<EventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        let bus = self.buses.get(bus_name)
            .ok_or_else(|| format!("Bus '{}' not found", bus_name))?;
        
        let _subscription = bus.subscribe(&topic).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        // For now, return a simple channel - this would need proper implementation
        let (_tx, rx) = tokio::sync::broadcast::channel(1000);
        Ok(rx)
    }

    /// Subscribe to events from default bus
    pub async fn subscribe(
        &self,
        topic: String,
    ) -> Result<tokio::sync::broadcast::Receiver<EventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        let default_name = self.config.default_bus.as_ref()
            .ok_or("No default bus configured")?;
        
        self.subscribe_to_bus(default_name, topic).await
    }

    /// Get combined metrics from all buses
    pub async fn get_combined_metrics(&self) -> Result<CombinedMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let mut combined = CombinedMetrics::new();
        
        for (name, bus) in &self.buses {
            if let Ok(metrics) = bus.get_metrics().await {
                combined.add_bus_metrics(name.clone(), metrics);
            }
        }
        
        Ok(combined)
    }

    /// Get configuration
    pub fn config(&self) -> &MultiBusConfig {
        &self.config
    }
}

/// Combined metrics from multiple buses
#[derive(Debug, Serialize, Deserialize)]
pub struct CombinedMetrics {
    /// Per-bus metrics
    pub buses: HashMap<String, ServiceMetrics>,
    /// Aggregated totals
    pub totals: ServiceMetrics,
    /// Collection timestamp
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

impl CombinedMetrics {
    pub fn new() -> Self {
        Self {
            buses: HashMap::new(),
            totals: ServiceMetrics::default(),
            collected_at: chrono::Utc::now(),
        }
    }

    pub fn add_bus_metrics(&mut self, bus_name: String, metrics: ServiceMetrics) {
        // Add to per-bus metrics (we'll clone the serializable version)
        let serializable_metrics = ServiceMetrics {
            events_processed: AtomicU64::new(metrics.events_processed.load(Ordering::Relaxed)),
            events_last_second_count: metrics.events_last_second_count,
            active_subscriptions: AtomicU64::new(metrics.active_subscriptions.load(Ordering::Relaxed)),
            current_operations: AtomicU64::new(metrics.current_operations.load(Ordering::Relaxed)),
            error_count: AtomicU64::new(metrics.error_count.load(Ordering::Relaxed)),
            events_last_second: parking_lot::RwLock::new(Vec::new()),
        };
        self.buses.insert(bus_name, serializable_metrics);
        
        // Add to totals using atomic operations
        self.totals.events_processed.fetch_add(metrics.events_processed.load(Ordering::Relaxed), Ordering::Relaxed);
        self.totals.events_last_second_count += metrics.events_last_second_count;
        self.totals.active_subscriptions.fetch_add(metrics.active_subscriptions.load(Ordering::Relaxed), Ordering::Relaxed);
        self.totals.error_count.fetch_add(metrics.error_count.load(Ordering::Relaxed), Ordering::Relaxed);
        
        // Update timestamp
        self.collected_at = chrono::Utc::now();
    }
    
    /// Get total events processed across all buses
    pub fn total_events_processed(&self) -> u64 {
        self.totals.events_processed()
    }
    
    /// Get total active subscriptions across all buses
    pub fn total_active_subscriptions(&self) -> u64 {
        self.totals.active_subscriptions()
    }
    
    /// Get per-bus metrics iterator
    pub fn buses(&self) -> impl Iterator<Item = (&String, &ServiceMetrics)> {
        self.buses.iter()
    }
    
    /// Get metrics for a specific bus
    pub fn get_bus_metrics(&self, bus_name: &str) -> Option<&ServiceMetrics> {
        self.buses.get(bus_name)
    }
}

impl Default for CombinedMetrics {
    fn default() -> Self {
        Self::new()
    }
} 