use async_trait::async_trait;

use super::traits::{
    ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind, ProviderRequest,
    ProviderResponse,
};

pub struct CursorProvider {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    composer_model_id: String,
    #[allow(dead_code)]
    premium_model_id: String,
}

impl CursorProvider {
    pub fn new(composer_model_id: impl Into<String>, premium_model_id: impl Into<String>) -> Self {
        Self {
            name: "cursor".to_string(),
            composer_model_id: composer_model_id.into(),
            premium_model_id: premium_model_id.into(),
        }
    }
}

#[async_trait]
impl Provider for CursorProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Cursor
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[
            ProviderCapability::Chat,
            ProviderCapability::Completion,
            ProviderCapability::RepoAware,
            ProviderCapability::MultiFileEdit,
            ProviderCapability::Streaming,
        ]
    }

    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        // TODO: implement Cursor integration (no Node.js runtime required)
        // This is a placeholder. Real integration will use Cursor SDK or REST API.
        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: self.composer_model_id.clone(),
            output: "Cursor provider skeleton: not yet implemented.".to_string(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
            context_packet_path: None,
            prompt_included_context: false,
            prompt_sections_included: Vec::new(),
            bridge_diagnostics: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_provider_kind() {
        let provider = CursorProvider::new("composer-2.5", "cursor-premium");
        assert_eq!(provider.kind(), ProviderKind::Cursor);
    }

    #[test]
    fn test_cursor_provider_capabilities() {
        let provider = CursorProvider::new("composer-2.5", "cursor-premium");
        assert!(provider.supports(ProviderCapability::RepoAware));
        assert!(provider.supports(ProviderCapability::MultiFileEdit));
    }
}
