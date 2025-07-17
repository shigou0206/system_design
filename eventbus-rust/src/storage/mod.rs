//! Storage backends for the event bus system
//! 
//! This module provides different storage implementations for events,
//! from in-memory storage for development to persistent storage for production.

pub mod memory;

#[cfg(feature = "persistence")]
pub mod sqlite;

#[cfg(feature = "persistence")]
pub mod postgres;

use std::sync::Arc;

use crate::core::{EventStorage, EventBusResult};
use crate::config::StorageConfig;

// Re-export implementations
pub use memory::MemoryStorage;

#[cfg(feature = "persistence")]
pub use sqlite::SqliteStorage;

#[cfg(feature = "persistence")]
pub use postgres::PostgresStorage;

/// Create a storage instance from configuration
pub async fn create_storage(config: &StorageConfig) -> EventBusResult<Arc<dyn EventStorage>> {
    match config {
        StorageConfig::Memory { .. } => {
            let storage = MemoryStorage::new();
            storage.initialize().await?;
            Ok(Arc::new(storage))
        }
        
        #[cfg(feature = "persistence")]
        StorageConfig::Sqlite { path, .. } => {
            let database_url = format!("sqlite:{}", path);
            let storage = SqliteStorage::new(&database_url).await?;
            storage.initialize().await?;
            Ok(Arc::new(storage))
        }
        
        #[cfg(feature = "persistence")]
        StorageConfig::Postgres { url, .. } => {
            let storage = PostgresStorage::new(url).await?;
            storage.initialize().await?;
            Ok(Arc::new(storage))
        }
        
        #[cfg(not(feature = "persistence"))]
        StorageConfig::Sqlite { .. } | StorageConfig::Postgres { .. } => {
            Err(crate::core::EventBusError::configuration(
                "Persistence features not enabled. Enable 'persistence' feature to use SQLite/Postgres storage"
            ))
        }
    }
}

/// Storage factory with connection pooling and caching
pub struct StorageFactory {
    /// Cache of created storage instances
    cache: dashmap::DashMap<String, Arc<dyn EventStorage>>,
}

impl StorageFactory {
    /// Create a new storage factory
    pub fn new() -> Self {
        Self {
            cache: dashmap::DashMap::new(),
        }
    }
    
    /// Get or create a storage instance
    pub async fn get_storage(&self, config: &StorageConfig) -> EventBusResult<Arc<dyn EventStorage>> {
        let key = format!("{:?}", config);
        
        if let Some(storage) = self.cache.get(&key) {
            return Ok(Arc::clone(&storage));
        }
        
        let storage = create_storage(config).await?;
        self.cache.insert(key, Arc::clone(&storage));
        
        Ok(storage)
    }
    
    /// Clear the storage cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
} 