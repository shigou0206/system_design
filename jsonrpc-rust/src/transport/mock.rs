//! Mock transport implementation for testing
//! 
//! This module provides mock implementations of transport and connection
//! traits for unit testing, integration testing, and fuzzing.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::sleep;
use uuid::Uuid;

use crate::core::error::{Error, Result};
use crate::core::traits::{Transport, Connection};
use super::abstraction::{
    TransportLayer, ConnectionManager, MessageCodec, TransportConfig,
    JsonRpcMessage, TransportStats, ConnectionInfo, ConnectionState,
    TimeoutConfig, RetryConfig, ConnectionLimits, FramingType,
    DefaultMessageCodec,
};

/// Mock transport implementation for testing
pub struct MockTransport {
    /// Transport configuration
    config: MockConfig,
    /// Connection manager
    connection_manager: Arc<Mutex<MockConnectionManager>>,
    /// Message queue for sending
    send_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,
    /// Message queue for receiving
    receive_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,
    /// Error injection settings
    error_injection: Arc<RwLock<ErrorInjection>>,
    /// Transport statistics
    stats: Arc<RwLock<TransportStats>>,
    /// Mock behaviors
    behaviors: Arc<RwLock<MockBehaviors>>,
}

/// Mock transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Timeout configuration
    pub timeouts: TimeoutConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Connection limits
    pub connection_limits: ConnectionLimits,
    /// Simulated network latency
    pub network_latency: Duration,
    /// Message loss probability (0.0 to 1.0)
    pub message_loss_rate: f64,
    /// Enable deterministic behavior for tests
    pub deterministic: bool,
    /// Maximum queue size
    pub max_queue_size: usize,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            timeouts: TimeoutConfig::default(),
            retry_config: RetryConfig::default(),
            connection_limits: ConnectionLimits::default(),
            network_latency: Duration::from_millis(10),
            message_loss_rate: 0.0,
            deterministic: true,
            max_queue_size: 1000,
        }
    }
}

impl TransportConfig for MockConfig {
    fn validate(&self) -> Result<()> {
        if self.message_loss_rate < 0.0 || self.message_loss_rate > 1.0 {
            return Err(Error::Configuration {
                message: "Message loss rate must be between 0.0 and 1.0".to_string(),
                source: None,
            });
        }
        
        if self.max_queue_size == 0 {
            return Err(Error::Configuration {
                message: "Max queue size cannot be zero".to_string(),
                source: None,
            });
        }
        
        Ok(())
    }
    
    fn timeouts(&self) -> TimeoutConfig {
        self.timeouts.clone()
    }
    
    fn retry_config(&self) -> RetryConfig {
        self.retry_config.clone()
    }
    
    fn connection_limits(&self) -> ConnectionLimits {
        self.connection_limits.clone()
    }
}

/// Mock connection implementation
pub struct MockConnection {
    /// Connection ID
    id: String,
    /// Connection state
    state: ConnectionState,
    /// Connection info
    info: ConnectionInfo,
    /// Send channel
    send_tx: Option<mpsc::UnboundedSender<JsonRpcMessage>>,
    /// Receive channel
    receive_rx: Option<mpsc::UnboundedReceiver<JsonRpcMessage>>,
    /// Error injection
    error_injection: Arc<RwLock<ErrorInjection>>,
    /// Last error
    last_error: Option<Error>,
    /// Connection behaviors
    behaviors: MockConnectionBehaviors,
}

/// Mock connection behaviors for testing
#[derive(Debug, Clone)]
pub struct MockConnectionBehaviors {
    /// Delay before connection establishment
    pub connect_delay: Duration,
    /// Delay before disconnection
    pub disconnect_delay: Duration,
    /// Fail connection attempts
    pub fail_connect: bool,
    /// Fail ping attempts
    pub fail_ping: bool,
    /// Auto-disconnect after duration
    pub auto_disconnect_after: Option<Duration>,
}

impl Default for MockConnectionBehaviors {
    fn default() -> Self {
        Self {
            connect_delay: Duration::from_millis(0),
            disconnect_delay: Duration::from_millis(0),
            fail_connect: false,
            fail_ping: false,
            auto_disconnect_after: None,
        }
    }
}

