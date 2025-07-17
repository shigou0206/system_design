//! Core traits for the event bus system

use async_trait::async_trait;
use std::pin::Pin;
use futures::Stream;
use std::collections::HashMap;

use crate::core::{EventEnvelope, EventQuery, EventTriggerRule, ToolInvocation};
use crate::core::error::EventBusError;

/// Result type for event bus operations
pub type EventBusResult<T> = Result<T, EventBusError>;

/// Core trait for event bus functionality
/// 
/// This trait defines the fundamental operations that any event bus implementation
/// must provide: publishing events, subscribing to topics, and querying events.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to the bus
    async fn emit(&self, event: EventEnvelope) -> EventBusResult<()>;
    
    /// Query events based on criteria
    async fn poll(&self, query: EventQuery) -> EventBusResult<Vec<EventEnvelope>>;
    
    /// Subscribe to a topic and receive events as a stream
    async fn subscribe(&self, topic: &str) -> EventBusResult<Pin<Box<dyn Stream<Item = EventEnvelope> + Send>>>;
    
    /// Get list of all available topics
    async fn list_topics(&self) -> EventBusResult<Vec<String>>;
    
    /// Get bus statistics
    async fn get_stats(&self) -> EventBusResult<BusStats>;
    
    /// Emit multiple events in batch for better performance
    async fn emit_batch(&self, events: Vec<EventEnvelope>) -> EventBusResult<()> {
        for event in events {
            self.emit(event).await?;
        }
        Ok(())
    }
}

/// Event storage trait for persistence
/// 
/// This trait provides comprehensive event storage capabilities with support for
/// various query patterns, batch operations, and maintenance operations.
/// 
/// ## Time Semantics
/// 
/// All timestamps are Unix epoch milliseconds (i64). The storage implementation
/// should preserve exact timestamps for consistency across different time zones.
/// 
/// ## Idempotency
/// 
/// The `store` method should be idempotent when called with the same event ID.
/// Subsequent calls with the same ID should either succeed (no-op) or return
/// a specific error indicating the duplicate.
/// 
/// ## Consistency Guarantees
/// 
/// - Events stored via `store` should be immediately visible via `query`
/// - Batch operations should be atomic where possible
/// - Cleanup operations should not interfere with ongoing queries
#[async_trait]
pub trait EventStorage: Send + Sync {
    /// Initialize the storage backend
    /// 
    /// This method should create necessary tables, indexes, and perform
    /// any required schema migrations. It should be idempotent and safe
    /// to call multiple times.
    async fn initialize(&self) -> EventBusResult<()>;
    
    /// Store a single event
    /// 
    /// Should be idempotent for the same event ID. Events with duplicate IDs
    /// should either be ignored or return a specific duplicate error.
    async fn store(&self, event: &EventEnvelope) -> EventBusResult<()>;
    
    /// Store multiple events in batch
    /// 
    /// Implementations should use transactions where possible for atomicity.
    /// Default implementation calls store() for each event.
    async fn store_batch(&self, events: &[EventEnvelope]) -> EventBusResult<()> {
        for event in events {
            self.store(event).await?;
        }
        Ok(())
    }
    
    /// Query stored events
    /// 
    /// Should support filtering by topic, time range, TRN, and other criteria.
    /// Results should be ordered by timestamp in descending order (newest first).
    async fn query(&self, query: &EventQuery) -> EventBusResult<Vec<EventEnvelope>>;
    
    /// Get storage statistics
    async fn get_stats(&self) -> EventBusResult<StorageStats>;
    
    /// Cleanup old events based on retention policy
    /// 
    /// Should remove events with timestamp less than the provided threshold.
    /// Returns the number of events that were deleted.
    async fn cleanup(&self, before_timestamp: i64) -> EventBusResult<u64>;
    
