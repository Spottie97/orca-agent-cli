use async_trait::async_trait;

use super::traits::{
    CostEstimate, ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind,
    ProviderRequest, ProviderResponse,
};

pub struct MockProvider {
    name: String,
}

impl MockProvider {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new("mock")
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Mock
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
        let task_id = request.task_id.clone();
        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: "mock-model".to_string(),
            output: format!("Mock execution completed for task {}", task_id),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        })
    }

    fn estimate_cost(&self, request: &ProviderRequest) -> CostEstimate {
        CostEstimate {
            input_tokens: request.prompt.len() as u32 / 4,
            output_tokens: 100,
            cost_usd: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_execute() {
        let provider = MockProvider::default();
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };
        let response = provider.execute(request).await.unwrap();
        assert_eq!(response.task_id, "TASK-001");
        assert_eq!(response.provider, ProviderKind::Mock);
        assert_eq!(response.status, ExecutionStatus::Success);
    }

    #[test]
    fn test_mock_provider_capabilities() {
        let provider = MockProvider::default();
        assert!(provider.supports(ProviderCapability::Chat));
        assert!(provider.supports(ProviderCapability::Completion));
        assert!(!provider.supports(ProviderCapability::Embeddings));
    }

    #[test]
    fn test_mock_provider_estimate_cost() {
        let provider = MockProvider::default();
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello world".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };
        let cost = provider.estimate_cost(&request);
        assert_eq!(cost.input_tokens, 2);
        assert_eq!(cost.cost_usd, 0.0);
    }
}
