//! Transport registry for dynamic transport selection and management
//! 
//! This module provides a registry system for managing different transport
//! implementations, allowing dynamic selection and creation of transports
//! based on protocol type or URI scheme.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use crate::core::error::{Error, Result};
use crate::core::traits::{Transport, Connection};
use super::abstraction::{
    TransportLayer, TransportConfig, JsonRpcMessage, ConnectionInfo, 
    TimeoutConfig, RetryConfig, ConnectionLimits,
};
use super::{Protocol, tcp::TcpConfig, mock::MockConfig};

/// Transport type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportType {
    /// TCP transport
    Tcp,
    /// WebSocket transport
    WebSocket,
    /// HTTP transport
    Http,
    /// Mock transport for testing
    Mock,
    /// Custom transport type
    Custom(String),
}

impl From<Protocol> for TransportType {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Tcp => TransportType::Tcp,
            Protocol::WebSocket => TransportType::WebSocket,
            Protocol::Http => TransportType::Http,
            Protocol::Mock => TransportType::Mock,
        }
    }
}

impl std::fmt::Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Tcp => write!(f, "tcp"),
            TransportType::WebSocket => write!(f, "websocket"),
            TransportType::Http => write!(f, "http"),
            TransportType::Mock => write!(f, "mock"),
            TransportType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::str::FromStr for TransportType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(TransportType::Tcp),
            "websocket" | "ws" => Ok(TransportType::WebSocket),
            "http" | "https" => Ok(TransportType::Http),
            "mock" => Ok(TransportType::Mock),
            custom => Ok(TransportType::Custom(custom.to_string())),
        }
    }
}

/// Transport factory trait for creating transport instances
#[async_trait]
pub trait TransportFactory: Send + Sync {
    /// The transport type this factory creates
    type Transport: Transport;
    
    /// The configuration type for this transport
    type Config: TransportConfig;
    
    /// Create a new transport instance with the given configuration
    async fn create(&self, config: Self::Config) -> Result<Self::Transport>;
    
    /// Get the transport type this factory handles
    fn transport_type(&self) -> TransportType;
    
    /// Get default configuration for this transport type
    fn default_config(&self) -> Self::Config;
    
    /// Validate the configuration
    fn validate_config(&self, config: &Self::Config) -> Result<()> {
        config.validate()
    }
    
    /// Parse a URI to extract configuration
    fn parse_uri(&self, uri: &str) -> Result<Self::Config>;
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Default transport type to use if none specified
    pub default_transport: TransportType,
    /// Timeout configuration for transport operations
    pub timeouts: TimeoutConfig,
    /// Retry configuration for failed operations
    pub retry_config: RetryConfig,
    /// Connection limits
    pub connection_limits: ConnectionLimits,
    /// Enable automatic transport selection based on URI scheme
    pub auto_select: bool,
    /// Registry-specific settings
    pub registry_settings: HashMap<String, serde_json::Value>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_transport: TransportType::Tcp,
            timeouts: TimeoutConfig::default(),
            retry_config: RetryConfig::default(),
            connection_limits: ConnectionLimits::default(),
            auto_select: true,
            registry_settings: HashMap::new(),
        }
    }
}

impl TransportConfig for RegistryConfig {
    fn validate(&self) -> Result<()> {
        self.timeouts.connect_timeout.is_zero().then(|| {
            Err(Error::Configuration {
                message: "Connect timeout cannot be zero".to_string(),
                source: None,
            })
        }).unwrap_or(Ok(()))?;
        
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

/// Transport registry for managing multiple transport types
pub struct TransportRegistry {
    /// Registry configuration
    config: RegistryConfig,
    /// Transport instances cache
    transports: Arc<RwLock<HashMap<String, Box<dyn Transport>>>>,
    /// Statistics
    stats: Arc<RwLock<RegistryStats>>,
}

/// Registry statistics
#[derive(Debug, Default, Clone)]
pub struct RegistryStats {
    /// Number of registered transport types
    pub registered_types: usize,
    /// Number of created transport instances
    pub created_instances: u64,
    /// Number of failed creation attempts
    pub creation_failures: u64,
    /// Transport usage counts
    pub usage_counts: HashMap<TransportType, u64>,
}

impl TransportRegistry {
    /// Create a new transport registry
    pub fn new(config: RegistryConfig) -> Result<Self> {
        config.validate()?;
        
        Ok(Self {
            config,
            transports: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RegistryStats::default())),
        })
    }
    
