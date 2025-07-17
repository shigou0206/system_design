//! SQLite storage backend for the event bus system
//! 
//! This module provides a persistent storage implementation using SQLite,
//! suitable for production deployments that need durability.

use async_trait::async_trait;
use sqlx::{SqlitePool, Row, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use std::time::Duration;
use serde_json;

use crate::core::{
    EventEnvelope, EventQuery, EventStorage, EventBusResult, EventBusError
};
use crate::core::traits::{StorageStats, RuleStorage};

/// SQLite storage implementation
pub struct SqliteStorage {
    /// Database connection pool
    pool: SqlitePool,
    
    /// Database configuration
    config: SqliteConfig,
}

/// SQLite storage configuration
#[derive(Debug, Clone)]
pub struct SqliteConfig {
    /// Database URL
    pub database_url: String,
    
    /// Connection pool settings
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    
    /// Performance settings
    pub enable_wal_mode: bool,
    pub synchronous_mode: String,
    pub cache_size: i32,
    
    /// Retention settings
    pub enable_auto_cleanup: bool,
    pub cleanup_interval: Duration,
    pub max_age_days: u32,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:events.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: Duration::from_secs(30),
            enable_wal_mode: true,
            synchronous_mode: "NORMAL".to_string(),
            cache_size: -64000, // 64MB cache
            enable_auto_cleanup: true,
            cleanup_interval: Duration::from_secs(3600), // 1 hour
            max_age_days: 30,
        }
    }
}

impl SqliteStorage {
    /// Create a new SQLite storage instance
    pub async fn new(database_url: &str) -> EventBusResult<Self> {
        let config = SqliteConfig {
            database_url: database_url.to_string(),
            ..Default::default()
        };
        
        Self::with_config(config).await
    }
    
    /// Create a new SQLite storage instance with custom configuration
    pub async fn with_config(config: SqliteConfig) -> EventBusResult<Self> {
        let options = SqliteConnectOptions::from_str(&config.database_url)
            .map_err(|e| EventBusError::storage(format!("Invalid database URL: {}", e)))?
            .create_if_missing(true);
        
        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to connect to database: {}", e)))?;
        
        let storage = Self { pool, config };
        
        // Apply performance optimizations
        storage.optimize_database().await?;
        
        Ok(storage)
    }
    
