//! TCP transport implementation for JSON-RPC
//! 
//! This module provides a TCP-based transport for JSON-RPC communication
//! with support for connection pooling, message framing, and automatic
//! reconnection.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio_util::codec::{Framed, LengthDelimitedCodec, LinesCodec};
use uuid::Uuid;

use crate::core::error::{Error, Result};
use crate::core::traits::{Transport, Connection};
use super::abstraction::{
    TransportLayer, ConnectionManager, MessageCodec, TransportConfig,
    JsonRpcMessage, TransportStats, ConnectionInfo, ConnectionState,
    TimeoutConfig, RetryConfig, ConnectionLimits, FramingType,
    DefaultMessageCodec,
};

/// TCP transport implementation
pub struct TcpTransport {
    /// Transport configuration
    config: TcpConfig,
    /// Connection manager for pooling
    connection_manager: Arc<Mutex<TcpConnectionManager>>,
    /// Message codec for framing
    codec: Box<dyn MessageCodec>,
    /// Transport statistics
    stats: Arc<RwLock<TransportStats>>,
    /// Active connections
    connections: Arc<RwLock<HashMap<String, Arc<RwLock<TcpConnection>>>>>,
}

/// TCP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    /// Bind address for server mode
    pub bind_address: Option<SocketAddr>,
    /// Default server address for client connections
    pub server_address: Option<SocketAddr>,
    /// Timeout configuration
    pub timeouts: TimeoutConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Connection limits
    pub connection_limits: ConnectionLimits,
    /// Message framing type
    pub framing: FramingType,
    /// Enable TCP_NODELAY
    pub no_delay: bool,
    /// Keep-alive settings
    pub keep_alive: Option<Duration>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            bind_address: None,
            server_address: None,
            timeouts: TimeoutConfig::default(),
            retry_config: RetryConfig::default(),
            connection_limits: ConnectionLimits::default(),
            framing: FramingType::LengthPrefixed,
            no_delay: true,
            keep_alive: Some(Duration::from_secs(60)),
        }
    }
}

impl TransportConfig for TcpConfig {
    fn validate(&self) -> Result<()> {
        if self.timeouts.connect_timeout.is_zero() {
            return Err(Error::Configuration {
                message: "Connect timeout cannot be zero".to_string(),
                source: None,
            });
        }
        
        if self.connection_limits.max_connections == 0 {
            return Err(Error::Configuration {
                message: "Max connections cannot be zero".to_string(),
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

/// TCP connection implementation
pub struct TcpConnection {
    /// Connection ID
    id: String,
    /// TCP stream
    stream: Option<TcpStream>,
    /// Remote address
    remote_addr: Option<SocketAddr>,
    /// Local address
    local_addr: Option<SocketAddr>,
    /// Connection state
    state: ConnectionState,
    /// Connection metadata
    info: ConnectionInfo,
    /// Last error
    last_error: Option<Error>,
}

impl TcpConnection {
    /// Create a new TCP connection
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: id.clone(),
            stream: None,
            remote_addr: None,
            local_addr: None,
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
            last_error: None,
        }
    }
    
    /// Create from existing TCP stream
    pub async fn from_stream(stream: TcpStream) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let remote_addr = stream.peer_addr().ok();
        let local_addr = stream.local_addr().ok();
        let now = chrono::Utc::now();
        
        let mut connection = Self::new(id.clone());
        connection.stream = Some(stream);
        connection.remote_addr = remote_addr;
        connection.local_addr = local_addr;
        connection.state = ConnectionState::Connected;
        connection.info = ConnectionInfo {
            id,
            remote_addr,
            local_addr,
            state: ConnectionState::Connected,
            connected_at: now,
            last_activity: now,
            messages_sent: 0,
            messages_received: 0,
        };
        
        Ok(connection)
    }
    
    /// Connect to a remote address
    pub async fn connect_to(&mut self, addr: SocketAddr, config: &TcpConfig) -> Result<()> {
        self.state = ConnectionState::Connecting;
        
        let stream = tokio::time::timeout(
            config.timeouts.connect_timeout,
            TcpStream::connect(addr)
        ).await
        .map_err(|_| Error::Transport {
            message: format!("Connection timeout to {}", addr),
            source: None,
        })?
        .map_err(|e| Error::Transport {
            message: format!("Failed to connect to {}: {}", addr, e),
            source: Some(Box::new(e)),
        })?;
        
        // Configure socket options
        if config.no_delay {
            if let Err(e) = stream.set_nodelay(true) {
                tracing::warn!("Failed to set TCP_NODELAY: {}", e);
            }
        }
        
        self.remote_addr = stream.peer_addr().ok();
        self.local_addr = stream.local_addr().ok();
        self.stream = Some(stream);
        self.state = ConnectionState::Connected;
        self.info.state = ConnectionState::Connected;
        self.info.remote_addr = self.remote_addr;
        self.info.local_addr = self.local_addr;
        self.info.connected_at = chrono::Utc::now();
        self.info.last_activity = chrono::Utc::now();
        
        Ok(())
    }
    
    /// Send raw data through the connection
    pub async fn send_data(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.write_all(data).await
                .map_err(|e| Error::Transport {
                    message: format!("Failed to send data: {}", e),
                    source: Some(Box::new(e)),
                })?;
            
            self.info.messages_sent += 1;
            self.info.last_activity = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::Transport {
                message: "Connection not established".to_string(),
                source: None,
            })
        }
    }
    
