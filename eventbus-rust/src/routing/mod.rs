//! Event routing and rule engine implementations

pub mod memory_router;
pub mod rule_engine;

pub use memory_router::MemoryEventRouter;
pub use rule_engine::MemoryRuleEngine;

// Re-export traits
pub use crate::core::traits::RuleEngine;

/// Event router trait (currently using memory implementation)
pub type EventRouter = MemoryEventRouter; 