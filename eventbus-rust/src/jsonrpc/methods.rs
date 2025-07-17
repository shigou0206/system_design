//! JSON-RPC method definitions for EventBus operations
//! 
//! This module defines the JSON-RPC method names and parameter structures
//! for all EventBus operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::{EventEnvelope, EventQuery, BusStats};

/// JSON-RPC method names for EventBus operations
pub mod method_names {
    /// Emit a single event
    pub const EMIT: &str = "eventbus.emit";
    
    /// Emit multiple events in batch
    pub const EMIT_BATCH: &str = "eventbus.emit_batch";
    
    /// Query events based on criteria
    pub const POLL: &str = "eventbus.poll";
    
    /// Subscribe to a topic (returns subscription ID)
    pub const SUBSCRIBE: &str = "eventbus.subscribe";
    
    /// Unsubscribe from a topic
    pub const UNSUBSCRIBE: &str = "eventbus.unsubscribe";
    
    /// List all available topics
    pub const LIST_TOPICS: &str = "eventbus.list_topics";
    
    /// Get bus statistics
    pub const GET_STATS: &str = "eventbus.get_stats";
    
    /// Get next events from subscription (for polling-based clients)
    pub const GET_SUBSCRIPTION_EVENTS: &str = "eventbus.get_subscription_events";
}

/// Parameters for emit method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitParams {
    /// Event to emit
    pub event: EventEnvelope,
}

/// Parameters for emit_batch method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitBatchParams {
    /// Events to emit
    pub events: Vec<EventEnvelope>,
}

/// Parameters for poll method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollParams {
    /// Query criteria
    pub query: EventQuery,
}

/// Parameters for subscribe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeParams {
    /// Topic to subscribe to
    pub topic: String,
    /// Optional client ID for tracking
    pub client_id: Option<String>,
}

/// Parameters for unsubscribe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeParams {
    /// Subscription ID to unsubscribe
    pub subscription_id: String,
}

/// Parameters for get_subscription_events method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSubscriptionEventsParams {
    /// Subscription ID
    pub subscription_id: String,
    /// Maximum number of events to return
    pub max_events: Option<usize>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Response for emit method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitResponse {
    /// Success indicator
    pub success: bool,
}

/// Response for emit_batch method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitBatchResponse {
    /// Success indicator
    pub success: bool,
    /// Number of events processed
    pub processed_count: usize,
}

/// Response for poll method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResponse {
    /// Found events
    pub events: Vec<EventEnvelope>,
    /// Total count (may be larger than events.len() if limited)
    pub total_count: usize,
}

/// Response for subscribe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeResponse {
    /// Subscription ID for tracking
    pub subscription_id: String,
    /// Success indicator
    pub success: bool,
}

/// Response for unsubscribe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeResponse {
    /// Success indicator
    pub success: bool,
}

/// Response for list_topics method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTopicsResponse {
    /// Available topics
    pub topics: Vec<String>,
}

/// Response for get_stats method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStatsResponse {
    /// Bus statistics
    pub stats: BusStatsJson,
}

/// Response for get_subscription_events method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSubscriptionEventsResponse {
    /// Events from subscription
    pub events: Vec<EventEnvelope>,
    /// Whether there are more events available
    pub has_more: bool,
}

/// JSON-serializable version of BusStats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusStatsJson {
    /// Total number of events processed
    pub events_processed: u64,
    /// Number of active topics
    pub topic_count: usize,
    /// Number of active subscriptions
    pub active_subscriptions: u32,
    /// Events per second (recent rate)
    pub events_per_second: f64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage statistics
    pub memory_usage: MemoryStatsJson,
}

impl From<BusStats> for BusStatsJson {
    fn from(stats: BusStats) -> Self {
        Self {
            events_processed: stats.events_processed,
            topic_count: stats.topic_count as usize,
            active_subscriptions: stats.active_subscriptions,
            events_per_second: stats.events_per_second,
            uptime_seconds: 0, // Will be filled in by server
            memory_usage: MemoryStatsJson {
                events_in_memory: stats.events_processed as usize,
                estimated_bytes: stats.events_processed as usize * 512,
            },
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsJson {
    /// Events stored in memory
    pub events_in_memory: usize,
    /// Estimated memory usage in bytes
    pub estimated_bytes: usize,
}

/// Error codes for EventBus JSON-RPC errors
pub mod error_codes {
    /// Invalid parameters provided
    pub const INVALID_PARAMS: i32 = -32602;
    
    /// Event storage error
    pub const STORAGE_ERROR: i32 = -32001;
    
    /// Subscription not found
    pub const SUBSCRIPTION_NOT_FOUND: i32 = -32002;
    
    /// Topic not found
    pub const TOPIC_NOT_FOUND: i32 = -32003;
    
    /// Service unavailable
    pub const SERVICE_UNAVAILABLE: i32 = -32004;
    
    /// Rate limit exceeded
    pub const RATE_LIMIT_EXCEEDED: i32 = -32005;
} 