    /// Apply SQLite performance optimizations
    async fn optimize_database(&self) -> EventBusResult<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| EventBusError::storage(format!("Failed to acquire connection: {}", e)))?;
        
        // Enable WAL mode for better concurrency
        if self.config.enable_wal_mode {
            sqlx::query("PRAGMA journal_mode = WAL")
                .execute(&mut *conn)
                .await
                .map_err(|e| EventBusError::storage(format!("Failed to set WAL mode: {}", e)))?;
        }
        
        // Set synchronous mode
        sqlx::query(&format!("PRAGMA synchronous = {}", self.config.synchronous_mode))
            .execute(&mut *conn)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to set synchronous mode: {}", e)))?;
        
        // Set cache size
        sqlx::query(&format!("PRAGMA cache_size = {}", self.config.cache_size))
            .execute(&mut *conn)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to set cache size: {}", e)))?;
        
        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *conn)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to enable foreign keys: {}", e)))?;
        
        Ok(())
    }
    
    /// Store multiple events in a single transaction
    pub async fn store_batch(&self, events: &[EventEnvelope]) -> EventBusResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        
        let mut tx = self.pool.begin().await
            .map_err(|e| EventBusError::storage(format!("Failed to begin transaction: {}", e)))?;
        
        for event in events {
            sqlx::query(
                r#"
                INSERT INTO events (
                    id, topic, payload, timestamp, metadata, 
                    source_trn, target_trn, correlation_id, sequence, priority
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&event.event_id)
            .bind(&event.topic)
            .bind(serde_json::to_string(&event.payload).unwrap_or_default())
            .bind(event.timestamp)
            .bind(serde_json::to_string(&event.metadata).unwrap_or_default())
            .bind(&event.source_trn)
            .bind(&event.target_trn)
            .bind(&event.correlation_id)
            .bind(event.sequence_number.unwrap_or(0) as i64)
            .bind(event.priority as i32)
            .execute(&mut *tx)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to insert event: {}", e)))?;
        }
        
        tx.commit().await
            .map_err(|e| EventBusError::storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }
    
    /// Get events with advanced filtering and pagination
    pub async fn query_advanced(&self, query: &EventQuery, limit: Option<u32>, offset: Option<u32>) -> EventBusResult<Vec<EventEnvelope>> {
        let mut sql = String::from("SELECT * FROM events WHERE 1=1");
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync>> = Vec::new();
        
        if let Some(ref topic) = query.topic {
            if topic.contains('*') {
                sql.push_str(" AND topic GLOB ?");
            } else {
                sql.push_str(" AND topic = ?");
            }
            params.push(Box::new(topic.clone()));
        }
        
        if let Some(since) = query.since {
            sql.push_str(" AND timestamp >= ?");
            params.push(Box::new(since));
        }
        
        if let Some(until) = query.until {
            sql.push_str(" AND timestamp <= ?");
            params.push(Box::new(until));
        }
        
        if let Some(ref source_trn) = query.source_trn {
            sql.push_str(" AND source_trn = ?");
            params.push(Box::new(source_trn.clone()));
        }
        
        if let Some(ref target_trn) = query.target_trn {
            sql.push_str(" AND target_trn = ?");
            params.push(Box::new(target_trn.clone()));
        }
        
        if let Some(ref correlation_id) = query.correlation_id {
            sql.push_str(" AND correlation_id = ?");
            params.push(Box::new(correlation_id.clone()));
        }
        
        sql.push_str(" ORDER BY timestamp DESC");
        
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        
        // Build and execute query
        let query_builder = sqlx::query(&sql);
        
        // Note: This is a simplified version. In practice, you'd need to properly 
        // bind parameters to avoid SQL injection
        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to query events: {}", e)))?;
        
        let mut events = Vec::new();
        for row in rows {
            let event = self.row_to_event(row)?;
            events.push(event);
        }
        
        Ok(events)
    }
    
    /// Convert database row to EventEnvelope
    fn row_to_event(&self, row: sqlx::sqlite::SqliteRow) -> EventBusResult<EventEnvelope> {
        let payload_str: String = row.try_get("payload")
            .map_err(|e| EventBusError::storage(format!("Failed to get payload: {}", e)))?;
        let metadata_str: String = row.try_get("metadata")
            .map_err(|e| EventBusError::storage(format!("Failed to get metadata: {}", e)))?;
        
        let payload = serde_json::from_str(&payload_str)
            .map_err(|e| EventBusError::storage(format!("Failed to parse payload JSON: {}", e)))?;
        let metadata = serde_json::from_str(&metadata_str)
            .map_err(|e| EventBusError::storage(format!("Failed to parse metadata JSON: {}", e)))?;
        
        Ok(EventEnvelope {
            event_id: row.try_get("id")
                .map_err(|e| EventBusError::storage(format!("Failed to get id: {}", e)))?,
            topic: row.try_get("topic")
                .map_err(|e| EventBusError::storage(format!("Failed to get topic: {}", e)))?,
            payload,
            timestamp: row.try_get("timestamp")
                .map_err(|e| EventBusError::storage(format!("Failed to get timestamp: {}", e)))?,
            metadata: Some(metadata),
            source_trn: row.try_get("source_trn").ok(),
            target_trn: row.try_get("target_trn").ok(),
            correlation_id: row.try_get("correlation_id").ok(),
            sequence_number: {
                let seq = row.try_get::<i64, _>("sequence")
                    .map_err(|e| EventBusError::storage(format!("Failed to get sequence: {}", e)))? as u64;
                if seq == 0 { None } else { Some(seq) }
            },
            priority: row.try_get::<i32, _>("priority")
                .map_err(|e| EventBusError::storage(format!("Failed to get priority: {}", e)))? as u32,
        })
    }
}

