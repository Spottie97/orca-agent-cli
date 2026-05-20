use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::traits::{
    CostEstimate, ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind,
    ProviderRequest, ProviderResponse,
};
use crate::config::schema::OpenAiModelConfig;
use crate::utils::redact::redact_secrets;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

pub struct OpenAiProvider {
    name: String,
    api_key: String,
    base_url: String,
    default_model: String,
    timeout: Duration,
    max_input_tokens: u32,
    max_output_tokens: u32,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            name: "openai".to_string(),
            api_key: api_key.into(),
            base_url: "https://api.openai.com".to_string(),
            default_model: default_model.into(),
            timeout: Duration::from_secs(120),
            max_input_tokens: 4096,
            max_output_tokens: 2048,
            client: Client::new(),
        }
    }

    pub fn from_config(config: &OpenAiModelConfig) -> Self {
        let api_key = std::env::var(&config.api_key_env_var).unwrap_or_else(|_| String::new());
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            name: "openai".to_string(),
            api_key,
            base_url: config.base_url.clone(),
            default_model: config.default_model.clone(),
            timeout: Duration::from_secs(config.timeout_seconds),
            max_input_tokens: config.max_input_tokens,
            max_output_tokens: config.max_output_tokens,
            client,
        }
    }

    fn approximate_token_count(text: &str) -> u32 {
        // Very rough approximation: 1 token ~ 4 characters for English text
        ((text.len() as f64) / 4.0).ceil() as u32
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
        let model = request
            .model_id
            .clone()
            .unwrap_or_else(|| self.default_model.clone());
        let prompt = request.prompt;
        let context = request.context.unwrap_or_default();

        let combined_input = format!("{}\n\n{}", context, prompt);
        let input_tokens = Self::approximate_token_count(&combined_input);

        if input_tokens > self.max_input_tokens {
            return Err(ProviderError::InvalidRequest(format!(
                "Input exceeds max_input_tokens limit: {} > {}",
                input_tokens, self.max_input_tokens
            )));
        }

        let messages = vec![
            ChatCompletionMessage {
                role: "system".to_string(),
                content: "You are a helpful coding assistant.".to_string(),
            },
            ChatCompletionMessage {
                role: "user".to_string(),
                content: combined_input,
            },
        ];

        let max_tokens = request
            .max_tokens
            .unwrap_or(self.max_output_tokens)
            .min(self.max_output_tokens);

        let body = ChatCompletionRequest {
            model,
            messages,
            max_tokens: Some(max_tokens),
        };

        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let mut builder = self.client.post(&url);
        if !self.api_key.is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }

        let response = builder.json(&body).send().await.map_err(|e| {
            let msg = redact_secrets(&e.to_string());
            if e.is_timeout() {
                ProviderError::ApiError(format!("Request timed out after {:?}", self.timeout))
            } else {
                ProviderError::ApiError(format!("HTTP request failed: {}", msg))
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            let sanitized = redact_secrets(&text);
            return match status.as_u16() {
                401 => Err(ProviderError::AuthError(format!(
                    "Authentication failed (401): {}",
                    sanitized
                ))),
                429 => Err(ProviderError::RateLimited),
                _ => Err(ProviderError::ApiError(format!(
                    "HTTP {}: {}",
                    status, sanitized
                ))),
            };
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(format!("Failed to parse response: {}", e)))?;

        let output = completion
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let _output_tokens = Self::approximate_token_count(&output);

        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind(),
            model_id: request
                .model_id
                .unwrap_or_else(|| self.default_model.clone()),
            output,
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
        })
    }

    fn estimate_cost(&self, request: &ProviderRequest) -> CostEstimate {
        let prompt = &request.prompt;
        let context = request.context.as_deref().unwrap_or("");
        let input_tokens = Self::approximate_token_count(&format!("{} {}", context, prompt));
        let output_tokens = self.max_output_tokens / 2; // rough guess
        CostEstimate {
            input_tokens,
            output_tokens,
            cost_usd: 0.0, // local providers don't have fixed pricing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    #[tokio::test]
    async fn test_openai_provider_sends_expected_request() {
        let server = MockServer::start().await;

        let response_body = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Hello from mock server"
                    }
                }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_json(serde_json::json!({
                "model": "test-model",
                "messages": [
                    {"role": "system", "content": "You are a helpful coding assistant."},
                    {"role": "user", "content": "\n\nHello"}
                ],
                "max_tokens": 100
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .expect(1)
            .mount(&server)
            .await;

        let provider = OpenAiProvider {
            name: "openai".to_string(),
            api_key: "sk-test".to_string(),
            base_url: server.uri(),
            default_model: "test-model".to_string(),
            timeout: Duration::from_secs(5),
            max_input_tokens: 4096,
            max_output_tokens: 2048,
            client: Client::new(),
        };

        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: Some("test-model".to_string()),
            context: None,
            max_tokens: Some(100),
        };

        let response = provider.execute(request).await.unwrap();
        assert_eq!(response.output, "Hello from mock server");
        assert_eq!(response.status, ExecutionStatus::Success);
    }

    #[tokio::test]
    async fn test_openai_provider_handles_http_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .expect(1)
            .mount(&server)
            .await;

        let provider = OpenAiProvider {
            name: "openai".to_string(),
            api_key: "sk-test".to_string(),
            base_url: server.uri(),
            default_model: "test-model".to_string(),
            timeout: Duration::from_secs(5),
            max_input_tokens: 4096,
            max_output_tokens: 2048,
            client: Client::new(),
        };

        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let result = provider.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));
        // Secrets should be redacted from error messages
        assert!(!err.to_string().contains("sk-test"));
    }

    #[tokio::test]
    async fn test_openai_provider_handles_auth_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .expect(1)
            .mount(&server)
            .await;

        let provider = OpenAiProvider {
            name: "openai".to_string(),
            api_key: "sk-test".to_string(),
            base_url: server.uri(),
            default_model: "test-model".to_string(),
            timeout: Duration::from_secs(5),
            max_input_tokens: 4096,
            max_output_tokens: 2048,
            client: Client::new(),
        };

        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let result = provider.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Authentication failed"));
    }

    #[tokio::test]
    async fn test_openai_provider_handles_timeout() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .expect(1)
            .mount(&server)
            .await;

        let provider = OpenAiProvider {
            name: "openai".to_string(),
            api_key: String::new(),
            base_url: server.uri(),
            default_model: "test-model".to_string(),
            timeout: Duration::from_millis(100),
            max_input_tokens: 4096,
            max_output_tokens: 2048,
            client: Client::builder()
                .timeout(Duration::from_millis(100))
                .build()
                .unwrap(),
        };

        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let result = provider.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_openai_estimate_cost() {
        let provider = OpenAiProvider::new("test-key", "gpt-5");
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello world".to_string(),
            model_id: None,
            context: Some("Some context".to_string()),
            max_tokens: None,
        };
        let cost = provider.estimate_cost(&request);
        assert!(cost.input_tokens > 0);
        assert!(cost.output_tokens > 0);
    }

    #[test]
    fn test_openai_from_config_reads_env_var() {
        // This test documents the behavior; in CI the env var may not be set.
        let cfg = OpenAiModelConfig {
            enabled: true,
            base_url: "http://localhost:8080".to_string(),
            default_model: "llama3".to_string(),
            api_key_env_var: "NONEXISTENT_VAR_FOR_TEST".to_string(),
            role: "test".to_string(),
            timeout_seconds: 30,
            max_input_tokens: 2048,
            max_output_tokens: 1024,
        };
        let provider = OpenAiProvider::from_config(&cfg);
        assert_eq!(provider.api_key, "");
        assert_eq!(provider.base_url, "http://localhost:8080");
        assert_eq!(provider.default_model, "llama3");
        assert_eq!(provider.timeout, Duration::from_secs(30));
    }
}
