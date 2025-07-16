//! Event System Module
//! 
//! Provides event-driven architecture with pub/sub patterns for system-wide events

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, error};
use uuid::Uuid;

/// Event types in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    JsonRpcRequest,
    JsonRpcResponse,
    WebSocketConnect,
    WebSocketDisconnect,
    WebSocketMessage,
    SseConnect,
    SseDisconnect,
    SystemStats,
    UserAction,
    ServiceStart,
    ServiceStop,
    Custom(String),
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// System event structure
#[derive(Debug, Clone, Serialize)]
pub struct SystemEvent {
    pub id: String,
    pub event_type: EventType,
    pub level: EventLevel,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub data: Value,
    pub tags: Vec<String>,
}

impl SystemEvent {
    #[allow(dead_code)]
    pub fn new(event_type: EventType, level: EventLevel, source: String, data: Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            level,
            timestamp: chrono::Utc::now(),
            source,
            data,
            tags: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Event subscriber callback
#[allow(dead_code)]
pub type EventCallback = Box<dyn Fn(&SystemEvent) + Send + Sync>;

/// Event subscriber
#[allow(dead_code)]
pub struct EventSubscriber {
    pub id: String,
    pub event_types: Vec<EventType>,
    pub sender: mpsc::UnboundedSender<SystemEvent>,
}

/// Event bus for pub/sub functionality
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<String, EventSubscriber>>>,
    event_log: Arc<RwLock<Vec<SystemEvent>>>,
    max_log_size: usize,
}

#[allow(dead_code)]
impl EventBus {
    pub fn new(max_log_size: usize) -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
            max_log_size,
        }
    }

    /// Subscribe to specific event types
    pub async fn subscribe(&self, event_types: Vec<EventType>) -> (String, mpsc::UnboundedReceiver<SystemEvent>) {
        let subscriber_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        let subscriber = EventSubscriber {
            id: subscriber_id.clone(),
            event_types,
            sender: tx,
        };

        self.subscribers.write().await.insert(subscriber_id.clone(), subscriber);
        info!("New event subscriber registered: {}", subscriber_id);

        (subscriber_id, rx)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscriber_id: &str) {
        if self.subscribers.write().await.remove(subscriber_id).is_some() {
            info!("Event subscriber unregistered: {}", subscriber_id);
        }
    }

    /// Publish event to all matching subscribers
    pub async fn publish(&self, event: SystemEvent) {
        debug!("Publishing event: {:?} from {}", event.event_type, event.source);

        // Add to event log
        {
            let mut log = self.event_log.write().await;
            log.push(event.clone());
            
            // Trim log if it exceeds max size
            if log.len() > self.max_log_size {
                let excess = log.len() - self.max_log_size;
                log.drain(0..excess);
            }
        }

        // Send to matching subscribers
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.values() {
            if subscriber.event_types.contains(&event.event_type) || 
               subscriber.event_types.contains(&EventType::Custom("*".to_string())) {
                if let Err(e) = subscriber.sender.send(event.clone()) {
                    error!("Failed to send event to subscriber {}: {}", subscriber.id, e);
                }
            }
        }
    }

    /// Get recent events from log
    pub async fn get_recent_events(&self, limit: Option<usize>) -> Vec<SystemEvent> {
        let log = self.event_log.read().await;
        let count = limit.unwrap_or(100).min(log.len());
        log.iter().rev().take(count).cloned().collect()
    }

    /// Get events by type
    pub async fn get_events_by_type(&self, event_type: &EventType, limit: Option<usize>) -> Vec<SystemEvent> {
        let log = self.event_log.read().await;
        let count = limit.unwrap_or(100);
        
        log.iter()
            .rev()
            .filter(|event| &event.event_type == event_type)
            .take(count)
            .cloned()
            .collect()
    }

    /// Get events by level
    pub async fn get_events_by_level(&self, level: &EventLevel, limit: Option<usize>) -> Vec<SystemEvent> {
        let log = self.event_log.read().await;
        let count = limit.unwrap_or(100);
        
        log.iter()
            .rev()
            .filter(|event| std::mem::discriminant(&event.level) == std::mem::discriminant(level))
            .take(count)
            .cloned()
            .collect()
    }

    /// Get subscriber count
    pub async fn get_subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }

    /// Get event statistics
    pub async fn get_event_stats(&self) -> Value {
        let log = self.event_log.read().await;
        let total_events = log.len();
        
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut level_counts: HashMap<String, usize> = HashMap::new();
        
        for event in log.iter() {
            let type_name = format!("{:?}", event.event_type);
            let level_name = format!("{:?}", event.level);
            
            *type_counts.entry(type_name).or_insert(0) += 1;
            *level_counts.entry(level_name).or_insert(0) += 1;
        }
        
        serde_json::json!({
            "total_events": total_events,
            "max_log_size": self.max_log_size,
            "active_subscribers": self.get_subscriber_count().await,
            "event_types": type_counts,
            "event_levels": level_counts,
            "oldest_event": log.first().map(|e| e.timestamp),
            "newest_event": log.last().map(|e| e.timestamp)
        })
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_EVENT_BUS: EventBus = EventBus::new(1000);
}