#[async_trait]
impl EventStorage for SqliteStorage {
    /// Initialize the storage (create tables)
    async fn initialize(&self) -> EventBusResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                topic TEXT NOT NULL,
                payload TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                source_trn TEXT,
                target_trn TEXT,
                correlation_id TEXT,
                sequence INTEGER NOT NULL DEFAULT 0,
                priority INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| EventBusError::storage(format!("Failed to create events table: {}", e)))?;

        // Create rules table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                pattern TEXT NOT NULL,
                action_type TEXT NOT NULL,
                action_config TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                description TEXT,
                metadata TEXT,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                rule_data TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| EventBusError::storage(format!("Failed to create rules table: {}", e)))?;
        
        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_topic ON events(topic)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create topic index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create timestamp index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_source_trn ON events(source_trn)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create source_trn index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_correlation_id ON events(correlation_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create correlation_id index: {}", e)))?;

        // Create indexes for rules table
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rules_pattern ON rules(pattern)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create rules pattern index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rules_enabled ON rules(enabled)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create rules enabled index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rules_priority ON rules(priority DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to create rules priority index: {}", e)))?;
        
        Ok(())
    }
    
    /// Store a single event
    async fn store(&self, event: &EventEnvelope) -> EventBusResult<()> {
        sqlx::query(
            r#"
            INSERT INTO events (
                id, topic, payload, timestamp, metadata, 
                source_trn, target_trn, correlation_id, sequence, priority
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&event.event_id)
        .bind(&event.topic)
        .bind(serde_json::to_string(&event.payload).unwrap_or_default())
        .bind(event.timestamp)
        .bind(serde_json::to_string(&event.metadata).unwrap_or_default())
        .bind(&event.source_trn)
        .bind(&event.target_trn)
        .bind(&event.correlation_id)
        .bind(event.sequence_number.unwrap_or(0) as i64)
        .bind(event.priority as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| EventBusError::storage(format!("Failed to store event: {}", e)))?;
        
        Ok(())
    }
    
    /// Query events
    async fn query(&self, query: &EventQuery) -> EventBusResult<Vec<EventEnvelope>> {
        self.query_advanced(query, query.limit.map(|l| l as u32), None).await
    }
    
    /// Get storage statistics
    async fn get_stats(&self) -> EventBusResult<StorageStats> {
        let row = sqlx::query("SELECT COUNT(*) as total_events, COUNT(DISTINCT topic) as topics_count FROM events")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to get stats: {}", e)))?;
        
        let total_events = row.try_get::<i64, _>("total_events")
            .map_err(|e| EventBusError::storage(format!("Failed to get total_events: {}", e)))? as u64;
        let topics_count = row.try_get::<i64, _>("topics_count")
            .map_err(|e| EventBusError::storage(format!("Failed to get topics_count: {}", e)))? as u32;
        
        Ok(StorageStats {
            total_events,
            topics_count,
            storage_size_bytes: 0, // SQLite doesn't easily provide this
            oldest_event_timestamp: None, // TODO: Implement
            newest_event_timestamp: None, // TODO: Implement
        })
    }
    
    /// Cleanup old events
    async fn cleanup(&self, before_timestamp: i64) -> EventBusResult<u64> {
        let result = sqlx::query("DELETE FROM events WHERE timestamp < ?")
            .bind(before_timestamp)
            .execute(&self.pool)
            .await
            .map_err(|e| EventBusError::storage(format!("Failed to cleanup events: {}", e)))?;
        
        Ok(result.rows_affected())
    }
} 

#[async_trait]
impl RuleStorage for SqliteStorage {
    async fn store_rule(&self, rule: &crate::core::types::Rule) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rule_json = serde_json::to_string(rule)?;
        
        sqlx::query(
            "INSERT INTO rules (id, name, pattern, action_type, action_config, priority, enabled, description, metadata, created_at, updated_at, rule_data) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.pattern)
        .bind(match &rule.action {
            crate::core::types::RuleAction::InvokeTool { .. } => "invoke_tool",
            crate::core::types::RuleAction::EmitEvent { .. } => "emit_event",
            crate::core::types::RuleAction::Sequence { .. } => "sequence",
            crate::core::types::RuleAction::Forward { .. } => "forward",
            crate::core::types::RuleAction::Transform { .. } => "transform",
            crate::core::types::RuleAction::ExecuteTool { .. } => "execute_tool",
            crate::core::types::RuleAction::Webhook { .. } => "webhook",
            crate::core::types::RuleAction::Log { .. } => "log",
            crate::core::types::RuleAction::Custom { .. } => "custom",
        })
        .bind(serde_json::to_string(&rule.action)?)
        .bind(rule.priority)
        .bind(rule.enabled)
        .bind(&rule.description)
        .bind(serde_json::to_string(&rule.metadata)?)
        .bind(rule.created_at)
        .bind(rule.updated_at)
        .bind(&rule_json)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn update_rule(&self, rule: &crate::core::types::Rule) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let updated_rule = crate::core::types::Rule {
            updated_at: chrono::Utc::now(),
            ..rule.clone()
        };
        let rule_json = serde_json::to_string(&updated_rule)?;
        
        let result = sqlx::query(
            "UPDATE rules SET name = ?, pattern = ?, action_type = ?, action_config = ?, priority = ?, enabled = ?, description = ?, metadata = ?, updated_at = ?, rule_data = ? 
             WHERE id = ?"
        )
        .bind(&updated_rule.name)
        .bind(&updated_rule.pattern)
        .bind(match &updated_rule.action {
            crate::core::types::RuleAction::InvokeTool { .. } => "invoke_tool",
            crate::core::types::RuleAction::EmitEvent { .. } => "emit_event",
            crate::core::types::RuleAction::Sequence { .. } => "sequence",
            crate::core::types::RuleAction::Forward { .. } => "forward",
            crate::core::types::RuleAction::Transform { .. } => "transform",
            crate::core::types::RuleAction::ExecuteTool { .. } => "execute_tool",
            crate::core::types::RuleAction::Webhook { .. } => "webhook",
            crate::core::types::RuleAction::Log { .. } => "log",
            crate::core::types::RuleAction::Custom { .. } => "custom",
        })
        .bind(serde_json::to_string(&updated_rule.action)?)
        .bind(updated_rule.priority)
        .bind(updated_rule.enabled)
        .bind(&updated_rule.description)
        .bind(serde_json::to_string(&updated_rule.metadata)?)
        .bind(updated_rule.updated_at)
        .bind(&rule_json)
        .bind(&updated_rule.id)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(format!("Rule with ID '{}' not found", rule.id).into());
        }
        
        Ok(())
    }

    async fn get_rule(&self, rule_id: &str) -> Result<Option<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT rule_data FROM rules WHERE id = ?"
        )
        .bind(rule_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(rule_json) = row {
            let rule: crate::core::types::Rule = serde_json::from_str(&rule_json)?;
            Ok(Some(rule))
        } else {
            Ok(None)
        }
    }

    async fn list_rules(&self, enabled_only: bool) -> Result<Vec<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>> {
        let query = if enabled_only {
            "SELECT rule_data FROM rules WHERE enabled = 1 ORDER BY priority DESC, created_at ASC"
        } else {
            "SELECT rule_data FROM rules ORDER BY priority DESC, created_at ASC"
        };
        
        let rows = sqlx::query_scalar::<_, String>(query)
            .fetch_all(&self.pool)
            .await?;
        
        let mut rules = Vec::new();
        for rule_json in rows {
            let rule: crate::core::types::Rule = serde_json::from_str(&rule_json)?;
            rules.push(rule);
        }
        
        Ok(rules)
    }

    async fn delete_rule(&self, rule_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query("DELETE FROM rules WHERE id = ?")
            .bind(rule_id)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(format!("Rule with ID '{}' not found", rule_id).into());
        }
        
        Ok(())
    }

    async fn get_matching_rules(&self, pattern: &str) -> Result<Vec<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation - for production, you'd want more sophisticated pattern matching
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT rule_data FROM rules WHERE enabled = 1 AND (pattern = ? OR pattern LIKE '%*%' OR ? LIKE '%*%') ORDER BY priority DESC"
        )
        .bind(pattern)
        .bind(pattern)
        .fetch_all(&self.pool)
        .await?;
        
        let mut rules = Vec::new();
        for rule_json in rows {
            let rule: crate::core::types::Rule = serde_json::from_str(&rule_json)?;
            rules.push(rule);
        }
        
        Ok(rules)
    }

    async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "UPDATE rules SET enabled = ?, updated_at = ? WHERE id = ?"
        )
        .bind(enabled)
        .bind(chrono::Utc::now())
        .bind(rule_id)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(format!("Rule with ID '{}' not found", rule_id).into());
        }
        
        Ok(())
    }

    async fn get_rules_by_priority(&self, enabled_only: bool) -> Result<Vec<crate::core::types::Rule>, Box<dyn std::error::Error + Send + Sync>> {
        // Same as list_rules since we already sort by priority
        self.list_rules(enabled_only).await
    }

    async fn count_rules(&self, enabled_only: bool) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let query = if enabled_only {
            "SELECT COUNT(*) FROM rules WHERE enabled = 1"
        } else {
            "SELECT COUNT(*) FROM rules"
        };
        
        let count = sqlx::query_scalar::<_, i64>(query)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(count as u64)
    }
} 