impl MockConnection {
    /// Create a new mock connection
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: id.clone(),
            state: ConnectionState::Disconnected,
            info: ConnectionInfo {
                id,
                remote_addr: None,
                local_addr: None,
                state: ConnectionState::Disconnected,
                connected_at: now,
                last_activity: now,
                messages_sent: 0,
                messages_received: 0,
            },
            send_tx: None,
            receive_rx: None,
            error_injection: Arc::new(RwLock::new(ErrorInjection::default())),
            last_error: None,
            behaviors: MockConnectionBehaviors::default(),
        }
    }
    
    /// Create a new mock connection with behaviors
    pub fn with_behaviors(id: String, behaviors: MockConnectionBehaviors) -> Self {
        let mut connection = Self::new(id);
        connection.behaviors = behaviors;
        connection
    }
    
    /// Set error injection for this connection
    pub async fn set_error_injection(&mut self, injection: ErrorInjection) {
        *self.error_injection.write().await = injection;
    }
    
    /// Simulate receiving a message
    pub async fn simulate_receive(&mut self, message: JsonRpcMessage) -> Result<()> {
        if let Some(ref tx) = self.send_tx {
            tx.send(message)
                .map_err(|_| Error::Transport {
                    message: "Failed to send message to receive queue".to_string(),
                    source: None,
                })?;
            
            self.info.messages_received += 1;
            self.info.last_activity = chrono::Utc::now();
        }
        Ok(())
    }
    
    /// Check if connection should fail based on error injection
    async fn should_inject_error(&self, operation: &str) -> Option<Error> {
        let injection = self.error_injection.read().await;
        injection.should_fail(operation)
    }
}

#[async_trait]
impl Connection for MockConnection {
    async fn connect(&mut self) -> Result<()> {
        if self.behaviors.fail_connect {
            return Err(Error::Transport {
                message: "Mock connection configured to fail".to_string(),
                source: None,
            });
        }
        
        if let Some(error) = self.should_inject_error("connect").await {
            return Err(error);
        }
        
        self.state = ConnectionState::Connecting;
        self.info.state = ConnectionState::Connecting;
        
        // Simulate connection delay
        if !self.behaviors.connect_delay.is_zero() {
            sleep(self.behaviors.connect_delay).await;
        }
        
        // Create channels for message passing
        let (send_tx, mut send_rx) = mpsc::unbounded_channel::<()>();
        let (receive_tx, receive_rx) = mpsc::unbounded_channel();
        
        self.send_tx = Some(receive_tx);
        self.receive_rx = Some(receive_rx);
        
        self.state = ConnectionState::Connected;
        self.info.state = ConnectionState::Connected;
        self.info.connected_at = chrono::Utc::now();
        self.info.last_activity = chrono::Utc::now();
        
        // Setup auto-disconnect if configured
        if let Some(duration) = self.behaviors.auto_disconnect_after {
            let id = self.id.clone();
            tokio::spawn(async move {
                sleep(duration).await;
                tracing::debug!("Auto-disconnecting mock connection {}", id);
            });
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if let Some(error) = self.should_inject_error("disconnect").await {
            return Err(error);
        }
        
        self.state = ConnectionState::Disconnecting;
        self.info.state = ConnectionState::Disconnecting;
        
        // Simulate disconnection delay
        if !self.behaviors.disconnect_delay.is_zero() {
            sleep(self.behaviors.disconnect_delay).await;
        }
        
        // Close channels
        self.send_tx.take();
        self.receive_rx.take();
        
        self.state = ConnectionState::Disconnected;
        self.info.state = ConnectionState::Disconnected;
        
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }
    
    fn is_closed(&self) -> bool {
        matches!(self.state, ConnectionState::Disconnected)
    }
    
    fn connection_info(&self) -> HashMap<String, serde_json::Value> {
        let mut info = HashMap::new();
        info.insert("id".to_string(), self.id.clone().into());
        info.insert("protocol".to_string(), "mock".into());
        info.insert("state".to_string(), format!("{:?}", self.state).into());
        info.insert("messages_sent".to_string(), self.info.messages_sent.into());
        info.insert("messages_received".to_string(), self.info.messages_received.into());
        info.insert("mock_connection".to_string(), true.into());
        info
    }
    
    fn last_error(&self) -> Option<&Error> {
        self.last_error.as_ref()
    }
    
    async fn ping(&mut self) -> Result<()> {
        if self.behaviors.fail_ping {
            return Err(Error::Transport {
                message: "Mock ping configured to fail".to_string(),
                source: None,
            });
        }
        
        if let Some(error) = self.should_inject_error("ping").await {
            return Err(error);
        }
        
        // Simulate ping latency
        sleep(Duration::from_millis(1)).await;
        
        self.info.last_activity = chrono::Utc::now();
        Ok(())
    }
}

/// Error injection configuration for testing
#[derive(Debug, Clone)]
pub struct ErrorInjection {
    /// Operations that should fail
    pub failing_operations: HashMap<String, ErrorSpec>,
    /// Global failure rate (0.0 to 1.0)
    pub global_failure_rate: f64,
    /// Random seed for deterministic testing
    pub random_seed: Option<u64>,
}

/// Error specification for injection
#[derive(Debug, Clone)]
pub struct ErrorSpec {
    /// Error message to inject
    pub error_message: String,
    /// Failure probability (0.0 to 1.0)
    pub failure_rate: f64,
    /// Delay before failing
    pub delay: Duration,
    /// Number of times to fail before succeeding
    pub fail_count: Option<usize>,
}

impl Default for ErrorInjection {
    fn default() -> Self {
        Self {
            failing_operations: HashMap::new(),
            global_failure_rate: 0.0,
            random_seed: None,
        }
    }
}

impl ErrorInjection {
    /// Check if an operation should fail
    pub fn should_fail(&self, operation: &str) -> Option<Error> {
        if let Some(spec) = self.failing_operations.get(operation) {
            // For simplicity, always fail if configured
            // In a real implementation, you'd use the failure_rate and random_seed
            if spec.failure_rate > 0.0 {
                return Some(Error::Transport {
                    message: format!("Injected error for {}: {}", operation, spec.error_message),
                    source: None,
                });
            }
        }
        
        // Check global failure rate
        if self.global_failure_rate > 0.0 {
            // Simplified: always fail if global rate > 0
            return Some(Error::Transport {
                message: format!("Global error injection for {}", operation),
                source: None,
            });
        }
        
        None
    }
    
