use crate::config::schema::{Config, ModelsConfig};
use crate::providers::{
    bridge::BridgeProvider, mock::MockProvider, ollama::OllamaProvider, openai::OpenAiProvider,
    traits::Provider, ProviderKind,
};

/// Create a provider instance from a routing decision and config.
/// Returns a boxed trait object that can be used for execution.
pub fn create_provider(kind: ProviderKind, config: &Config) -> Box<dyn Provider> {
    create_provider_from_models(kind, &config.models)
}

/// Create a provider instance using explicit models config (for testing).
pub fn create_provider_from_models(kind: ProviderKind, models: &ModelsConfig) -> Box<dyn Provider> {
    match kind {
        ProviderKind::OpenAi => Box::new(OpenAiProvider::from_config(&models.openai)),
        ProviderKind::Ollama => {
            if models.ollama.enabled {
                Box::new(OllamaProvider::from_config(&models.ollama))
            } else {
                Box::new(MockProvider::default())
            }
        }
        ProviderKind::ClaudeCode => {
            if models.claude_code.enabled {
                if let Some(provider) = BridgeProvider::from_claude_code_config(&models.claude_code)
                {
                    Box::new(provider)
                } else {
                    Box::new(MockProvider::default())
                }
            } else {
                Box::new(MockProvider::default())
            }
        }
        ProviderKind::Codex => {
            if models.codex.enabled && models.codex.command.is_some() {
                if let Some(provider) = BridgeProvider::from_codex_config(&models.codex) {
                    Box::new(provider)
                } else {
                    Box::new(MockProvider::default())
                }
            } else {
                Box::new(MockProvider::default())
            }
        }
        ProviderKind::Cursor => {
            if models.cursor.enabled && models.cursor.command.is_some() {
                if let Some(provider) = BridgeProvider::from_cursor_config(&models.cursor) {
                    Box::new(provider)
                } else {
                    Box::new(MockProvider::default())
                }
            } else {
                // Preserve existing placeholder behavior when bridge not configured
                Box::new(MockProvider::default())
            }
        }
        _ => Box::new(MockProvider::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::ModelsConfig;

    #[test]
    fn test_factory_returns_openai_provider() {
        let models = ModelsConfig::default();
        let provider = create_provider_from_models(ProviderKind::OpenAi, &models);
        assert_eq!(provider.kind(), ProviderKind::OpenAi);
    }

    #[test]
    fn test_factory_returns_ollama_provider_when_enabled() {
        let models = ModelsConfig::default();
        let provider = create_provider_from_models(ProviderKind::Ollama, &models);
        assert_eq!(provider.kind(), ProviderKind::Ollama);
    }

    #[test]
    fn test_factory_returns_mock_for_unsupported() {
        let models = ModelsConfig::default();
        let provider = create_provider_from_models(ProviderKind::Mock, &models);
        assert_eq!(provider.kind(), ProviderKind::Mock);
    }

    #[test]
    fn test_factory_returns_mock_for_disabled_ollama() {
        let mut models = ModelsConfig::default();
        models.ollama.enabled = false;
        let provider = create_provider_from_models(ProviderKind::Ollama, &models);
        assert_eq!(provider.kind(), ProviderKind::Mock);
    }

    #[test]
    fn test_factory_returns_claude_code_bridge_when_enabled() {
        let mut models = ModelsConfig::default();
        models.claude_code.enabled = true;
        models.claude_code.command = Some("claude".to_string());
        let provider = create_provider_from_models(ProviderKind::ClaudeCode, &models);
        assert_eq!(provider.kind(), ProviderKind::ClaudeCode);
        assert_eq!(provider.name(), "claude_code");
    }

    #[test]
    fn test_factory_returns_mock_for_disabled_claude_code() {
        let models = ModelsConfig::default();
        let provider = create_provider_from_models(ProviderKind::ClaudeCode, &models);
        assert_eq!(provider.kind(), ProviderKind::Mock);
    }

    #[test]
    fn test_factory_returns_codex_bridge_when_configured() {
        let mut models = ModelsConfig::default();
        models.codex.enabled = true;
        models.codex.command = Some("codex".to_string());
        let provider = create_provider_from_models(ProviderKind::Codex, &models);
        assert_eq!(provider.kind(), ProviderKind::Codex);
        assert_eq!(provider.name(), "codex");
    }

    #[test]
    fn test_factory_returns_mock_for_codex_without_command() {
        let models = ModelsConfig::default();
        let provider = create_provider_from_models(ProviderKind::Codex, &models);
        assert_eq!(provider.kind(), ProviderKind::Mock);
    }

    #[test]
    fn test_factory_returns_cursor_bridge_when_configured() {
        let mut models = ModelsConfig::default();
        models.cursor.enabled = true;
        models.cursor.command = Some("cursor".to_string());
        let provider = create_provider_from_models(ProviderKind::Cursor, &models);
        assert_eq!(provider.kind(), ProviderKind::Cursor);
        assert_eq!(provider.name(), "cursor");
    }

    #[test]
    fn test_factory_returns_mock_for_cursor_without_command() {
        let models = ModelsConfig::default();
        let provider = create_provider_from_models(ProviderKind::Cursor, &models);
        assert_eq!(provider.kind(), ProviderKind::Mock);
    }
}