    /// Get events for a topic since a given timestamp
    /// 
    /// This is a convenience method for real-time subscriptions and polling.
    async fn get_events_since(&self, topic: &str, since_timestamp: i64, limit: Option<usize>) -> EventBusResult<Vec<EventEnvelope>> {
        let query = EventQuery {
            topic: Some(topic.to_string()),
            since: Some(since_timestamp),
            limit: limit.map(|l| l as u32),
            ..Default::default()
        };
        self.query(&query).await
    }
}

/// Rule engine trait for event-driven automation
#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// Register a new rule
    async fn register_rule(&self, rule: EventTriggerRule) -> EventBusResult<()>;
    
    /// Remove a rule
    async fn remove_rule(&self, rule_id: &str) -> EventBusResult<()>;
    
    /// List all registered rules
    async fn list_rules(&self) -> EventBusResult<Vec<EventTriggerRule>>;
    
    /// Process an event against all rules
    async fn process_event(&self, event: &EventEnvelope) -> EventBusResult<Vec<ToolInvocation>>;
    
    /// Enable or disable a rule
    async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> EventBusResult<()>;
}

/// Rule storage trait for managing event routing rules
#[async_trait]
pub trait RuleStorage: Send + Sync {
    /// Store a new rule
    /// 
    /// # Arguments
    /// * `rule` - The rule to store
    /// 
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Errors
    /// Returns error if rule already exists or storage operation fails
    async fn store_rule(&self, rule: &crate::core::types::Rule) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Update an existing rule
    /// 
    /// # Arguments
    /// * `rule` - The updated rule
    /// 
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Errors
    /// Returns error if rule doesn't exist or storage operation fails
    async fn update_rule(&self, rule: &crate::core::types::Rule) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Retrieve a rule by ID
    /// 
    /// # Arguments
    /// * `rule_id` - The ID of the rule to retrieve
    /// 
    /// # Returns
    /// The rule if found, None otherwise
    ///
    /// # Errors
    /// Returns error if storage operation fails
    async fn get_rule(&self, rule_id: &str) -> Result<Option<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>>;

    /// List all rules, optionally filtered by enabled status
    /// 
    /// # Arguments
    /// * `enabled_only` - If true, return only enabled rules
    /// 
    /// # Returns
    /// Vector of rules matching the criteria
    ///
    /// # Errors
    /// Returns error if storage operation fails
    async fn list_rules(&self, enabled_only: bool) -> Result<Vec<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>>;

    /// Delete a rule by ID
    /// 
    /// # Arguments
    /// * `rule_id` - The ID of the rule to delete
    /// 
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Errors
    /// Returns error if rule doesn't exist or storage operation fails
    async fn delete_rule(&self, rule_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get rules matching a specific pattern or topic
    /// 
    /// # Arguments
    /// * `pattern` - TRN pattern to match against
    /// 
    /// # Returns
    /// Vector of rules that could match the pattern
    ///
    /// # Errors
    /// Returns error if storage operation fails
    async fn get_matching_rules(&self, pattern: &str) -> Result<Vec<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>>;

    /// Enable or disable a rule
    /// 
    /// # Arguments
    /// * `rule_id` - The ID of the rule to modify
    /// * `enabled` - Whether to enable or disable the rule
    /// 
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Errors
    /// Returns error if rule doesn't exist or storage operation fails
    async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get rules ordered by priority (highest first)
    /// 
    /// # Arguments
    /// * `enabled_only` - If true, return only enabled rules
    /// 
    /// # Returns
    /// Vector of rules ordered by priority
    ///
    /// # Errors
    /// Returns error if storage operation fails
    async fn get_rules_by_priority(&self, enabled_only: bool) -> Result<Vec<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>>;

    /// Count total number of rules
    /// 
    /// # Arguments
    /// * `enabled_only` - If true, count only enabled rules
    /// 
    /// # Returns
    /// Number of rules
    ///
    /// # Errors
    /// Returns error if storage operation fails
    async fn count_rules(&self, enabled_only: bool) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;
}

/// Tool execution trait for handling various tool types
/// 
/// This trait provides a unified interface for executing different types of tools:
/// - OpenAPI/HTTP endpoints
/// - Python scripts and functions
/// - Local command-line tools
/// - Workflow engines
/// - Database operations
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool invocation and return the result
    /// 
    /// The implementation should handle different tool types based on the
    /// tool_id format or metadata and route to appropriate execution engines.
    async fn execute(&self, invocation: &ToolInvocation) -> EventBusResult<ToolExecutionResult>;
    