    /// Receive raw data from the connection
    pub async fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(ref mut stream) = self.stream {
            use tokio::io::AsyncReadExt;
            
            let bytes_read = stream.read(buffer).await
                .map_err(|e| Error::Transport {
                    message: format!("Failed to receive data: {}", e),
                    source: Some(Box::new(e)),
                })?;
            
            if bytes_read > 0 {
                self.info.messages_received += 1;
                self.info.last_activity = chrono::Utc::now();
            }
            
            Ok(bytes_read)
        } else {
            Err(Error::Transport {
                message: "Connection not established".to_string(),
                source: None,
            })
        }
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn connect(&mut self) -> Result<()> {
        // Connection is handled by connect_to method
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.state = ConnectionState::Disconnecting;
        self.info.state = ConnectionState::Disconnecting;
        
        if let Some(stream) = self.stream.take() {
            drop(stream); // Close the stream
        }
        
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
        info.insert("protocol".to_string(), "tcp".into());
        info.insert("state".to_string(), format!("{:?}", self.state).into());
        
        if let Some(addr) = self.remote_addr {
            info.insert("remote_addr".to_string(), addr.to_string().into());
        }
        
        if let Some(addr) = self.local_addr {
            info.insert("local_addr".to_string(), addr.to_string().into());
        }
        
        info.insert("messages_sent".to_string(), self.info.messages_sent.into());
        info.insert("messages_received".to_string(), self.info.messages_received.into());
        
        info
    }
    
    fn last_error(&self) -> Option<&Error> {
        self.last_error.as_ref()
    }
    
    async fn ping(&mut self) -> Result<()> {
        // Send a simple ping message
        let ping_data = b"PING\n";
        self.send_data(ping_data).await
    }
}

/// TCP connection manager
pub struct TcpConnectionManager {
    /// Active connections
    connections: HashMap<String, Arc<RwLock<TcpConnection>>>,
    /// Configuration
    config: TcpConfig,
}

impl TcpConnectionManager {
    pub fn new(config: TcpConfig) -> Self {
        Self {
            connections: HashMap::new(),
            config,
        }
    }
}

#[async_trait]
impl ConnectionManager for TcpConnectionManager {
    type Connection = TcpConnection;
    
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
            // Try to extract the connection from Arc<RwLock<>>
            match Arc::try_unwrap(conn_arc) {
                Ok(rwlock) => Ok(Some(rwlock.into_inner())),
                Err(_) => {
                    // Connection is still referenced elsewhere
                    Err(Error::Transport {
                        message: "Connection is still in use".to_string(),
                        source: None,
                    })
                }
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
        // This would need async access to get the actual stats
        // For now, return empty map
        HashMap::new()
    }
}