    /// Add error injection for a specific operation
    pub fn inject_error(&mut self, operation: String, spec: ErrorSpec) {
        self.failing_operations.insert(operation, spec);
    }
    
    /// Remove error injection for an operation
    pub fn remove_error(&mut self, operation: &str) {
        self.failing_operations.remove(operation);
    }
    
    /// Clear all error injections
    pub fn clear(&mut self) {
        self.failing_operations.clear();
        self.global_failure_rate = 0.0;
    }
}

/// Mock behaviors for transport-level operations
#[derive(Debug, Clone)]
pub struct MockBehaviors {
    /// Simulate message reordering
    pub reorder_messages: bool,
    /// Simulate message duplication
    pub duplicate_messages: bool,
    /// Simulate network partitions
    pub network_partition: bool,
    /// Custom latency per message type
    pub message_latencies: HashMap<String, Duration>,
}

impl Default for MockBehaviors {
    fn default() -> Self {
        Self {
            reorder_messages: false,
            duplicate_messages: false,
            network_partition: false,
            message_latencies: HashMap::new(),
        }
    }
}

/// Mock connection manager
pub struct MockConnectionManager {
    /// Active connections
    connections: HashMap<String, Arc<RwLock<MockConnection>>>,
    /// Configuration
    config: MockConfig,
}

impl MockConnectionManager {
    pub fn new(config: MockConfig) -> Self {
        Self {
            connections: HashMap::new(),
            config,
        }
    }
}

#[async_trait]
impl ConnectionManager for MockConnectionManager {
    type Connection = MockConnection;
    
    async fn add_connection(&mut self, id: String, connection: Self::Connection) -> Result<()> {
        if self.connections.len() >= self.config.connection_limits.max_connections {
            return Err(Error::Transport {
                message: "Connection limit exceeded".to_string(),
                source: None,
            });
        }
        
        self.connections.insert(id, Arc::new(RwLock::new(connection)));
        Ok(())
    }
    
    async fn remove_connection(&mut self, id: &str) -> Result<Option<Self::Connection>> {
        if let Some(conn_arc) = self.connections.remove(id) {
            match Arc::try_unwrap(conn_arc) {
                Ok(rwlock) => Ok(Some(rwlock.into_inner())),
                Err(_) => Err(Error::Transport {
                    message: "Connection is still in use".to_string(),
                    source: None,
                }),
            }
        } else {
            Ok(None)
        }
    }
    
    async fn get_connection(&self, id: &str) -> Option<Arc<RwLock<Self::Connection>>> {
        self.connections.get(id).cloned()
    }
    
    async fn list_connections(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }
    
    async fn close_all(&mut self) -> Result<()> {
        for (_, connection) in self.connections.iter() {
            let mut conn = connection.write().await;
            let _ = conn.disconnect().await;
        }
        self.connections.clear();
        Ok(())
    }
    