    /// Check if a tool exists and is available for execution
    async fn is_available(&self, tool_id: &str) -> EventBusResult<bool>;
    
    /// Get metadata about a tool
    async fn get_metadata(&self, tool_id: &str) -> EventBusResult<ToolMetadata>;
    
    /// List all available tools
    async fn list_tools(&self) -> EventBusResult<Vec<ToolMetadata>>;
    
    /// Register a new tool definition
    async fn register_tool(&self, metadata: ToolMetadata) -> EventBusResult<()>;
    
    /// Execute multiple tools in parallel
    async fn execute_batch(&self, invocations: &[ToolInvocation]) -> EventBusResult<Vec<ToolExecutionResult>> {
        let mut results = Vec::new();
        for invocation in invocations {
            let result = self.execute(invocation).await?;
            results.push(result);
        }
        Ok(results)
    }
}

/// Tool invocation trait for legacy compatibility
#[async_trait]
pub trait ToolInvoker: Send + Sync {
    /// Invoke a tool with the given parameters
    async fn invoke_tool(&self, invocation: &ToolInvocation) -> EventBusResult<serde_json::Value>;
    
    /// Check if a tool exists and is available
    async fn tool_exists(&self, tool_id: &str) -> EventBusResult<bool>;
    
    /// Get tool metadata
    async fn get_tool_metadata(&self, tool_id: &str) -> EventBusResult<ToolMetadata>;
}

/// Tool execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionResult {
    /// Invocation ID for tracking
    pub invocation_id: String,
    
    /// Tool ID that was executed
    pub tool_id: String,
    
    /// Execution status
    pub status: ToolExecutionStatus,
    
    /// Result data (if successful)
    pub result: Option<serde_json::Value>,
    
    /// Error information (if failed)
    pub error: Option<String>,
    
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Events generated by the tool execution
    pub generated_events: Vec<EventEnvelope>,
}

/// Tool execution status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ToolExecutionStatus {
    /// Execution completed successfully
    Success,
    
    /// Execution failed with an error
    Failed,
    
    /// Execution is still in progress (for async tools)
    InProgress,
    
    /// Execution was cancelled
    Cancelled,
    
    /// Tool execution timed out
    Timeout,
}

/// Tool metadata definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolMetadata {
    /// Unique tool identifier
    pub id: String,
    
    /// Human-readable tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// Tool version
    pub version: String,
    
    /// Tool type
    pub tool_type: ToolType,
    
    /// Tool execution configuration
    pub config: ToolConfig,
    
    /// Input schema (JSON Schema)
    pub input_schema: Option<serde_json::Value>,
    
    /// Output schema (JSON Schema)
    pub output_schema: Option<serde_json::Value>,
    
    /// Tool tags for categorization
    pub tags: Vec<String>,
    
    /// Whether the tool is currently enabled
    pub enabled: bool,
    
    /// Tool capabilities and requirements
    pub capabilities: ToolCapabilities,
}

/// Tool type enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ToolType {
    /// HTTP/OpenAPI endpoint
    HttpApi {
        base_url: String,
        auth_type: Option<String>,
    },
    
    /// Python script or function
    Python {
        script_path: Option<String>,
        function_name: Option<String>,
        environment: Option<String>,
    },
    
    /// Command line tool
    Command {
        executable: String,
        working_directory: Option<String>,
    },
    
    /// Workflow engine integration
    Workflow {
        engine_type: String,
        workflow_id: String,
    },
    
    /// Database operation
    Database {
        connection_string: String,
        query_type: String,
    },
    
    /// Custom tool type
    Custom {
        handler: String,
        config: HashMap<String, serde_json::Value>,
    },
}

