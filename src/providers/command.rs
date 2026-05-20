use async_trait::async_trait;

use super::traits::{
    ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind, ProviderRequest,
    ProviderResponse,
};

pub struct CommandProvider {
    #[allow(dead_code)]
    name: String,
}

impl CommandProvider {
    pub fn new() -> Self {
        Self {
            name: "command".to_string(),
        }
    }
}

impl Default for CommandProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for CommandProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Command
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[ProviderCapability::Chat, ProviderCapability::Completion]
    }

    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        // TODO: implement subprocess execution based on request.prompt
        // This provider runs local shell commands instead of calling an API.
        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: "command".to_string(),
            output: "Command provider skeleton: not yet implemented.".to_string(),
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
    fn test_command_provider_kind() {
        let provider = CommandProvider::new();
        assert_eq!(provider.kind(), ProviderKind::Command);
    }

    #[test]
    fn test_command_provider_capabilities() {
        let provider = CommandProvider::new();
        assert!(provider.supports(ProviderCapability::Chat));
    }
}