    fn connection_stats(&self) -> HashMap<String, ConnectionInfo> {
        HashMap::new()
    }
}

impl MockTransport {
    /// Create a new mock transport
    pub async fn new(config: MockConfig) -> Result<Self> {
        config.validate()?;
        
        let connection_manager = Arc::new(Mutex::new(MockConnectionManager::new(config.clone())));
        let send_queue = Arc::new(Mutex::new(VecDeque::new()));
        let receive_queue = Arc::new(Mutex::new(VecDeque::new()));
        let error_injection = Arc::new(RwLock::new(ErrorInjection::default()));
        let stats = Arc::new(RwLock::new(TransportStats::default()));
        let behaviors = Arc::new(RwLock::new(MockBehaviors::default()));
        
        Ok(Self {
            config,
            connection_manager,
            send_queue,
            receive_queue,
            error_injection,
            stats,
            behaviors,
        })
    }
    
    /// Create a simple mock transport with default configuration
    pub async fn simple() -> Result<Self> {
        Self::new(MockConfig::default()).await
    }
    
    /// Set error injection for the transport
    pub async fn set_error_injection(&self, injection: ErrorInjection) {
        *self.error_injection.write().await = injection;
    }
    
    /// Set behaviors for the transport
    pub async fn set_behaviors(&self, behaviors: MockBehaviors) {
        *self.behaviors.write().await = behaviors;
    }
    
    /// Add a message to the receive queue (simulates receiving from network)
    pub async fn queue_message(&self, message: JsonRpcMessage) -> Result<()> {
        let mut queue = self.receive_queue.lock().await;
        if queue.len() >= self.config.max_queue_size {
            return Err(Error::Transport {
                message: "Receive queue is full".to_string(),
                source: None,
            });
        }
        queue.push_back(message);
        Ok(())
    }
    
    /// Get all messages from the send queue
    pub async fn drain_sent_messages(&self) -> Vec<JsonRpcMessage> {
        let mut queue = self.send_queue.lock().await;
        queue.drain(..).collect()
    }
    
    /// Get the number of queued sent messages
    pub async fn sent_message_count(&self) -> usize {
        self.send_queue.lock().await.len()
    }
    
    /// Create a mock connection
    pub async fn create_connection(&self, id: Option<String>) -> Result<String> {
        let connection_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let connection = MockConnection::new(connection_id.clone());
        
        let mut manager = self.connection_manager.lock().await;
        manager.add_connection(connection_id.clone(), connection).await?;
        
        Ok(connection_id)
    }
}

#[async_trait]
impl TransportLayer for MockTransport {
    type Connection = MockConnection;
    type Config = MockConfig;
    
    async fn new(config: Self::Config) -> Result<Self> {
        Self::new(config).await
    }
    
    async fn get_connection(&mut self, address: &str) -> Result<Arc<RwLock<Self::Connection>>> {
        // For mock transport, we create a connection if it doesn't exist
        let manager = self.connection_manager.lock().await;
        
        // Check if we already have a connection for this address
        for (id, conn) in manager.connections.iter() {
            if id.contains(address) {
                return Ok(conn.clone());
            }
        }
        drop(manager);
        
        // Create new connection
        let connection_id = format!("mock-{}-{}", address, Uuid::new_v4());
        self.create_connection(Some(connection_id.clone())).await?;
        
        let manager = self.connection_manager.lock().await;
        manager.get_connection(&connection_id).await
            .ok_or_else(|| Error::Transport {
                message: "Failed to retrieve created connection".to_string(),
                source: None,
            })
    }
    
    async fn send_message(&mut self, message: JsonRpcMessage, _address: &str) -> Result<()> {
        // Simulate network latency
        if !self.config.network_latency.is_zero() {
            sleep(self.config.network_latency).await;
        }
        
        // Check for message loss
        if self.config.message_loss_rate > 0.0 && !self.config.deterministic {
            // Simplified: skip message loss for deterministic testing
        }
        
        // Add to send queue
        let mut queue = self.send_queue.lock().await;
        if queue.len() >= self.config.max_queue_size {
            return Err(Error::Transport {
                message: "Send queue is full".to_string(),
                source: None,
            });
        }
        queue.push_back(message);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.messages_sent += 1;
        
        Ok(())
    }
    
    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        // Simulate network latency
        if !self.config.network_latency.is_zero() {
            sleep(self.config.network_latency).await;
        }
        
