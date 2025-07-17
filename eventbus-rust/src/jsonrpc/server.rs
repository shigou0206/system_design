//! JSON-RPC server implementation for EventBus
//! 
//! This module provides the JSON-RPC server that exposes EventBus functionality
//! over the network using the jsonrpc-rust framework.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use serde_json::{json, Value};

use jsonrpc_rust::prelude::*;
use jsonrpc_rust::transport::tcp::TcpTransport;

use crate::core::traits::{EventBus, BusStats};
use crate::core::{EventEnvelope, EventQuery};
use crate::service::EventBusService;
use crate::jsonrpc::methods::*;

/// Subscription information for managing client subscriptions
#[derive(Debug, Clone)]
struct SubscriptionInfo {
    pub subscription_id: String,
    pub topic: String,
    pub client_id: Option<String>,
    pub sender: broadcast::Sender<EventEnvelope>,
}

/// EventBus JSON-RPC server
pub struct EventBusRpcServer {
    /// The underlying EventBus service
    bus_service: Arc<EventBusService>,
    /// Active subscriptions for clients
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionInfo>>>,
    /// Server start time
    start_time: SystemTime,
}

impl EventBusRpcServer {
    /// Create a new EventBus JSON-RPC server
    pub fn new(bus_service: Arc<EventBusService>) -> Self {
        Self {
            bus_service,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            start_time: SystemTime::now(),
        }
    }

    /// Start the JSON-RPC server on the specified address
    pub async fn start(&self, addr: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting EventBus JSON-RPC server on {}", addr);
        
        // 这里会在实际的jsonrpc-rust实现完成后再完善
        // 目前先提供基本的结构
        
        Ok(())
    }

    /// Handle emit method
    pub async fn handle_emit(&self, params: EmitParams) -> std::result::Result<EmitResponse, JsonRpcError> {
        match self.bus_service.emit(params.event).await {
            Ok(_) => Ok(EmitResponse { success: true }),
            Err(e) => Err(JsonRpcError::new(
                JsonRpcErrorCode::ServerError(error_codes::STORAGE_ERROR),
                format!("Failed to emit event: {}", e),
            )),
        }
    }

    /// Handle emit_batch method
    pub async fn handle_emit_batch(&self, params: EmitBatchParams) -> std::result::Result<EmitBatchResponse, JsonRpcError> {
        let count = params.events.len();
        match self.bus_service.emit_batch(params.events).await {
            Ok(_) => Ok(EmitBatchResponse { 
                success: true, 
                processed_count: count 
            }),
            Err(e) => Err(JsonRpcError::new(
                JsonRpcErrorCode::ServerError(error_codes::STORAGE_ERROR),
                format!("Failed to emit batch: {}", e),
            )),
        }
    }

    /// Handle poll method
    pub async fn handle_poll(&self, params: PollParams) -> std::result::Result<PollResponse, JsonRpcError> {
        match self.bus_service.poll(params.query).await {
            Ok(events) => {
                let total_count = events.len();
                Ok(PollResponse { events, total_count })
            },
            Err(e) => Err(JsonRpcError::new(
                JsonRpcErrorCode::ServerError(error_codes::STORAGE_ERROR),
                format!("Failed to poll events: {}", e),
            )),
        }
    }

    /// Handle subscribe method
    pub async fn handle_subscribe(&self, params: SubscribeParams) -> std::result::Result<SubscribeResponse, JsonRpcError> {
        let subscription_id = Uuid::new_v4().to_string();
        let (sender, _receiver) = broadcast::channel(1000);

        let subscription_info = SubscriptionInfo {
            subscription_id: subscription_id.clone(),
            topic: params.topic.clone(),
            client_id: params.client_id,
            sender: sender.clone(),
        };

        // Store subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription_info);
        }

