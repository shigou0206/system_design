//! Event storage implementations

pub mod memory;
pub mod sqlite;
pub mod postgres;

use crate::core::traits::EventStorage;
use crate::core::EventBusResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Re-export storage implementations
pub use memory::MemoryStorage;
pub use sqlite::SqliteStorage;
pub use postgres::PostgresStorage;

/// Storage configuration enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageConfig {
    /// In-memory storage (for testing/development)
    Memory { 
        max_events: usize 
    },
    /// SQLite storage (for single-node deployments)
    Sqlite { 
        database_url: String 
    },
    /// PostgreSQL storage (for production deployments)
    Postgres {
        database_url: String,
        max_connections: u32,
        enable_partitioning: bool,
    },
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig::Memory { max_events: 10000 }
    }
}

/// Create a storage instance based on configuration
pub async fn create_storage(config: &StorageConfig) -> EventBusResult<Arc<dyn EventStorage>> {
    let storage: Arc<dyn EventStorage> = match config {
        StorageConfig::Memory { max_events } => {
            let storage = MemoryStorage::with_limits(*max_events);
            Arc::new(storage)
        }
        StorageConfig::Sqlite { database_url } => {
            let storage = SqliteStorage::new(database_url).await?;
            Arc::new(storage)
        }
        StorageConfig::Postgres { database_url, max_connections, enable_partitioning } => {
            let postgres_config = postgres::PostgresConfig {
                database_url: database_url.clone(),
                max_connections: *max_connections,
                enable_partitioning: *enable_partitioning,
                ..Default::default()
            };
            
            let storage = PostgresStorage::with_config(postgres_config).await?;
            Arc::new(storage)
        }
    };
    
    // Initialize the storage
    storage.initialize().await?;
    
    Ok(storage)
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