//! Contract system for plugin communication
//!
//! Contracts define interfaces that plugins can provide or consume.
//! This enables loose coupling between plugins.
//!
//! Note: Contract validation is not yet wired into the plugin loader.
//! These types are used for manifest parsing but full validation is pending.

#![allow(dead_code)] // Contract validation pending integration

use std::collections::HashMap;

pub mod integration;
pub mod memory;
pub mod skill;

/// Contract types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContractType {
    MemoryProvider,
    HookHandler,
    SkillProvider,
    IntegrationProvider { service: String },
    NotificationProvider,
}

impl ContractType {
    pub fn from_spec(contract: &str, service: Option<&str>) -> Option<Self> {
        match contract {
            "MemoryProvider" => Some(Self::MemoryProvider),
            "HookHandler" => Some(Self::HookHandler),
            "SkillProvider" => Some(Self::SkillProvider),
            "IntegrationProvider" => Some(Self::IntegrationProvider {
                service: service?.to_string(),
            }),
            "NotificationProvider" => Some(Self::NotificationProvider),
            _ => None,
        }
    }
}

/// Contract registry - maps contracts to providers
#[derive(Debug, Default)]
pub struct ContractRegistry {
    providers: HashMap<ContractType, String>, // contract -> plugin name
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin as providing a contract
    pub fn register(&mut self, contract: ContractType, plugin: String) -> eyre::Result<()> {
        if let Some(existing) = self.providers.get(&contract) {
            eyre::bail!("Contract {:?} already provided by plugin {}", contract, existing);
        }
        self.providers.insert(contract, plugin);
        Ok(())
    }

    /// Get the plugin that provides a contract
    pub fn get_provider(&self, contract: &ContractType) -> Option<&String> {
        self.providers.get(contract)
    }

    /// Check if a contract is available
    pub fn has_provider(&self, contract: &ContractType) -> bool {
        self.providers.contains_key(contract)
    }

    /// List all registered contracts
    pub fn list(&self) -> impl Iterator<Item = (&ContractType, &String)> {
        self.providers.iter()
    }
}
