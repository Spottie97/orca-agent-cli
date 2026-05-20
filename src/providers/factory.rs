use crate::config::schema::{Config, ModelsConfig};
use crate::providers::{
    mock::MockProvider, ollama::OllamaProvider, openai::OpenAiProvider, traits::Provider,
    ProviderKind,
};

/// Create a provider instance from a routing decision and config.
/// Returns a boxed trait object that can be used for execution.
pub fn create_provider(kind: ProviderKind, config: &Config) -> Box<dyn Provider> {
    match kind {
        ProviderKind::OpenAi => Box::new(OpenAiProvider::from_config(&config.models.openai)),
        ProviderKind::Ollama => {
            if config.models.ollama.enabled {
                Box::new(OllamaProvider::from_config(&config.models.ollama))
            } else {
                Box::new(MockProvider::default())
            }
        }
        _ => Box::new(MockProvider::default()),
    }
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
}