    /// Create a registry with default configuration
    pub fn default() -> Result<Self> {
        Self::new(RegistryConfig::default())
    }
    
    /// Unregister a transport factory
    pub async fn unregister_factory(&self, transport_type: &TransportType) -> Result<bool> {
        let mut factories = self.factories.write().await;
        let removed = factories.remove(transport_type).is_some();
        
        if removed {
            let mut stats = self.stats.write().await;
            stats.registered_types = factories.len();
            stats.usage_counts.remove(transport_type);
        }
        
        Ok(removed)
    }
    
    /// Get list of registered transport types
    pub async fn list_transport_types(&self) -> Vec<TransportType> {
        let factories = self.factories.read().await;
        factories.keys().cloned().collect()
    }
    
    /// Check if a transport type is registered
    pub async fn is_registered(&self, transport_type: &TransportType) -> bool {
        let factories = self.factories.read().await;
        factories.contains_key(transport_type)
    }
    
    /// Create a transport instance by type
    pub async fn create_transport(&self, transport_type: TransportType, config: Option<serde_json::Value>) -> Result<Box<dyn Transport>> {
        let factories = self.factories.read().await;
        let factory = factories.get(&transport_type)
            .ok_or_else(|| Error::Configuration {
                message: format!("Transport type {} not registered", transport_type),
                source: None,
            })?;
        
        // This is a simplified implementation
        // In practice, you'd need to deserialize the config and create the transport
        drop(factories);
        
        let mut stats = self.stats.write().await;
        stats.creation_failures += 1; // Since we're not actually creating
        
        Err(Error::Configuration {
            message: "Transport creation not fully implemented due to type erasure complexity".to_string(),
            source: None,
        })
    }
    
    /// Create a transport from a URI
    pub async fn create_from_uri(&self, uri: &str) -> Result<Box<dyn Transport>> {
        let parsed = Url::parse(uri)
            .map_err(|e| Error::Configuration {
                message: format!("Invalid URI {}: {}", uri, e),
                source: Some(Box::new(e)),
            })?;
        
        let transport_type = self.scheme_to_transport_type(parsed.scheme())?;
        
        if !self.is_registered(&transport_type).await {
            return Err(Error::Configuration {
                message: format!("Transport type {} not registered for scheme {}", transport_type, parsed.scheme()),
                source: None,
            });
        }
        
        // Extract configuration from URI
        let config = self.extract_config_from_uri(&parsed, &transport_type).await?;
        
        self.create_transport(transport_type, Some(config)).await
    }
    
    /// Get or create a cached transport instance
    pub async fn get_or_create_transport(&self, key: &str, transport_type: TransportType, config: Option<serde_json::Value>) -> Result<()> {
        let transports = self.transports.read().await;
        if transports.contains_key(key) {
            return Ok(());
        }
        drop(transports);
        
        // Create new transport
        let transport = self.create_transport(transport_type.clone(), config).await?;
        
        // Cache it
        let mut transports = self.transports.write().await;
        transports.insert(key.to_string(), transport);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.created_instances += 1;
        *stats.usage_counts.entry(transport_type).or_insert(0) += 1;
        
        Ok(())
    }
    
    /// Remove a cached transport instance
    pub async fn remove_transport(&self, key: &str) -> Result<bool> {
        let mut transports = self.transports.write().await;
        Ok(transports.remove(key).is_some())
    }
    
    /// Clear all cached transport instances
    pub async fn clear_transports(&self) -> Result<()> {
        let mut transports = self.transports.write().await;
        transports.clear();
        Ok(())
    }
    
    /// Get registry statistics
    pub async fn stats(&self) -> RegistryStats {
        self.stats.read().await.clone()
    }
    
