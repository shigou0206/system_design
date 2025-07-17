//! In-memory event storage implementation

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::core::{
    traits::{EventStorage, RuleStorage, EventBusResult},
    types::{EventEnvelope, Rule, EventQuery},
};
use crate::StorageStats;

/// In-memory storage implementation
#[derive(Debug, Clone)]
pub struct MemoryStorage {
    events: Arc<RwLock<HashMap<String, Vec<EventEnvelope>>>>,
    rules: Arc<RwLock<HashMap<String, Rule>>>,
    max_events_per_topic: usize,
}

impl MemoryStorage {
    /// Create new memory storage with default limits
    pub fn new() -> Self {
        Self::with_limits(10000)
    }

    /// Create new memory storage with custom limits
    pub fn with_limits(max_events_per_topic: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(HashMap::new())),
            max_events_per_topic,
        }
    }

    /// Get current event count across all topics
    pub async fn event_count(&self) -> usize {
        let events = self.events.read().await;
        events.values().map(|v| v.len()).sum()
    }

    /// Get current rule count
    pub async fn rule_count(&self) -> usize {
        let rules = self.rules.read().await;
        rules.len()
    }

    /// Clear all events and rules
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        let mut rules = self.rules.write().await;
        events.clear();
        rules.clear();
    }

    /// Cleanup old events (for testing/maintenance)
    pub async fn cleanup_old_events(&self, before: DateTime<Utc>) -> usize {
        let mut events = self.events.write().await;
        let mut removed_count = 0;

        for topic_events in events.values_mut() {
            let original_len = topic_events.len();
            topic_events.retain(|event| event.timestamp > before.timestamp());
            removed_count += original_len - topic_events.len();
        }

        // Remove empty topics
        events.retain(|_, topic_events| !topic_events.is_empty());
        
        removed_count
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStorage for MemoryStorage {
    async fn store(&self, event: &EventEnvelope) -> EventBusResult<()> {
        // Store in topic-specific collection
        {
                    let mut events = self.events.write().await;
            
            events
                .entry(event.topic.clone())
                .or_insert_with(Vec::new)
                .push(event.clone());
        }
        
        // Events are already stored in topic-specific collections above
        
        Ok(())
    }
    
    async fn query(&self, query: &EventQuery) -> EventBusResult<Vec<EventEnvelope>> {
        let events = self.events.read().await;
        
        // Collect all events from all topics
        let all_events: Vec<&EventEnvelope> = events.values().flatten().collect();
        
        let mut filtered_events: Vec<EventEnvelope> = all_events
            .iter()
            .filter(|&event| {
                // Filter by topic if specified
                if let Some(ref topic_pattern) = query.topic {
                    if !event.matches_topic(topic_pattern) {
                        return false;
                    }
                }
                
                // Filter by timestamp range
                if let Some(since) = query.since {
                    if event.timestamp < since {
                        return false;
                    }
                }
                
                if let Some(until) = query.until {
                    if event.timestamp >= until {
                        return false;
                    }
                }
                
                // Filter by source TRN
                if let Some(ref source_trn) = query.source_trn {
                    if event.source_trn.as_ref() != Some(source_trn) {
                        return false;
                    }
                }
                
                // Filter by target TRN
                if let Some(ref target_trn) = query.target_trn {
                    if event.target_trn.as_ref() != Some(target_trn) {
                        return false;
                    }
                }
                
                // Filter by correlation ID
                if let Some(ref correlation_id) = query.correlation_id {
                    if event.correlation_id.as_ref() != Some(correlation_id) {
                        return false;
                    }
                }
                
                true
            })
            .map(|&event| event.clone())
            .collect();
        
        // Sort by timestamp (newest first)
        filtered_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply pagination
        if let Some(offset) = query.offset {
            let offset = offset as usize;
            if offset >= filtered_events.len() {
                return Ok(vec![]);
            }
            filtered_events = filtered_events.into_iter().skip(offset).collect();
        }
        
        if let Some(limit) = query.limit {
            let limit = limit as usize;
            filtered_events.truncate(limit);
        }
        
        Ok(filtered_events)
    }
    
    async fn get_stats(&self) -> EventBusResult<StorageStats> {
        let events = self.events.read().await;
        
        let topics_count = events.len() as u32;
        
        // Collect all events from all topics
        let all_events: Vec<&EventEnvelope> = events.values().flatten().collect();
        
        let (oldest_timestamp, newest_timestamp) = if all_events.is_empty() {
            (None, None)
        } else {
            let mut timestamps: Vec<i64> = all_events.iter().map(|e| e.timestamp).collect();
            timestamps.sort();
            (timestamps.first().copied(), timestamps.last().copied())
        };
        
        // Estimate storage size (rough approximation)
        let storage_size_bytes = all_events.iter()
            .map(|event| {
                // Rough estimate: JSON size + overhead
                serde_json::to_string(event).unwrap_or_default().len() + 100
            })
            .sum::<usize>() as u64;
        
        Ok(StorageStats {
            total_events: all_events.len() as u64,
            storage_size_bytes,
            topics_count,
            oldest_event_timestamp: oldest_timestamp,
            newest_event_timestamp: newest_timestamp,
        })
    }
    
    async fn initialize(&self) -> EventBusResult<()> {
        // Memory storage doesn't need initialization
        Ok(())
    }
    
    async fn cleanup(&self, before_timestamp: i64) -> EventBusResult<u64> {
        let mut removed_count = 0;
        
        // Clean up topic-specific events
        {
            let mut events = self.events.write().await;
            
            for topic_events in events.values_mut() {
                let initial_len = topic_events.len();
                topic_events.retain(|event| event.timestamp >= before_timestamp);
                removed_count += (initial_len - topic_events.len()) as u64;
            }
            
            // Remove empty topics
            events.retain(|_, topic_events| !topic_events.is_empty());
        }
        
        Ok(removed_count)
    }
}

#[async_trait]
impl RuleStorage for MemoryStorage {
    async fn store_rule(&self, rule: &Rule) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        
        if rules.contains_key(&rule.id) {
            return Err(format!("Rule with ID '{}' already exists", rule.id).into());
        }
        
        rules.insert(rule.id.clone(), rule.clone());
        Ok(())
    }

    async fn update_rule(&self, rule: &Rule) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        
        if !rules.contains_key(&rule.id) {
            return Err(format!("Rule with ID '{}' not found", rule.id).into());
        }
        
        let mut updated_rule = rule.clone();
        updated_rule.updated_at = Utc::now();
        rules.insert(rule.id.clone(), updated_rule);
        Ok(())
    }

    async fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        Ok(rules.get(rule_id).cloned())
    }

    async fn list_rules(&self, enabled_only: bool) -> Result<Vec<Rule>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        let mut result: Vec<Rule> = rules.values()
            .filter(|rule| !enabled_only || rule.enabled)
            .cloned()
            .collect();
        
        // Sort by priority (highest first), then by created_at
        result.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        
        Ok(result)
    }

    async fn delete_rule(&self, rule_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        
        if rules.remove(rule_id).is_none() {
            return Err(format!("Rule with ID '{}' not found", rule_id).into());
        }
        
        Ok(())
    }

    async fn get_matching_rules(&self, pattern: &str) -> Result<Vec<Rule>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        
        // For pattern matching, we need to check if rule patterns could match this pattern
        // This is a simplified implementation - could be optimized with pattern indexing
        let result: Vec<Rule> = rules.values()
            .filter(|rule| {
                rule.enabled && (
                    rule.pattern == pattern ||
                    rule.pattern.contains('*') ||
                    pattern.contains('*')
                )
            })
            .cloned()
            .collect();
        
        Ok(result)
    }

    async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        
        if let Some(rule) = rules.get_mut(rule_id) {
            rule.enabled = enabled;
            rule.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Rule with ID '{}' not found", rule_id).into())
        }
    }

    async fn get_rules_by_priority(&self, enabled_only: bool) -> Result<Vec<Rule>, Box<dyn std::error::Error + Send + Sync>> {
        // This is the same as list_rules since we already sort by priority
        self.list_rules(enabled_only).await
    }

    async fn count_rules(&self, enabled_only: bool) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        let count = rules.values()
            .filter(|rule| !enabled_only || rule.enabled)
            .count() as u64;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_memory_storage_basic() {
        let storage = MemoryStorage::new();
        
        // Test initialization
        assert!(storage.initialize().await.is_ok());
        assert_eq!(storage.event_count().await, 0);
        
        // Test storing an event
        let event = EventEnvelope::new("test.topic", json!({"message": "hello"}));
        assert!(storage.store(&event).await.is_ok());
        assert_eq!(storage.event_count().await, 1);
        
        // Test querying events
        let query = EventQuery::new().with_topic("test.topic");
        let results = storage.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].topic, "test.topic");
    }
    
    #[tokio::test]
    async fn test_memory_storage_filtering() {
        let storage = MemoryStorage::new();
        
        // Store multiple events
        let event1 = EventEnvelope::new("user.login", json!({"user": "alice"}))
            .set_trn(Some("trn:user:alice".to_string()), None);
        let event2 = EventEnvelope::new("user.logout", json!({"user": "bob"}))
            .set_trn(Some("trn:user:bob".to_string()), None);
        
        storage.store(&event1).await.unwrap();
        storage.store(&event2).await.unwrap();
        
        // Test topic filtering
        let query = EventQuery::new().with_topic("user.login");
        let results = storage.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].topic, "user.login");
        
        // Test TRN filtering
        let query = EventQuery::new();
        let mut query = query;
        query.source_trn = Some("trn:user:alice".to_string());
        let results = storage.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].payload["user"], "alice");
    }
    
    #[tokio::test]
    async fn test_memory_storage_cleanup() {
        let storage = MemoryStorage::new();
        
        // Store events with different timestamps
        let mut event1 = EventEnvelope::new("test", json!({"id": 1}));
        event1.timestamp = 1000;
        
        let mut event2 = EventEnvelope::new("test", json!({"id": 2}));
        event2.timestamp = 2000;
        
        storage.store(&event1).await.unwrap();
        storage.store(&event2).await.unwrap();
        
        assert_eq!(storage.event_count().await, 2);
        
        // Cleanup events before timestamp 1500
        let removed = storage.cleanup(1500).await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(storage.event_count().await, 1);
        
        // Verify only the newer event remains
        let query = EventQuery::new();
        let results = storage.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].payload["id"], 2);
    }
    
    #[tokio::test]
    async fn test_memory_storage_stats() {
        let storage = MemoryStorage::new();
        
        // Test empty storage stats
        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.topics_count, 0);
        assert!(stats.oldest_event_timestamp.is_none());
        assert!(stats.newest_event_timestamp.is_none());
        
        // Add some events
        let event1 = EventEnvelope::new("topic1", json!({"data": "test1"}));
        let event2 = EventEnvelope::new("topic2", json!({"data": "test2"}));
        
        storage.store(&event1).await.unwrap();
        storage.store(&event2).await.unwrap();
        
        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.topics_count, 2);
        assert!(stats.oldest_event_timestamp.is_some());
        assert!(stats.newest_event_timestamp.is_some());
        assert!(stats.storage_size_bytes > 0);
    }
} 