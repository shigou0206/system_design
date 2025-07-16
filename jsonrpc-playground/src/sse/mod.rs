//! Server-Sent Events (SSE) Module
//! 
//! Provides SSE-based streaming for JsonRPC responses and real-time updates

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::StreamExt as _;
use tracing::{info, debug, error};
use uuid::Uuid;

use crate::server::AppState;

/// SSE connection parameters
#[derive(Debug, Deserialize)]
pub struct SseParams {
    pub stream_type: Option<String>,
    pub interval_ms: Option<u64>,
    #[allow(dead_code)]
    pub filter: Option<String>,
}

/// SSE stream type
#[derive(Debug, Clone, Serialize)]
pub enum SseStreamType {
    SystemStats,
    JsonRpcEvents,
    DataStream,
    LogStream,
    MetricsStream,
}

/// SSE event message
#[derive(Debug, Clone, Serialize)]
pub struct SseMessage {
    pub id: String,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: Value,
}

/// SSE connection info
#[derive(Debug, Clone)]
pub struct SseConnection {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub stream_type: SseStreamType,
    #[allow(dead_code)]
    pub connected_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub sender: mpsc::UnboundedSender<SseMessage>,
}

/// Global SSE state manager
pub struct SseManager {
    connections: Arc<RwLock<HashMap<String, SseConnection>>>,
    event_bus: mpsc::UnboundedSender<SseMessage>,
}

impl SseManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<SseMessage>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_bus: event_tx,
        };
        (manager, event_rx)
    }

    pub async fn add_connection(&self, connection: SseConnection) {
        self.connections.write().await.insert(connection.id.clone(), connection);
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        self.connections.write().await.remove(connection_id);
    }

    #[allow(dead_code)]
    pub async fn broadcast_event(&self, event: SseMessage) {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            if let Err(e) = conn.sender.send(event.clone()) {
                error!("Failed to send SSE event to connection {}: {}", conn.id, e);
            }
        }
    }

    pub fn send_event(&self, event: SseMessage) {
        if let Err(e) = self.event_bus.send(event) {
            error!("Failed to send event to event bus: {}", e);
        }
    }

    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

lazy_static::lazy_static! {
    static ref SSE_MANAGER: (SseManager, std::sync::Mutex<Option<mpsc::UnboundedReceiver<SseMessage>>>) = {
        let (manager, receiver) = SseManager::new();
        (manager, std::sync::Mutex::new(Some(receiver)))
    };
}

/// SSE endpoint handler
pub async fn sse_handler(
    Query(params): Query<SseParams>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let connection_id = Uuid::new_v4().to_string();
    let stream_type = parse_stream_type(params.stream_type.as_deref());
    
    info!("New SSE connection: {} with stream type: {:?}", connection_id, stream_type);

    let stream = create_sse_stream(connection_id.clone(), stream_type.clone(), params, app_state).await;
    
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
}

/// Parse stream type from string
fn parse_stream_type(stream_type: Option<&str>) -> SseStreamType {
    match stream_type {
        Some("stats") => SseStreamType::SystemStats,
        Some("events") => SseStreamType::JsonRpcEvents,
        Some("data") => SseStreamType::DataStream,
        Some("logs") => SseStreamType::LogStream,
        Some("metrics") => SseStreamType::MetricsStream,
        _ => SseStreamType::SystemStats,
    }
}

/// Create SSE stream based on type
async fn create_sse_stream(
    connection_id: String,
    stream_type: SseStreamType,
    params: SseParams,
    app_state: AppState,
) -> impl Stream<Item = Result<Event, axum::Error>> {
    let (tx, rx) = mpsc::unbounded_channel::<SseMessage>();
    
    // Register connection
    let connection = SseConnection {
        id: connection_id.clone(),
        stream_type: stream_type.clone(),
        connected_at: chrono::Utc::now(),
        sender: tx,
    };
    
    SSE_MANAGER.0.add_connection(connection).await;

    // Start appropriate stream based on type
    match stream_type {
        SseStreamType::SystemStats => {
            start_system_stats_stream(connection_id.clone(), app_state, params.interval_ms).await;
        }
        SseStreamType::JsonRpcEvents => {
            start_jsonrpc_events_stream(connection_id.clone()).await;
        }
        SseStreamType::DataStream => {
            start_data_stream(connection_id.clone(), params.interval_ms).await;
        }
        SseStreamType::LogStream => {
            start_log_stream(connection_id.clone()).await;
        }
        SseStreamType::MetricsStream => {
            start_metrics_stream(connection_id.clone(), app_state).await;
        }
    }

    // Convert receiver to SSE event stream
    let connection_id_for_cleanup = connection_id.clone();
    tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
        .map(move |msg| {
            let event = Event::default()
                .id(msg.id)
                .event(msg.event_type)
                .json_data(&json!({
                    "timestamp": msg.timestamp,
                    "data": msg.data
                }));
            
            match event {
                Ok(e) => Ok(e),
                Err(e) => {
                    error!("Failed to create SSE event: {}", e);
                    Err(axum::Error::new(e))
                }
            }
        })
        .chain(stream::once(async move {
            // Cleanup on stream end
            SSE_MANAGER.0.remove_connection(&connection_id_for_cleanup).await;
            info!("SSE connection closed: {}", connection_id_for_cleanup);
            Err(axum::Error::new(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Stream ended")))
        }))
}

/// Start system stats streaming
async fn start_system_stats_stream(connection_id: String, app_state: AppState, interval_ms: Option<u64>) {
    let interval = Duration::from_millis(interval_ms.unwrap_or(5000));
    let connection_id_clone = connection_id.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval);
        let mut counter = 0u64;
        
        loop {
            interval_timer.tick().await;
            counter += 1;
            
            let stats = app_state.stats.read().await.clone();
            let session_count = app_state.sessions.read().await.len();
            let sse_connections = SSE_MANAGER.0.get_connection_count().await;
            
            let message = SseMessage {
                id: format!("stats-{}", counter),
                event_type: "system-stats".to_string(),
                timestamp: chrono::Utc::now(),
                data: json!({
                    "total_requests": stats.total_requests,
                    "successful_requests": stats.successful_requests,
                    "failed_requests": stats.failed_requests,
                    "average_response_time_ms": stats.average_response_time_ms,
                    "active_sessions": session_count,
                    "sse_connections": sse_connections,
                    "counter": counter,
                    "uptime_seconds": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }),
            };
            
            SSE_MANAGER.0.send_event(message);
            debug!("Sent system stats update #{} for connection {}", counter, connection_id_clone);
        }
    });
}