/// Helper functions for common events
#[allow(dead_code)]
/// Publish JsonRPC request event
pub async fn publish_jsonrpc_request(method: &str, params: &Value, request_id: &str) {
    let event = SystemEvent::new(
        EventType::JsonRpcRequest,
        EventLevel::Info,
        "jsonrpc-server".to_string(),
        serde_json::json!({
            "method": method,
            "params": params,
            "request_id": request_id
        })
    ).with_tags(vec!["jsonrpc".to_string(), "request".to_string()]);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Publish JsonRPC response event
pub async fn publish_jsonrpc_response(method: &str, response: &Value, success: bool, request_id: &str) {
    let level = if success { EventLevel::Info } else { EventLevel::Error };
    
    let event = SystemEvent::new(
        EventType::JsonRpcResponse,
        level,
        "jsonrpc-server".to_string(),
        serde_json::json!({
            "method": method,
            "response": response,
            "success": success,
            "request_id": request_id
        })
    ).with_tags(vec!["jsonrpc".to_string(), "response".to_string()]);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Publish WebSocket connection event
pub async fn publish_websocket_connect(connection_id: &str, client_info: &Value) {
    let event = SystemEvent::new(
        EventType::WebSocketConnect,
        EventLevel::Info,
        "websocket-server".to_string(),
        serde_json::json!({
            "connection_id": connection_id,
            "client_info": client_info
        })
    ).with_tags(vec!["websocket".to_string(), "connection".to_string()]);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Publish WebSocket disconnect event
pub async fn publish_websocket_disconnect(connection_id: &str, reason: &str) {
    let event = SystemEvent::new(
        EventType::WebSocketDisconnect,
        EventLevel::Info,
        "websocket-server".to_string(),
        serde_json::json!({
            "connection_id": connection_id,
            "reason": reason
        })
    ).with_tags(vec!["websocket".to_string(), "disconnection".to_string()]);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Publish SSE connection event
pub async fn publish_sse_connect(connection_id: &str, stream_type: &str) {
    let event = SystemEvent::new(
        EventType::SseConnect,
        EventLevel::Info,
        "sse-server".to_string(),
        serde_json::json!({
            "connection_id": connection_id,
            "stream_type": stream_type
        })
    ).with_tags(vec!["sse".to_string(), "connection".to_string()]);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Publish system stats event
pub async fn publish_system_stats(stats: &Value) {
    let event = SystemEvent::new(
        EventType::SystemStats,
        EventLevel::Debug,
        "system-monitor".to_string(),
        stats.clone()
    ).with_tags(vec!["system".to_string(), "stats".to_string()]);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Publish custom event
pub async fn publish_custom_event(event_name: &str, level: EventLevel, source: &str, data: Value, tags: Vec<String>) {
    let event = SystemEvent::new(
        EventType::Custom(event_name.to_string()),
        level,
        source.to_string(),
        data
    ).with_tags(tags);

    GLOBAL_EVENT_BUS.publish(event).await;
}

#[allow(dead_code)]
/// Event stream handler for SSE
pub async fn create_event_stream_handler() -> mpsc::UnboundedReceiver<SystemEvent> {
    let (_subscriber_id, receiver) = GLOBAL_EVENT_BUS.subscribe(vec![
        EventType::JsonRpcRequest,
        EventType::JsonRpcResponse,
        EventType::WebSocketConnect,
        EventType::WebSocketDisconnect,
        EventType::SseConnect,
        EventType::SseDisconnect,
        EventType::SystemStats,
        EventType::Custom("*".to_string()),
    ]).await;

    receiver
}

/// Get event API info
pub async fn get_events_info() -> Value {
    let stats = GLOBAL_EVENT_BUS.get_event_stats().await;
    
    serde_json::json!({
        "event_system": {
            "description": "Global event bus with pub/sub functionality",
            "features": [
                "Real-time event publishing",
                "Type-based subscriptions", 
                "Event logging and history",
                "Statistics and analytics"
            ]
        },
        "available_apis": [
            {
                "endpoint": "/api/events/recent",
                "description": "Get recent events",
                "parameters": "?limit=100"
            },
            {
                "endpoint": "/api/events/stats",
                "description": "Get event statistics"
            },
            {
                "endpoint": "/api/events/types/{type}",
                "description": "Get events by type"
            }
        ],
        "current_stats": stats
    })
} 