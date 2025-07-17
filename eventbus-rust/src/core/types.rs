//! Core data types for the event bus system

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority event
    Low = 0,
    /// Normal priority event (default)
    Normal = 1,
    /// High priority event
    High = 2,
    /// Critical priority event
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

/// Event envelope containing all event metadata and payload
/// 
/// This is the core data structure that represents an event in the system.
/// It includes TRN integration for source/target tracking and correlation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    /// Unique event identifier
    pub event_id: String,
    
    /// Event topic for routing and subscription
    pub topic: String,
    
    /// Event payload (arbitrary JSON data)
    pub payload: serde_json::Value,
    
    /// Unix timestamp when the event was created
    pub timestamp: i64,
    
    /// Optional event metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    
    // TRN Integration fields
    /// TRN of the event source (who generated this event)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_trn: Option<String>,
    
    /// TRN of the target resource (what this event is about)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_trn: Option<String>,
    
    /// Correlation ID for distributed tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    
    // Reliability fields
    /// Sequence number for ordering (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<u64>,
    
    /// Event priority (higher number = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 {
    100 // Normal priority
}

impl EventEnvelope {
    /// Create a new event envelope
    pub fn new(topic: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            topic: topic.into(),
            payload,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            metadata: None,
            source_trn: None,
            target_trn: None,
            correlation_id: None,
            sequence_number: None,
            priority: default_priority(),
        }
    }
    
    /// Create a new event with TRN information
    pub fn with_trn(
        topic: impl Into<String>,
        payload: serde_json::Value,
        source_trn: Option<String>,
        target_trn: Option<String>,
    ) -> Self {
        let mut event = Self::new(topic, payload);
        event.source_trn = source_trn;
        event.target_trn = target_trn;
        event
    }
    
    /// Set TRN information for an existing event (method style)
    pub fn set_trn(mut self, source_trn: Option<String>, target_trn: Option<String>) -> Self {
        self.source_trn = source_trn;
        self.target_trn = target_trn;
        self
    }
    
    /// Set correlation ID for tracing
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
    
    /// Set event priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set sequence number
    pub fn with_sequence(mut self, sequence_number: u64) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Check if event matches topic pattern
    pub fn matches_topic(&self, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        // Simple wildcard matching for now
        // TODO: Implement more sophisticated pattern matching
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            self.topic.starts_with(prefix)
        } else {
            self.topic == pattern
        }
    }
}

/// Tool invocation request triggered by rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInvocation {
    /// Tool identifier as TRN string
    /// Example: "trn:user:alice:openapi:github-api:get-repo:v1"
    pub tool_id: String,
    
    /// Input parameters for the tool
    pub input: serde_json::Value,
    
    /// Optional context for tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
    
    /// Timeout for tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl ToolInvocation {
    /// Create a new tool invocation
    pub fn new(tool_id: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            tool_id: tool_id.into(),
            input,
            context: None,
            timeout_ms: None,
        }
    }
    
    /// Set execution context
    pub fn with_context(mut self, context: HashMap<String, serde_json::Value>) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Set execution timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// Event trigger rule for automated responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventTriggerRule {
    /// Unique rule identifier
    pub id: String,
    
    /// Topic pattern to match
    pub topic: String,
    
    /// Field matching criteria (simple key-value for now)
    pub match_fields: HashMap<String, serde_json::Value>,
    
    /// Action to take when rule matches
    pub action: RuleAction,
    
    /// Rule priority (higher number = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
    
    /// Whether the rule is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl EventTriggerRule {
    /// Create a new trigger rule
    pub fn new(
        id: impl Into<String>,
        topic: impl Into<String>,
        action: RuleAction,
    ) -> Self {
        Self {
            id: id.into(),
            topic: topic.into(),
            match_fields: HashMap::new(),
            action,
            priority: default_priority(),
            enabled: true,
        }
    }
    
    /// Add a field matching criterion
    pub fn with_match_field(
        mut self,
        field: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        self.match_fields.insert(field.into(), value);
        self
    }
    
    /// Set rule priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Check if this rule matches the given event
    pub fn matches(&self, event: &EventEnvelope) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check topic match
        if !event.matches_topic(&self.topic) {
            return false;
        }
        
        // Check field matches
        for (field, expected_value) in &self.match_fields {
            let actual_value = match field.as_str() {
                "source_trn" => event.source_trn.as_ref().map(|s| serde_json::Value::String(s.clone())),
                "target_trn" => event.target_trn.as_ref().map(|s| serde_json::Value::String(s.clone())),
                "correlation_id" => event.correlation_id.as_ref().map(|s| serde_json::Value::String(s.clone())),
                "priority" => Some(serde_json::Value::Number(event.priority.into())),
                _ => {
                    // Try to extract from payload
                    event.payload.get(field).cloned()
                }
            };
            
            if actual_value.as_ref() != Some(expected_value) {
                return false;
            }
        }
        
        true
    }
}

