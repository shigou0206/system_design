//! Configuration management for the event bus system

use std::collections::HashMap;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

use crate::core::EventBusError;

/// Configuration for a single event bus instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// Unique identifier for this instance
    pub id: String,
    
    /// Address to listen on for JSON-RPC connections
    pub listen: SocketAddr,
    
    /// Whether to enable persistent storage
    #[serde(default)]
    pub persist: bool,
    
    /// Storage configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageConfig>,
    
    /// Whether to enable the rule engine
    #[serde(default)]
    pub enable_rules: bool,
    
    /// Rule engine configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_config: Option<RuleEngineConfig>,
    
    /// Allowed source TRNs (patterns)
    #[serde(default)]
    pub allowed_sources: Vec<String>,
    
    /// Maximum number of concurrent rule executions
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,
    
    /// Event retention settings
    #[serde(default)]
    pub retention: RetentionConfig,
    
    /// Transport configuration
    #[serde(default)]
    pub transport: TransportConfig,
}

fn default_max_concurrency() -> u32 {
    8
}

impl EventBusConfig {
    /// Create a new configuration with minimal settings
    pub fn new(id: impl Into<String>, listen: SocketAddr) -> Self {
        Self {
            id: id.into(),
            listen,
            persist: false,
            storage: None,
            enable_rules: false,
            rule_config: None,
            allowed_sources: vec!["*".to_string()], // Allow all by default
            max_concurrency: default_max_concurrency(),
            retention: RetentionConfig::default(),
            transport: TransportConfig::default(),
        }
    }
    
    /// Enable persistent storage with SQLite
    pub fn with_sqlite_storage(mut self, path: impl Into<String>) -> Self {
        self.persist = true;
        self.storage = Some(StorageConfig::Sqlite {
            path: path.into(),
        });
        self
    }
    
    /// Enable persistent storage with PostgreSQL
    pub fn with_postgres_storage(mut self, url: impl Into<String>) -> Self {
        self.persist = true;
        self.storage = Some(StorageConfig::Postgres {
            url: url.into(),
            pool_size: 10,
        });
        self
    }
    
    /// Enable the rule engine
    pub fn with_rules(mut self, config: RuleEngineConfig) -> Self {
        self.enable_rules = true;
        self.rule_config = Some(config);
        self
    }
    
    /// Set allowed source TRN patterns
    pub fn with_allowed_sources(mut self, sources: Vec<String>) -> Self {
        self.allowed_sources = sources;
        self
    }
    
    /// Set maximum concurrency
    pub fn with_max_concurrency(mut self, max_concurrency: u32) -> Self {
        self.max_concurrency = max_concurrency;
        self
    }
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    /// SQLite storage
    Sqlite {
        /// Database file path
        path: String,
    },
    
    /// PostgreSQL storage
    Postgres {
        /// Database connection URL
        url: String,
        /// Connection pool size
        #[serde(default = "default_pool_size")]
        pool_size: u32,
    },
    

    
    /// In-memory storage (for testing)
    Memory,
}

fn default_pool_size() -> u32 {
    10
}



/// Rule engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineConfig {
    /// Maximum number of concurrent rule executions
    #[serde(default = "default_max_rule_concurrency")]
    pub max_concurrency: u32,
    
    /// Default timeout for rule actions
    #[serde(default = "default_rule_timeout")]
    pub default_timeout_ms: u64,
    
    /// Whether to retry failed rule executions
    #[serde(default)]
    pub retry_failed: bool,
    
    /// Maximum number of retries for failed rules
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Delay between retries
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
}

fn default_max_rule_concurrency() -> u32 {
    4
}

fn default_rule_timeout() -> u64 {
    30000 // 30 seconds
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000 // 1 second
}

impl Default for RuleEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrency: default_max_rule_concurrency(),
            default_timeout_ms: default_rule_timeout(),
            retry_failed: false,
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
        }
    }
}

/// Event retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Maximum age of events in seconds (0 = no limit)
    #[serde(default)]
    pub max_age_seconds: u64,
    
    /// Maximum number of events to keep (0 = no limit)
    #[serde(default)]
    pub max_events: u64,
    
    /// How often to run cleanup in seconds
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_seconds: u64,
}

fn default_cleanup_interval() -> u64 {
    3600 // 1 hour
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            max_age_seconds: 0, // No limit by default
            max_events: 0,      // No limit by default
            cleanup_interval_seconds: default_cleanup_interval(),
        }
    }
}

/// Transport layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// TCP connection timeout
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_ms: u64,
    
    /// Read timeout for requests
    #[serde(default = "default_read_timeout")]
    pub read_timeout_ms: u64,
    
    /// Write timeout for responses
    #[serde(default = "default_write_timeout")]
    pub write_timeout_ms: u64,
    
    /// Maximum message size
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,
    
    /// Maximum number of concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_connect_timeout() -> u64 {
    5000 // 5 seconds
}

