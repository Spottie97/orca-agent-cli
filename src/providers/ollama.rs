use async_trait::async_trait;
use reqwest::Client;

use super::traits::{
    ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind, ProviderRequest,
    ProviderResponse,
};

pub struct OllamaProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    default_model: String,
    #[allow(dead_code)]
    client: Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            name: "ollama".to_string(),
            base_url: base_url.into(),
            default_model: default_model.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[
            ProviderCapability::Chat,
            ProviderCapability::Completion,
            ProviderCapability::Summarization,
            ProviderCapability::Compression,
        ]
    }

    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        // TODO: implement real Ollama HTTP call
        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: self.default_model.clone(),
            output: "Ollama provider skeleton: not yet implemented.".to_string(),
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
    fn test_ollama_provider_kind() {
        let provider = OllamaProvider::new("http://localhost:11434", "qwen");
        assert_eq!(provider.kind(), ProviderKind::Ollama);
    }

    #[test]
    fn test_ollama_provider_capabilities() {
        let provider = OllamaProvider::new("http://localhost:11434", "qwen");
        assert!(provider.supports(ProviderCapability::Chat));
        assert!(provider.supports(ProviderCapability::Completion));
    }
}