/// Actions that can be triggered by rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum RuleAction {
    /// Invoke a tool with the given parameters
    InvokeTool {
        tool_id: String,
        input: serde_json::Value,
    },
    
    /// Emit a new event
    EmitEvent {
        topic: String,
        payload: serde_json::Value,
    },
    
    /// Execute multiple actions
    Sequence {
        actions: Vec<RuleAction>,
    },
    
    /// Forward event to another topic
    Forward {
        target_topic: String,
        transform: Option<serde_json::Value>,
    },
    
    /// Transform the event data
    Transform {
        transformation: serde_json::Value,
    },
    
    /// Execute a tool
    ExecuteTool {
        tool_name: String,
        parameters: serde_json::Value,
    },
    
    /// Send webhook notification
    Webhook {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: serde_json::Value,
    },
    
    /// Log the event
    Log {
        level: String,
        message: String,
    },
    
    /// Custom action with arbitrary data
    Custom {
        action_type: String,
        data: serde_json::Value,
    },
}

/// Event query parameters for polling events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventQuery {
    /// Topic pattern to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    
    /// Minimum timestamp (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    
    /// Maximum timestamp (exclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
    
    /// Source TRN filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_trn: Option<String>,
    
    /// Target TRN filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_trn: Option<String>,
    
    /// Correlation ID filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    
    /// Maximum number of events to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    
    /// Offset for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl EventQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self {
            topic: None,
            since: None,
            until: None,
            source_trn: None,
            target_trn: None,
            correlation_id: None,
            limit: None,
            offset: None,
        }
    }
    
    /// Filter by topic
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }
    
    /// Filter by timestamp range
    pub fn with_time_range(mut self, since: Option<i64>, until: Option<i64>) -> Self {
        self.since = since;
        self.until = until;
        self
    }
    
    /// Set pagination
    pub fn with_pagination(mut self, limit: u32, offset: u32) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for EventQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// A rule definition for event routing and processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// TRN pattern to match against
    pub pattern: String,
    /// Action to take when rule matches
    pub action: RuleAction,
    /// Priority (higher number = higher priority)
    pub priority: i32,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Optional description
    pub description: Option<String>,
    /// Metadata for the rule
    pub metadata: HashMap<String, String>,
    /// When the rule was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the rule was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}