        let mut queue = self.receive_queue.lock().await;
        let message = queue.pop_front()
            .ok_or_else(|| Error::Transport {
                message: "No messages available".to_string(),
                source: None,
            })?;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.messages_received += 1;
        
        Ok(message)
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        let mut manager = self.connection_manager.lock().await;
        manager.close_all().await?;
        
        // Clear queues
        self.send_queue.lock().await.clear();
        self.receive_queue.lock().await.clear();
        
        Ok(())
    }
    
    fn stats(&self) -> TransportStats {
        // In a real implementation, this would be async
        TransportStats::default()
    }
    
    fn connection_count(&self) -> usize {
        // In a real implementation, this would be async
        0
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send(&mut self, message: &str) -> Result<()> {
        let json_message = JsonRpcMessage::from_json(message)?;
        self.send_message(json_message, "mock://default").await
    }
    
    async fn receive(&mut self) -> Result<String> {
        let message = self.receive_message().await?;
        message.to_json()
    }
    
    async fn close(&mut self) -> Result<()> {
        self.shutdown().await
    }
    
    fn is_bidirectional(&self) -> bool {
        true
    }
    
    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        metadata.insert("protocol".to_string(), "mock".into());
        metadata.insert("deterministic".to_string(), self.config.deterministic.into());
        metadata.insert("network_latency_ms".to_string(), (self.config.network_latency.as_millis() as u64).into());
        metadata.insert("message_loss_rate".to_string(), self.config.message_loss_rate.into());
        metadata.insert("max_queue_size".to_string(), self.config.max_queue_size.into());
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_mock_config() {
        let config = MockConfig::default();
        assert!(config.validate().is_ok());
        
        let mut invalid_config = config;
        invalid_config.message_loss_rate = 1.5; // Invalid rate
        assert!(invalid_config.validate().is_err());
    }
    
    #[tokio::test]
    async fn test_mock_connection() {
        let mut connection = MockConnection::new("test-conn".to_string());
        assert!(!connection.is_connected());
        assert!(connection.is_closed());
        
        connection.connect().await.unwrap();
        assert!(connection.is_connected());
        assert!(!connection.is_closed());
        
        connection.disconnect().await.unwrap();
        assert!(!connection.is_connected());
        assert!(connection.is_closed());
    }
    
    #[tokio::test]
    async fn test_mock_transport() {
        let mut transport = MockTransport::simple().await.unwrap();
        
        // Test sending a message
        let request = JsonRpcMessage::request("test_method", Some(json!({"param": "value"})));
        transport.send_message(request.clone(), "mock://test").await.unwrap();
        
        // Check that message was added to send queue
        let sent_messages = transport.drain_sent_messages().await;
        assert_eq!(sent_messages.len(), 1);
        assert_eq!(sent_messages[0], request);
        
        // Test receiving a message
        let response = JsonRpcMessage::response(json!(1), json!({"result": "success"}));
        transport.queue_message(response.clone()).await.unwrap();
        
        let received = transport.receive_message().await.unwrap();
        assert_eq!(received, response);
    }
    
    #[tokio::test]
    async fn test_error_injection() {
        let mut injection = ErrorInjection::default();
        injection.inject_error("connect".to_string(), ErrorSpec {
            error_message: "Test error".to_string(),
            failure_rate: 1.0,
            delay: Duration::from_millis(0),
            fail_count: None,
        });
        
        let error = injection.should_fail("connect");
        assert!(error.is_some());
        assert!(error.unwrap().to_string().contains("Test error"));
        
        let no_error = injection.should_fail("other_operation");
        assert!(no_error.is_none());
    }
    
    #[tokio::test]
    async fn test_mock_connection_behaviors() {
        let behaviors = MockConnectionBehaviors {
            fail_connect: true,
            ..Default::default()
        };
        
        let mut connection = MockConnection::with_behaviors("test".to_string(), behaviors);
        let result = connection.connect().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("configured to fail"));
    }
    
    #[tokio::test]
    async fn test_connection_manager() {
        let config = MockConfig::default();
        let mut manager = MockConnectionManager::new(config);
        
        let connection = MockConnection::new("test".to_string());
        assert!(manager.add_connection("test".to_string(), connection).await.is_ok());
        
        let connections = manager.list_connections().await;
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0], "test");
        
        let removed = manager.remove_connection("test").await.unwrap();
        assert!(removed.is_some());
        
        let connections = manager.list_connections().await;
        assert_eq!(connections.len(), 0);
    }
} 