impl TcpTransport {
    /// Create a new TCP transport with configuration
    pub async fn new(config: TcpConfig) -> Result<Self> {
        config.validate()?;
        
        let connection_manager = Arc::new(Mutex::new(TcpConnectionManager::new(config.clone())));
        let codec: Box<dyn MessageCodec> = Box::new(DefaultMessageCodec::new(config.framing.clone()));
        let stats = Arc::new(RwLock::new(TransportStats::default()));
        let connections = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            config,
            connection_manager,
            codec,
            stats,
            connections,
        })
    }
    
    /// Create a client TCP transport for connecting to a server
    pub async fn client(server_addr: SocketAddr) -> Result<Self> {
        let mut config = TcpConfig::default();
        config.server_address = Some(server_addr);
        Self::new(config).await
    }
    
    /// Create a server TCP transport for accepting connections
    pub async fn server(bind_addr: SocketAddr) -> Result<Self> {
        let mut config = TcpConfig::default();
        config.bind_address = Some(bind_addr);
        Self::new(config).await
    }
    
    /// Start listening for incoming connections (server mode)
    pub async fn listen(&self) -> Result<TcpListener> {
        if let Some(bind_addr) = self.config.bind_address {
            let listener = TcpListener::bind(bind_addr).await
                .map_err(|e| Error::Transport {
                    message: format!("Failed to bind to {}: {}", bind_addr, e),
                    source: Some(Box::new(e)),
                })?;
            
            tracing::info!("TCP transport listening on {}", bind_addr);
            Ok(listener)
        } else {
            Err(Error::Configuration {
                message: "No bind address configured for server mode".to_string(),
                source: None,
            })
        }
    }
    
    /// Accept an incoming connection
    pub async fn accept(&self, listener: &TcpListener) -> Result<String> {
        let (stream, addr) = listener.accept().await
            .map_err(|e| Error::Transport {
                message: format!("Failed to accept connection: {}", e),
                source: Some(Box::new(e)),
            })?;
        
        let connection = TcpConnection::from_stream(stream).await?;
        let connection_id = connection.id.clone();
        
        tracing::debug!("Accepted connection {} from {}", connection_id, addr);
        
        // Add to connection manager
        let mut manager = self.connection_manager.lock().await;
        manager.add_connection(connection_id.clone(), connection).await?;
        
        // Add to active connections
        if let Some(conn_arc) = manager.get_connection(&connection_id).await {
            self.connections.write().await.insert(connection_id.clone(), conn_arc);
        }
        
        Ok(connection_id)
    }
    
    /// Connect to a remote server (client mode)
    pub async fn connect(&self, addr: SocketAddr) -> Result<String> {
        let mut connection = TcpConnection::new(Uuid::new_v4().to_string());
        connection.connect_to(addr, &self.config).await?;
        
        let connection_id = connection.id.clone();
        
        tracing::debug!("Connected to {} with connection {}", addr, connection_id);
        
        // Add to connection manager
        let mut manager = self.connection_manager.lock().await;
        manager.add_connection(connection_id.clone(), connection).await?;
        
        // Add to active connections
        if let Some(conn_arc) = manager.get_connection(&connection_id).await {
            self.connections.write().await.insert(connection_id.clone(), conn_arc);
        }
        
        Ok(connection_id)
    }
}

#[async_trait]
impl TransportLayer for TcpTransport {
    type Connection = TcpConnection;
    type Config = TcpConfig;
    
    async fn new(config: Self::Config) -> Result<Self> {
        Self::new(config).await
    }
    
