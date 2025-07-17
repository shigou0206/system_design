//! Memory-based rule engine implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::{
    EventEnvelope, EventTriggerRule, ToolInvocation,
    traits::{RuleEngine, EventBusResult},
    EventBusError
};

/// Memory-based rule engine implementation
#[derive(Debug)]
pub struct MemoryRuleEngine {
    /// Registered rules indexed by ID
    rules: RwLock<HashMap<String, EventTriggerRule>>,
}

impl MemoryRuleEngine {
    /// Create a new memory rule engine
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuleEngine for MemoryRuleEngine {
    async fn register_rule(&self, rule: EventTriggerRule) -> EventBusResult<()> {
        let mut rules = self.rules.write()
            .map_err(|_| EventBusError::internal("Failed to acquire write lock on rules"))?;
        
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }
    
    async fn remove_rule(&self, rule_id: &str) -> EventBusResult<()> {
        let mut rules = self.rules.write()
            .map_err(|_| EventBusError::internal("Failed to acquire write lock on rules"))?;
        
        rules.remove(rule_id)
            .ok_or_else(|| EventBusError::not_found(format!("rule: {}", rule_id)))?;
        
        Ok(())
    }
    
    async fn list_rules(&self) -> EventBusResult<Vec<EventTriggerRule>> {
        let rules = self.rules.read()
            .map_err(|_| EventBusError::internal("Failed to acquire read lock on rules"))?;
        
        Ok(rules.values().cloned().collect())
    }
    
    async fn process_event(&self, event: &EventEnvelope) -> EventBusResult<Vec<ToolInvocation>> {
        let rules = self.rules.read()
            .map_err(|_| EventBusError::internal("Failed to acquire read lock on rules"))?;
        
        let mut invocations = Vec::new();
        
        for rule in rules.values() {
            if rule.matches(event) {
                match &rule.action {
                    crate::core::RuleAction::InvokeTool { tool_id, input } => {
                        invocations.push(ToolInvocation::new(tool_id.clone(), input.clone()));
                    }
                    crate::core::RuleAction::EmitEvent { .. } => {
                        // TODO: Handle event emission
                    }
                    crate::core::RuleAction::Sequence { .. } => {
                        // TODO: Handle sequence actions
                    }
                    crate::core::RuleAction::Forward { .. } => {
                        // TODO: Handle forward action
                    }
                    crate::core::RuleAction::Transform { .. } => {
                        // TODO: Handle transform action
                    }
                    crate::core::RuleAction::ExecuteTool { .. } => {
                        // TODO: Handle execute tool action
                    }
                    crate::core::RuleAction::Webhook { .. } => {
                        // TODO: Handle webhook action
                    }
                    crate::core::RuleAction::Log { .. } => {
                        // TODO: Handle log action
                    }
                    crate::core::RuleAction::Custom { .. } => {
                        // TODO: Handle custom action
                    }
                }
            }
        }
        
        Ok(invocations)
    }
    
    async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> EventBusResult<()> {
        let mut rules = self.rules.write()
            .map_err(|_| EventBusError::internal("Failed to acquire write lock on rules"))?;
        
        let rule = rules.get_mut(rule_id)
            .ok_or_else(|| EventBusError::not_found(format!("rule: {}", rule_id)))?;
        
        rule.enabled = enabled;
        Ok(())
    }
} 