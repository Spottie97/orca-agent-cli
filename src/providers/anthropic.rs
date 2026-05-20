use async_trait::async_trait;
use reqwest::Client;

use super::traits::{
    ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind, ProviderRequest,
    ProviderResponse,
};

pub struct AnthropicProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    default_model: String,
    #[allow(dead_code)]
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            name: "anthropic".to_string(),
            api_key: api_key.into(),
            default_model: default_model.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[
            ProviderCapability::Chat,
            ProviderCapability::Completion,
            ProviderCapability::Planning,
            ProviderCapability::Streaming,
            ProviderCapability::MultiFileEdit,
        ]
    }

    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        // TODO: implement real Anthropic API call
        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: self.default_model.clone(),
            output: "Anthropic provider skeleton: not yet implemented.".to_string(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_kind() {
        let provider = AnthropicProvider::new("test-key", "opus");
        assert_eq!(provider.kind(), ProviderKind::Anthropic);
    }

    #[test]
    fn test_anthropic_provider_capabilities() {
        let provider = AnthropicProvider::new("test-key", "opus");
        assert!(provider.supports(ProviderCapability::Chat));
        assert!(provider.supports(ProviderCapability::Planning));
    }
}