/// Tool execution configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolConfig {
    /// Maximum execution time in seconds
    pub timeout_seconds: u32,
    
    /// Number of retry attempts on failure
    pub retry_attempts: u32,
    
    /// Whether to run asynchronously
    pub async_execution: bool,
    
    /// Resource limits
    pub resource_limits: Option<ResourceLimits>,
    
    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Resource limits for tool execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u32>,
    
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<u32>,
    
    /// Maximum number of file descriptors
    pub max_file_descriptors: Option<u32>,
}

/// Tool capabilities and requirements
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCapabilities {
    /// Whether the tool supports batch execution
    pub supports_batch: bool,
    
    /// Whether the tool supports streaming results
    pub supports_streaming: bool,
    
    /// Whether the tool is stateful
    pub is_stateful: bool,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
    
    /// Supported input formats
    pub input_formats: Vec<String>,
    
    /// Supported output formats
    pub output_formats: Vec<String>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 300, // 5 minutes
            retry_attempts: 3,
            async_execution: false,
            resource_limits: None,
            environment: HashMap::new(),
        }
    }
}

impl Default for ToolCapabilities {
    fn default() -> Self {
        Self {
            supports_batch: false,
            supports_streaming: false,
            is_stateful: false,
            required_permissions: vec![],
            input_formats: vec!["json".to_string()],
            output_formats: vec!["json".to_string()],
        }
    }
}

/// Bus statistics
#[derive(Debug, Clone)]
pub struct BusStats {
    /// Total number of events processed
    pub events_processed: u64,
    
    /// Number of active subscriptions
    pub active_subscriptions: u32,
    
    /// Number of available topics
    pub topic_count: u32,
    
    /// Current events per second
    pub events_per_second: f64,
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total number of events stored
    pub total_events: u64,
    
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    
    /// Number of topics with stored events
    pub topics_count: u32,
    
    /// Oldest event timestamp
    pub oldest_event_timestamp: Option<i64>,
    
    /// Newest event timestamp
    pub newest_event_timestamp: Option<i64>,
}

/// Event listener trait for receiving notifications
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Called when an event is published
    async fn on_event_published(&self, event: &EventEnvelope) -> EventBusResult<()>;
    
    /// Called when a rule is triggered
    async fn on_rule_triggered(&self, rule_id: &str, event: &EventEnvelope) -> EventBusResult<()>;
    
    /// Called when a tool is invoked
    async fn on_tool_invoked(&self, invocation: &ToolInvocation, result: &serde_json::Value) -> EventBusResult<()>;
}

/// Event middleware trait for processing events
#[async_trait]
pub trait EventMiddleware: Send + Sync {
    /// Process an event before it's published
    async fn before_publish(&self, event: &mut EventEnvelope) -> EventBusResult<bool>;
    
    /// Process an event after it's published
    async fn after_publish(&self, event: &EventEnvelope) -> EventBusResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock implementations for testing
    struct MockEventBus;
    
    #[async_trait]
    impl EventBus for MockEventBus {
        async fn emit(&self, _event: EventEnvelope) -> EventBusResult<()> {
            Ok(())
        }
        
        async fn poll(&self, _query: EventQuery) -> EventBusResult<Vec<EventEnvelope>> {
            Ok(vec![])
        }
        
        async fn subscribe(&self, _topic: &str) -> EventBusResult<Pin<Box<dyn Stream<Item = EventEnvelope> + Send>>> {
            use futures::stream;
            Ok(Box::pin(stream::empty()))
        }
        
        async fn list_topics(&self) -> EventBusResult<Vec<String>> {
            Ok(vec![])
        }
        
        async fn get_stats(&self) -> EventBusResult<BusStats> {
            Ok(BusStats {
                events_processed: 0,
                active_subscriptions: 0,
                topic_count: 0,
                events_per_second: 0.0,
            })
        }
    }
    
    #[tokio::test]
    async fn test_event_bus_trait() {
        let bus = MockEventBus;
        let stats = bus.get_stats().await.unwrap();
        assert_eq!(stats.events_processed, 0);
    }
} 