    /// Map URI scheme to transport type
    fn scheme_to_transport_type(&self, scheme: &str) -> Result<TransportType> {
        match scheme.to_lowercase().as_str() {
            "tcp" => Ok(TransportType::Tcp),
            "ws" | "websocket" => Ok(TransportType::WebSocket),
            "http" | "https" => Ok(TransportType::Http),
            "mock" => Ok(TransportType::Mock),
            unknown => {
                if self.config.auto_select {
                    Ok(TransportType::Custom(unknown.to_string()))
                } else {
                    Err(Error::Configuration {
                        message: format!("Unknown URI scheme: {}", unknown),
                        source: None,
                    })
                }
            }
        }
    }
    
    /// Extract configuration from URI
    async fn extract_config_from_uri(&self, uri: &Url, transport_type: &TransportType) -> Result<serde_json::Value> {
        let mut config = serde_json::Map::new();
        
        // Extract common configuration
        if let Some(host) = uri.host_str() {
            config.insert("host".to_string(), host.into());
        }
        
        if let Some(port) = uri.port() {
            config.insert("port".to_string(), port.into());
        }
        
        config.insert("path".to_string(), uri.path().into());
        
        // Extract query parameters
        for (key, value) in uri.query_pairs() {
            config.insert(key.to_string(), value.to_string().into());
        }
        
        // Transport-specific configuration
        match transport_type {
            TransportType::Tcp => {
                // Add TCP-specific configuration
                config.insert("protocol".to_string(), "tcp".into());
            }
            TransportType::WebSocket => {
                // Add WebSocket-specific configuration
                config.insert("protocol".to_string(), "websocket".into());
            }
            TransportType::Http => {
                // Add HTTP-specific configuration
                config.insert("protocol".to_string(), "http".into());
                config.insert("secure".to_string(), (uri.scheme() == "https").into());
            }
            TransportType::Mock => {
                // Add Mock-specific configuration
                config.insert("protocol".to_string(), "mock".into());
                config.insert("deterministic".to_string(), true.into());
            }
            TransportType::Custom(name) => {
                config.insert("protocol".to_string(), name.clone().into());
            }
        }
        
        Ok(serde_json::Value::Object(config))
    }
}

/// Concrete factory implementations
pub struct TcpTransportFactory;

#[async_trait]
impl TransportFactory for TcpTransportFactory {
    type Transport = crate::transport::tcp::TcpTransport;
    type Config = TcpConfig;
    
    async fn create(&self, config: Self::Config) -> Result<Self::Transport> {
        crate::transport::tcp::TcpTransport::new(config).await
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }
    
    fn default_config(&self) -> Self::Config {
        TcpConfig::default()
    }
    
    fn parse_uri(&self, uri: &str) -> Result<Self::Config> {
        let parsed = Url::parse(uri)
            .map_err(|e| Error::Configuration {
                message: format!("Invalid TCP URI {}: {}", uri, e),
                source: Some(Box::new(e)),
            })?;
        
        let mut config = TcpConfig::default();
        
        if let (Some(host), Some(port)) = (parsed.host_str(), parsed.port()) {
            let addr = format!("{}:{}", host, port);
            config.server_address = Some(addr.parse()
                .map_err(|e| Error::Configuration {
                    message: format!("Invalid address {}: {}", addr, e),
                    source: Some(Box::new(e)),
                })?);
        }
        
        Ok(config)
    }
}

pub struct MockTransportFactory;

#[async_trait]
impl TransportFactory for MockTransportFactory {
    type Transport = crate::transport::mock::MockTransport;
    type Config = MockConfig;
    
    async fn create(&self, config: Self::Config) -> Result<Self::Transport> {
        crate::transport::mock::MockTransport::new(config).await
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Mock
    }
    
    fn default_config(&self) -> Self::Config {
        MockConfig::default()
    }
    
    fn parse_uri(&self, _uri: &str) -> Result<Self::Config> {
        // Mock transport doesn't need URI parsing
        Ok(MockConfig::default())
    }
}