/// Start JsonRPC events streaming
async fn start_jsonrpc_events_stream(connection_id: String) {
    debug!("Started JsonRPC events stream for connection: {}", connection_id);
    // This will be fed by the JsonRPC handler when requests are processed
}

/// Start data streaming
async fn start_data_stream(connection_id: String, interval_ms: Option<u64>) {
    let interval = Duration::from_millis(interval_ms.unwrap_or(1000));
    let connection_id_clone = connection_id.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval);
        let mut counter = 0u64;
        
        loop {
            interval_timer.tick().await;
            counter += 1;
            
            let message = SseMessage {
                id: format!("data-{}", counter),
                event_type: "data-update".to_string(),
                timestamp: chrono::Utc::now(),
                data: json!({
                    "counter": counter,
                    "random_value": fastrand::f64(),
                    "sine_wave": (counter as f64 * 0.1).sin(),
                    "fibonacci": calculate_fibonacci_nth(counter % 20),
                    "timestamp": chrono::Utc::now(),
                }),
            };
            
            SSE_MANAGER.0.send_event(message);
            debug!("Sent data update #{} for connection {}", counter, connection_id_clone);
        }
    });
}

/// Start log streaming
async fn start_log_stream(connection_id: String) {
    let connection_id_clone = connection_id.clone();
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        let mut counter = 0u64;
        
        let log_levels = ["INFO", "WARN", "ERROR", "DEBUG"];
        let components = ["server", "websocket", "sse", "events", "services"];
        
        loop {
            interval.tick().await;
            counter += 1;
            
            let level = log_levels[counter as usize % log_levels.len()];
            let component = components[counter as usize % components.len()];
            
            let message = SseMessage {
                id: format!("log-{}", counter),
                event_type: "log-entry".to_string(),
                timestamp: chrono::Utc::now(),
                data: json!({
                    "level": level,
                    "component": component,
                    "message": format!("Sample log message #{} from {}", counter, component),
                    "timestamp": chrono::Utc::now(),
                }),
            };
            
            SSE_MANAGER.0.send_event(message);
            debug!("Sent log entry #{} for connection {}", counter, connection_id_clone);
        }
    });
}

/// Start metrics streaming
async fn start_metrics_stream(connection_id: String, app_state: AppState) {
    let connection_id_clone = connection_id.clone();
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        let mut counter = 0u64;
        
        loop {
            interval.tick().await;
            counter += 1;
            
            let stats = app_state.stats.read().await.clone();
            
            let message = SseMessage {
                id: format!("metrics-{}", counter),
                event_type: "metrics-update".to_string(),
                timestamp: chrono::Utc::now(),
                data: json!({
                    "cpu_usage": fastrand::f64() * 100.0,
                    "memory_usage": 256.0 + fastrand::f64() * 512.0,
                    "request_rate": stats.total_requests as f64 / (counter as f64 + 1.0),
                    "error_rate": if stats.total_requests > 0 {
                        stats.failed_requests as f64 / stats.total_requests as f64 * 100.0
                    } else {
                        0.0
                    },
                    "connections": SSE_MANAGER.0.get_connection_count().await,
                    "timestamp": chrono::Utc::now(),
                }),
            };
            
            SSE_MANAGER.0.send_event(message);
            debug!("Sent metrics update #{} for connection {}", counter, connection_id_clone);
        }
    });
}

/// Send JsonRPC event to SSE streams
#[allow(dead_code)]
pub fn send_jsonrpc_event(method: &str, params: &Value, response: &Value, success: bool) {
    let message = SseMessage {
        id: Uuid::new_v4().to_string(),
        event_type: "jsonrpc-event".to_string(),
        timestamp: chrono::Utc::now(),
        data: json!({
            "method": method,
            "params": params,
            "response": response,
            "success": success,
            "timestamp": chrono::Utc::now(),
        }),
    };
    
    SSE_MANAGER.0.send_event(message);
}

/// Calculate nth Fibonacci number
fn calculate_fibonacci_nth(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    
    let mut a = 0;
    let mut b = 1;
    
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}

/// Get SSE connection info
pub async fn get_sse_info() -> Value {
    json!({
        "active_connections": SSE_MANAGER.0.get_connection_count().await,
        "available_streams": [
            {
                "type": "stats",
                "description": "Real-time system statistics",
                "endpoint": "/api/sse?stream_type=stats&interval_ms=5000"
            },
            {
                "type": "events", 
                "description": "JsonRPC request/response events",
                "endpoint": "/api/sse?stream_type=events"
            },
            {
                "type": "data",
                "description": "Generated data stream",
                "endpoint": "/api/sse?stream_type=data&interval_ms=1000"
            },
            {
                "type": "logs",
                "description": "System log entries",
                "endpoint": "/api/sse?stream_type=logs"
            },
            {
                "type": "metrics",
                "description": "Performance metrics",
                "endpoint": "/api/sse?stream_type=metrics"
            }
        ]
    })
} 