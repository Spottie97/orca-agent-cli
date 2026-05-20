use async_trait::async_trait;
use reqwest::Client;

use super::traits::{
    ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind, ProviderRequest,
    ProviderResponse,
};

pub struct OpenAiProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    default_model: String,
    #[allow(dead_code)]
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            name: "openai".to_string(),
            api_key: api_key.into(),
            default_model: default_model.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAi
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[
            ProviderCapability::Chat,
            ProviderCapability::Completion,
            ProviderCapability::ToolUse,
            ProviderCapability::MultiFileEdit,
        ]
    }

    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        // TODO: implement real OpenAI/Codex API call
        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: self.default_model.clone(),
            output: "OpenAI provider skeleton: not yet implemented.".to_string(),
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
    fn test_openai_provider_kind() {
        let provider = OpenAiProvider::new("test-key", "gpt-5");
        assert_eq!(provider.kind(), ProviderKind::OpenAi);
    }

    #[test]
    fn test_openai_provider_capabilities() {
        let provider = OpenAiProvider::new("test-key", "gpt-5");
        assert!(provider.supports(ProviderCapability::Chat));
        assert!(provider.supports(ProviderCapability::ToolUse));
    }
}