impl Rule {
    /// Create a new rule
    pub fn new(
        id: String,
        name: String,
        pattern: String,
        action: RuleAction,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            pattern,
            action,
            priority: 0,
            enabled: true,
            description: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set priority for the rule
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set description for the rule
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add metadata to the rule
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this rule matches the given event
    pub fn matches(&self, event: &EventEnvelope) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Simple pattern matching - for now, just check if the pattern matches the topic
        // In a real implementation, you'd want more sophisticated TRN pattern matching
        if self.pattern == "*" {
            return true;
        }
        
        // For now, use simple glob-style matching
        if self.pattern.ends_with("*") {
            let prefix = &self.pattern[..self.pattern.len() - 1];
            event.topic.starts_with(prefix)
        } else {
            event.topic == self.pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_event_envelope_creation() {
        let event = EventEnvelope::new("test.topic", json!({"message": "hello"}));
        
        assert_eq!(event.topic, "test.topic");
        assert_eq!(event.payload, json!({"message": "hello"}));
        assert_eq!(event.priority, 100);
        assert!(event.event_id.len() > 0);
        assert!(event.timestamp > 0);
    }
    
    #[test]
    fn test_event_topic_matching() {
        let event = EventEnvelope::new("user.login", json!({}));
        
        assert!(event.matches_topic("user.login"));
        assert!(event.matches_topic("user.*"));
        assert!(event.matches_topic("*"));
        assert!(!event.matches_topic("user.logout"));
        assert!(!event.matches_topic("admin.*"));
    }
    
    #[test]
    fn test_rule_matching() {
        let event = EventEnvelope::new("user.login", json!({"user_id": "123"}))
            .set_trn(Some("trn:user:alice".to_string()), None);
        
        let rule = EventTriggerRule::new(
            "test-rule",
            "user.*",
            RuleAction::EmitEvent {
                topic: "analytics.event".to_string(),
                payload: json!({"type": "login"}),
            },
        )
        .with_match_field("user_id", json!("123"));
        
        assert!(rule.matches(&event));
        
        // Test non-matching rule
        let rule2 = EventTriggerRule::new(
            "test-rule-2",
            "user.*",
            RuleAction::EmitEvent {
                topic: "analytics.event".to_string(),
                payload: json!({"type": "login"}),
            },
        )
        .with_match_field("user_id", json!("456"));
        
        assert!(!rule2.matches(&event));

    }
} 

/// Builder for constructing EventEnvelope instances
/// 
/// This builder provides a fluent interface for creating events with validation
/// and sensible defaults.
#[derive(Debug, Clone)]
pub struct EventEnvelopeBuilder {
    topic: Option<String>,
    payload: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
    source_trn: Option<String>,
    target_trn: Option<String>,
    correlation_id: Option<String>,
    sequence_number: Option<u64>,
    priority: EventPriority,
    timestamp: Option<i64>,
}

impl EventEnvelopeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            topic: None,
            payload: None,
            metadata: None,
            source_trn: None,
            target_trn: None,
            correlation_id: None,
            sequence_number: None,
            priority: EventPriority::Normal,
            timestamp: None,
        }
    }

    /// Set the topic for the event
    pub fn topic<S: Into<String>>(mut self, topic: S) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Set the payload for the event
    pub fn payload<T: serde::Serialize>(mut self, payload: T) -> Result<Self, serde_json::Error> {
        self.payload = Some(serde_json::to_value(payload)?);
        Ok(self)
    }

    /// Set the payload as raw JSON value
    pub fn payload_json(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Set metadata as JSON value
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add metadata from key-value pairs
    pub fn metadata_kv<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self {
        let mut map = match self.metadata {
            Some(serde_json::Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };
        map.insert(key.into(), serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
        self.metadata = Some(serde_json::Value::Object(map));
        self
    }

    /// Set the source TRN
    pub fn source_trn<S: Into<String>>(mut self, source_trn: S) -> Self {
        self.source_trn = Some(source_trn.into());
        self
    }

    /// Set the target TRN
    pub fn target_trn<S: Into<String>>(mut self, target_trn: S) -> Self {
        self.target_trn = Some(target_trn.into());
        self
    }

    /// Set the correlation ID
    pub fn correlation_id<S: Into<String>>(mut self, correlation_id: S) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Set the sequence number
    pub fn sequence_number(mut self, sequence_number: u64) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    /// Set the priority
    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set priority to high
    pub fn high_priority(mut self) -> Self {
        self.priority = EventPriority::High;
        self
    }

    /// Set priority to low
    pub fn low_priority(mut self) -> Self {
        self.priority = EventPriority::Low;
        self
    }

    /// Set priority to critical
    pub fn critical_priority(mut self) -> Self {
        self.priority = EventPriority::Critical;
        self
    }

    /// Set the timestamp
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set timestamp to now
    pub fn now(mut self) -> Self {
        self.timestamp = Some(chrono::Utc::now().timestamp());
        self
    }

    /// Build the EventEnvelope
    /// 
    /// # Errors
    /// Returns an error if required fields are missing or invalid
    pub fn build(self) -> Result<EventEnvelope, crate::core::error::EventBusError> {
        let topic = self.topic.ok_or_else(|| {
            crate::core::error::EventBusError::validation("Topic is required")
        })?;

        let payload = self.payload.ok_or_else(|| {
            crate::core::error::EventBusError::validation("Payload is required")
        })?;

        // Validate topic format
        crate::utils::validate_trn(&topic).map_err(|e| {
            crate::core::error::EventBusError::validation(format!("Invalid topic: {}", e))
        })?;

        // Validate TRNs if provided
        if let Some(ref source) = self.source_trn {
            crate::utils::validate_trn(source).map_err(|e| {
                crate::core::error::EventBusError::validation(format!("Invalid source TRN: {}", e))
            })?;
        }

        if let Some(ref target) = self.target_trn {
            crate::utils::validate_trn(target).map_err(|e| {
                crate::core::error::EventBusError::validation(format!("Invalid target TRN: {}", e))
            })?;
        }

        let mut event = EventEnvelope::new(topic, payload);
        
        // Set optional fields
        event.metadata = self.metadata;
        event.source_trn = self.source_trn;
        event.target_trn = self.target_trn;
        event.correlation_id = self.correlation_id;
        event.sequence_number = self.sequence_number;
        event.priority = self.priority as u32;
        
        if let Some(timestamp) = self.timestamp {
            event.timestamp = timestamp;
        }

        Ok(event)
    }
}

impl Default for EventEnvelopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience methods for creating builders with common patterns
impl EventEnvelopeBuilder {
    /// Create a builder for a system event
    pub fn system_event<S: Into<String>, T: serde::Serialize>(
        topic: S, 
        payload: T
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new()
            .topic(format!("system.{}", topic.into()))
            .payload(payload)?
            .metadata_kv("event_type", "system")
            .priority(EventPriority::High))
    }

    /// Create a builder for a workflow event
    pub fn workflow_event<S: Into<String>, T: serde::Serialize>(
        workflow_id: S,
        event_type: S,
        payload: T
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new()
            .topic(format!("workflow.{}.{}", workflow_id.into(), event_type.into()))
            .payload(payload)?
            .metadata_kv("event_type", "workflow"))
    }

    /// Create a builder for a user event
    pub fn user_event<S: Into<String>, T: serde::Serialize>(
        user_id: S,
        event_type: S,
        payload: T
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new()
            .topic(format!("user.{}.{}", user_id.into(), event_type.into()))
            .payload(payload)?
            .metadata_kv("event_type", "user"))
    }

    /// Create a builder for an error event
    pub fn error_event<S: Into<String>, T: serde::Serialize>(
        component: S,
        error: T
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new()
            .topic(format!("error.{}", component.into()))
            .payload(error)?
            .metadata_kv("event_type", "error")
            .priority(EventPriority::High))
    }

    /// Create a builder for a metric event
    pub fn metric_event<S: Into<String>, T: serde::Serialize>(
        metric_name: S,
        value: T
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new()
            .topic(format!("metrics.{}", metric_name.into()))
            .payload(value)?
            .metadata_kv("event_type", "metric")
            .priority(EventPriority::Low))
    }
} 