/// Builder for creating a configured transport registry
pub struct RegistryBuilder {
    config: RegistryConfig,
    factories: Vec<Box<dyn Fn() -> Box<dyn std::any::Any + Send + Sync> + Send + Sync>>,
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryBuilder {
    /// Create a new registry builder
    pub fn new() -> Self {
        Self {
            config: RegistryConfig::default(),
            factories: Vec::new(),
        }
    }
    
    /// Set the default transport type
    pub fn default_transport(mut self, transport_type: TransportType) -> Self {
        self.config.default_transport = transport_type;
        self
    }
    
    /// Set timeout configuration
    pub fn timeouts(mut self, timeouts: TimeoutConfig) -> Self {
        self.config.timeouts = timeouts;
        self
    }
    
    /// Set retry configuration
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.config.retry_config = retry_config;
        self
    }
    
    /// Enable or disable auto transport selection
    pub fn auto_select(mut self, auto_select: bool) -> Self {
        self.config.auto_select = auto_select;
        self
    }
    
    /// Add a transport factory (simplified interface)
    pub fn with_tcp(self) -> Self {
        // In a real implementation, this would register the TCP factory
        self
    }
    
    /// Add mock transport factory
    pub fn with_mock(self) -> Self {
        // In a real implementation, this would register the Mock factory
        self
    }
    
    /// Build the registry
    pub async fn build(self) -> Result<TransportRegistry> {
        let registry = TransportRegistry::new(self.config)?;
        
        // Register built-in factories
        registry.register_factory(TcpTransportFactory).await?;
        registry.register_factory(MockTransportFactory).await?;
        
        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_transport_type_conversion() {
        assert_eq!(TransportType::Tcp.to_string(), "tcp");
        assert_eq!("tcp".parse::<TransportType>().unwrap(), TransportType::Tcp);
        assert_eq!("websocket".parse::<TransportType>().unwrap(), TransportType::WebSocket);
        assert_eq!("ws".parse::<TransportType>().unwrap(), TransportType::WebSocket);
    }
    
    #[tokio::test]
    async fn test_registry_creation() {
        let registry = TransportRegistry::default();
        assert!(registry.is_ok());
        
        let invalid_config = RegistryConfig {
            timeouts: TimeoutConfig {
                connect_timeout: Duration::from_secs(0),
                ..Default::default()
            },
            ..Default::default()
        };
        
        let invalid_registry = TransportRegistry::new(invalid_config);
        assert!(invalid_registry.is_err());
    }
    
    #[tokio::test]
    async fn test_factory_registration() {
        let registry = TransportRegistry::default().unwrap();
        
        // Initially no factories registered
        let types = registry.list_transport_types().await;
        assert_eq!(types.len(), 0);
        
        // Register a factory
        registry.register_factory(TcpTransportFactory).await.unwrap();
        
        let types = registry.list_transport_types().await;
        assert_eq!(types.len(), 1);
        assert!(types.contains(&TransportType::Tcp));
        
        assert!(registry.is_registered(&TransportType::Tcp).await);
        assert!(!registry.is_registered(&TransportType::Mock).await);
    }
    
    #[tokio::test]
    async fn test_uri_scheme_mapping() {
        let registry = TransportRegistry::default().unwrap();
        
        assert_eq!(registry.scheme_to_transport_type("tcp").unwrap(), TransportType::Tcp);
        assert_eq!(registry.scheme_to_transport_type("ws").unwrap(), TransportType::WebSocket);
        assert_eq!(registry.scheme_to_transport_type("http").unwrap(), TransportType::Http);
        assert_eq!(registry.scheme_to_transport_type("mock").unwrap(), TransportType::Mock);
    }
    
    #[tokio::test]
    async fn test_registry_builder() {
        let registry = RegistryBuilder::new()
            .default_transport(TransportType::Mock)
            .auto_select(false)
            .with_tcp()
            .with_mock()
            .build()
            .await;
        
        assert!(registry.is_ok());
        let registry = registry.unwrap();
        
        let types = registry.list_transport_types().await;
        assert!(types.contains(&TransportType::Tcp));
        assert!(types.contains(&TransportType::Mock));
    }
    
    #[tokio::test]
    async fn test_registry_stats() {
        let registry = TransportRegistry::default().unwrap();
        let stats = registry.stats().await;
        
        assert_eq!(stats.registered_types, 0);
        assert_eq!(stats.created_instances, 0);
        assert_eq!(stats.creation_failures, 0);
    }
} 