fn default_read_timeout() -> u64 {
    30000 // 30 seconds
}

fn default_write_timeout() -> u64 {
    30000 // 30 seconds
}

fn default_max_message_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_max_connections() -> u32 {
    100
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: default_connect_timeout(),
            read_timeout_ms: default_read_timeout(),
            write_timeout_ms: default_write_timeout(),
            max_message_size: default_max_message_size(),
            max_connections: default_max_connections(),
        }
    }
}

/// Event bus instance that combines configuration with runtime state
#[derive(Debug)]
pub struct EventBusInstance {
    /// Instance configuration
    pub config: EventBusConfig,
    
    /// Runtime properties (not serialized)
    pub runtime: InstanceRuntime,
}

/// Runtime state for an event bus instance
#[derive(Debug)]
pub struct InstanceRuntime {
    /// Whether this instance is currently running
    pub running: bool,
    
    /// Statistics
    pub stats: InstanceStats,
    
    /// Additional runtime metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Instance statistics
#[derive(Debug, Default)]
pub struct InstanceStats {
    /// Total events processed
    pub events_processed: u64,
    
    /// Number of active connections
    pub active_connections: u32,
    
    /// Number of registered rules
    pub registered_rules: u32,
    
    /// Instance start time
    pub start_time: Option<std::time::SystemTime>,
}

impl EventBusInstance {
    /// Create a new instance with the given configuration
    pub fn new(config: EventBusConfig) -> Self {
        Self {
            config,
            runtime: InstanceRuntime {
                running: false,
                stats: InstanceStats::default(),
                metadata: HashMap::new(),
            },
        }
    }
    
    /// Get the instance ID
    pub fn id(&self) -> &str {
        &self.config.id
    }
    
    /// Get the listen address
    pub fn listen_addr(&self) -> SocketAddr {
        self.config.listen
    }
    
    /// Check if persistence is enabled
    pub fn has_persistence(&self) -> bool {
        self.config.persist
    }
    
    /// Check if rules are enabled
    pub fn has_rules(&self) -> bool {
        self.config.enable_rules
    }
}

/// Configuration file structure for multiple instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceConfig {
    /// List of event bus instances
    pub instances: Vec<EventBusConfig>,
    
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,
}

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Global log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Whether to enable metrics collection
    #[serde(default)]
    pub enable_metrics: bool,
    
    /// Metrics export configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics_config: Option<MetricsConfig>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            enable_metrics: false,
            metrics_config: None,
        }
    }
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Metrics export endpoint
    pub endpoint: SocketAddr,
    
    /// Export interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub interval_seconds: u64,
}

fn default_metrics_interval() -> u64 {
    60 // 1 minute
}

impl MultiInstanceConfig {
    /// Load configuration from a JSON file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, EventBusError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EventBusError::configuration(format!("Failed to read config file: {}", e)))?;
        
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| EventBusError::configuration(format!("Failed to parse config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a JSON file
    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), EventBusError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| EventBusError::configuration(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| EventBusError::configuration(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), EventBusError> {
        // Check for duplicate instance IDs
        let mut ids = std::collections::HashSet::new();
        for instance in &self.instances {
            if !ids.insert(&instance.id) {
                return Err(EventBusError::configuration(
                    format!("Duplicate instance ID: {}", instance.id)
                ));
            }
        }
        
        // Check for duplicate listen addresses
        let mut addresses = std::collections::HashSet::new();
        for instance in &self.instances {
            if !addresses.insert(instance.listen) {
                return Err(EventBusError::configuration(
                    format!("Duplicate listen address: {}", instance.listen)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get instance by ID
    pub fn get_instance(&self, id: &str) -> Option<&EventBusConfig> {
        self.instances.iter().find(|i| i.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    
    #[test]
    fn test_basic_config() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let config = EventBusConfig::new("test", addr)
            .with_sqlite_storage("test.db")
            .with_rules(RuleEngineConfig::default());
        
        assert_eq!(config.id, "test");
        assert_eq!(config.listen, addr);
        assert!(config.persist);
        assert!(config.enable_rules);
    }
    
    #[test]
    fn test_multi_instance_config() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        
        let config = MultiInstanceConfig {
            instances: vec![
                EventBusConfig::new("instance1", addr1),
                EventBusConfig::new("instance2", addr2),
            ],
            global: GlobalConfig::default(),
        };
        
        assert!(config.validate().is_ok());
        assert!(config.get_instance("instance1").is_some());
        assert!(config.get_instance("nonexistent").is_none());
    }
    
    #[test]
    fn test_duplicate_validation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        
        // Test duplicate IDs
        let config = MultiInstanceConfig {
            instances: vec![
                EventBusConfig::new("duplicate", addr),
                EventBusConfig::new("duplicate", addr),
            ],
            global: GlobalConfig::default(),
        };
        
        assert!(config.validate().is_err());
    }
} 