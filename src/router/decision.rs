use serde::{Deserialize, Serialize};

use crate::providers::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub provider: ProviderKind,
    pub model: String,
    pub reason: String,
    pub requires_approval: bool,
    pub risk: String,
    pub estimated_cost_class: String,
    pub fallback_provider: ProviderKind,
    pub notes: Vec<String>,
}

impl RoutingDecision {
    pub fn simple(
        provider: ProviderKind,
        model: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            reason: reason.into(),
            requires_approval: false,
            risk: "low".to_string(),
            estimated_cost_class: "cheap".to_string(),
            fallback_provider: ProviderKind::Mock,
            notes: Vec::new(),
        }
    }
}