        // Start forwarding events from EventBus subscription to our broadcast channel
        let bus_service = Arc::clone(&self.bus_service);
        let topic = params.topic.clone();
        let sub_id = subscription_id.clone();
        let subscriptions = Arc::clone(&self.subscriptions);
        
        tokio::spawn(async move {
            match bus_service.subscribe(&topic).await {
                Ok(mut stream) => {
                    use futures::StreamExt;
                    while let Some(event) = stream.next().await {
                        // Check if subscription still exists
                        let subscriptions_guard = subscriptions.read().await;
                        if let Some(sub_info) = subscriptions_guard.get(&sub_id) {
                            // Send event to broadcast channel (ignore if no receivers)
                            let _ = sub_info.sender.send(event);
                        } else {
                            // Subscription was removed, stop the task
                            break;
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to create subscription for topic '{}': {}", topic, e);
                }
            }
        });

        Ok(SubscribeResponse {
            subscription_id,
            success: true,
        })
    }

    /// Handle unsubscribe method
    pub async fn handle_unsubscribe(&self, params: UnsubscribeParams) -> std::result::Result<UnsubscribeResponse, JsonRpcError> {
        let mut subscriptions = self.subscriptions.write().await;
        let success = subscriptions.remove(&params.subscription_id).is_some();
        
        Ok(UnsubscribeResponse { success })
    }

    /// Handle list_topics method
    pub async fn handle_list_topics(&self) -> std::result::Result<ListTopicsResponse, JsonRpcError> {
        match self.bus_service.list_topics().await {
            Ok(topics) => Ok(ListTopicsResponse { topics }),
            Err(e) => Err(JsonRpcError::new(
                JsonRpcErrorCode::ServerError(error_codes::SERVICE_UNAVAILABLE),
                format!("Failed to list topics: {}", e),
            )),
        }
    }

    /// Handle get_stats method
    pub async fn handle_get_stats(&self) -> std::result::Result<GetStatsResponse, JsonRpcError> {
        match self.bus_service.get_stats().await {
            Ok(stats) => {
                let uptime_seconds = self.start_time
                    .elapsed()
                    .unwrap_or_default()
                    .as_secs();

                let mut stats_json = BusStatsJson::from(stats);
                stats_json.uptime_seconds = uptime_seconds;

                Ok(GetStatsResponse { stats: stats_json })
            },
            Err(e) => Err(JsonRpcError::new(
                JsonRpcErrorCode::ServerError(error_codes::SERVICE_UNAVAILABLE),
                format!("Failed to get stats: {}", e),
            )),
        }
    }

    /// Handle get_subscription_events method (for polling-based clients)
    pub async fn handle_get_subscription_events(
        &self,
        params: GetSubscriptionEventsParams,
    ) -> std::result::Result<GetSubscriptionEventsResponse, JsonRpcError> {
        let subscriptions = self.subscriptions.read().await;
        
        match subscriptions.get(&params.subscription_id) {
            Some(sub_info) => {
                let mut receiver = sub_info.sender.subscribe();
                let mut events = Vec::new();
                let max_events = params.max_events.unwrap_or(100);
                let timeout_ms = params.timeout_ms.unwrap_or(5000);

                // Try to receive events with timeout
                let timeout = tokio::time::Duration::from_millis(timeout_ms);
                let deadline = tokio::time::Instant::now() + timeout;

                while events.len() < max_events && tokio::time::Instant::now() < deadline {
                    match tokio::time::timeout_at(deadline, receiver.recv()).await {
                        Ok(Ok(event)) => events.push(event),
                        Ok(Err(_)) => break, // Channel closed
                        Err(_) => break, // Timeout
                    }
                }

                Ok(GetSubscriptionEventsResponse {
                    events,
                    has_more: false, // We don't know for sure
                })
            },
            None => Err(JsonRpcError::new(
                JsonRpcErrorCode::ServerError(error_codes::SUBSCRIPTION_NOT_FOUND),
                "Subscription not found".to_string(),
            )),
        }
    }
} 