    async fn get_connection(&mut self, address: &str) -> Result<Arc<RwLock<Self::Connection>>> {
        // Parse address and get or create connection
        let addr: SocketAddr = address.parse()
            .map_err(|e| Error::Configuration {
                message: format!("Invalid address {}: {}", address, e),
                source: Some(Box::new(e)),
            })?;
        
        // Check if we already have a connection to this address
        let connections = self.connections.read().await;
        for (_, conn_arc) in connections.iter() {
            let conn = conn_arc.read().await;
            if conn.remote_addr == Some(addr) && conn.is_connected() {
                return Ok(conn_arc.clone());
            }
        }
        drop(connections);
        
        // Create new connection
        let connection_id = self.connect(addr).await?;
        let connections = self.connections.read().await;
        connections.get(&connection_id)
            .cloned()
            .ok_or_else(|| Error::Transport {
                message: "Failed to retrieve created connection".to_string(),
                source: None,
            })
    }
    
    async fn send_message(&mut self, message: JsonRpcMessage, address: &str) -> Result<()> {
        let connection = self.get_connection(address).await?;
        let encoded = self.codec.encode(&message)?;
        
        let mut conn = connection.write().await;
        conn.send_data(&encoded).await?;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.messages_sent += 1;
        stats.bytes_sent += encoded.len() as u64;
        
        Ok(())
    }
    
    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        // This is a simplified implementation
        // In a real implementation, you'd want to handle multiple connections
        // and use a proper message framing protocol
        
        let connections = self.connections.read().await;
        if let Some((_, connection)) = connections.iter().next() {
            let mut conn = connection.write().await;
            let mut buffer = vec![0; self.config.connection_limits.max_message_size];
            let bytes_read = conn.receive_data(&mut buffer).await?;
            buffer.truncate(bytes_read);
            
            let message = self.codec.decode(&buffer)?;
            
            // Update stats
            let mut stats = self.stats.write().await;
            stats.messages_received += 1;
            stats.bytes_received += bytes_read as u64;
            
            Ok(message)
        } else {
            Err(Error::Transport {
                message: "No active connections".to_string(),
                source: None,
            })
        }
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        let mut manager = self.connection_manager.lock().await;
        manager.close_all().await?;
        self.connections.write().await.clear();
        Ok(())
    }
    
    fn stats(&self) -> TransportStats {
        // This would need async access in a real implementation
        TransportStats::default()
    }
    
    fn connection_count(&self) -> usize {
        // This would need async access in a real implementation
        0
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn send(&mut self, message: &str) -> Result<()> {
        if let Some(addr) = self.config.server_address {
            let json_message = JsonRpcMessage::from_json(message)?;
            self.send_message(json_message, &addr.to_string()).await
        } else {
            Err(Error::Configuration {
                message: "No server address configured".to_string(),
                source: None,
            })
        }
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
        metadata.insert("protocol".to_string(), "tcp".into());
        metadata.insert("framing".to_string(), format!("{:?}", self.config.framing).into());
        metadata.insert("no_delay".to_string(), self.config.no_delay.into());
        
        if let Some(addr) = self.config.bind_address {
            metadata.insert("bind_address".to_string(), addr.to_string().into());
        }
        
        if let Some(addr) = self.config.server_address {
            metadata.insert("server_address".to_string(), addr.to_string().into());
        }
        
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_tcp_config() {
        let config = TcpConfig::default();
        assert!(config.validate().is_ok());
        
        let mut invalid_config = config.clone();
        invalid_config.connection_limits.max_connections = 0;
        assert!(invalid_config.validate().is_err());
    }
    
    #[tokio::test]
    async fn test_tcp_connection() {
        let mut connection = TcpConnection::new("test-conn".to_string());
        assert!(!connection.is_connected());
        assert!(connection.is_closed());
        
        let info = connection.connection_info();
        assert_eq!(info.get("protocol").unwrap(), "tcp");
        assert_eq!(info.get("id").unwrap(), "test-conn");
    }
    
    #[tokio::test]
    async fn test_tcp_transport_creation() {
        let config = TcpConfig::default();
        let transport = TcpTransport::new(config).await;
        assert!(transport.is_ok());
    }
    
    #[tokio::test]
    async fn test_connection_manager() {
        let config = TcpConfig::default();
        let mut manager = TcpConnectionManager::new(config);
        
        let connection = TcpConnection::new("test".to_string());
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