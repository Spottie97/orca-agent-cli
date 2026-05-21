use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::schema::OllamaModelConfig;
use crate::providers::traits::{
    ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind, ProviderRequest,
    ProviderResponse,
};

const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_MAX_RESPONSE_BYTES: u64 = 10_485_760; // 10 MiB

pub struct OllamaProvider {
    name: String,
    base_url: String,
    default_model: String,
    client: Client,
    timeout_seconds: u64,
    max_response_bytes: u64,
    api_key: Option<String>,
    api_key_env_var: Option<String>,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    model: String,
    response: String,
    #[allow(dead_code)]
    #[serde(default)]
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            name: "ollama".to_string(),
            base_url: base_url.into(),
            default_model: default_model.into(),
            client: Client::new(),
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            api_key: None,
            api_key_env_var: None,
        }
    }

    pub fn from_config(config: &OllamaModelConfig) -> Self {
        let (api_key, api_key_env_var) = if config.api_key_env_var.is_empty() {
            (None, None)
        } else {
            match std::env::var(&config.api_key_env_var) {
                Ok(val) if !val.is_empty() => (Some(val), Some(config.api_key_env_var.clone())),
                _ => (None, Some(config.api_key_env_var.clone())),
            }
        };

        Self {
            name: "ollama".to_string(),
            base_url: config.base_url.clone(),
            default_model: config.default_model.clone(),
            client: Client::new(),
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            api_key,
            api_key_env_var,
        }
    }

    fn build_prompt(request: &ProviderRequest) -> String {
        match &request.context {
            Some(ctx) if !ctx.is_empty() => format!("{}\n\n{}", ctx, request.prompt),
            _ => request.prompt.clone(),
        }
    }

    fn is_transient_error(err: &ProviderError) -> bool {
        matches!(
            err,
            ProviderError::ApiError(msg)
                if msg.contains("timeout")
                    || msg.contains("connection")
                    || msg.contains("503")
                    || msg.contains("429")
                    || msg.contains("502")
                    || msg.contains("504")
        )
    }

    async fn execute_once(
        &self,
        request: &ProviderRequest,
    ) -> Result<ProviderResponse, ProviderError> {
        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
        let model = request
            .model_id
            .clone()
            .unwrap_or_else(|| self.default_model.clone());
        let prompt = Self::build_prompt(request);

        let body = OllamaGenerateRequest {
            model: model.clone(),
            prompt,
            stream: false,
        };

        if let Some(ref var_name) = self.api_key_env_var {
            if self.api_key.is_none() {
                return Err(ProviderError::NotConfigured(format!(
                    "Ollama API key environment variable '{}' is not set or empty",
                    var_name
                )));
            }
        }

        let mut request_builder = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .json(&body);

        if let Some(ref key) = self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = request_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                ProviderError::ApiError(format!("Ollama request timed out: {}", e))
            } else if e.is_connect() {
                ProviderError::ApiError(format!("Ollama connection failed: {}", e))
            } else {
                ProviderError::ApiError(format!("Ollama request failed: {}", e))
            }
        })?;

        let status = response.status();
        let content_length = response.content_length();

        if let Some(len) = content_length {
            if len > self.max_response_bytes {
                return Err(ProviderError::ApiError(format!(
                    "Ollama response too large: {} bytes (max {})",
                    len, self.max_response_bytes
                )));
            }
        }

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(format!(
                "Ollama returned HTTP {}: {}",
                status, text
            )));
        }

        let parsed: OllamaGenerateResponse = response.json().await.map_err(|e| {
            ProviderError::ApiError(format!("Failed to parse Ollama response: {}", e))
        })?;

        let duration_ms = parsed.total_duration.map(|d| d / 1_000_000);

        Ok(ProviderResponse {
            task_id: request.task_id.clone(),
            provider: self.kind(),
            model_id: parsed.model,
            output: parsed.response,
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms,
            input_tokens: parsed.prompt_eval_count,
            output_tokens: parsed.eval_count,
            context_packet_path: None,
            prompt_included_context: false,
            prompt_sections_included: Vec::new(),
            bridge_diagnostics: None,
        })
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
        let max_retries = 2;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.execute_once(&request).await {
                Ok(mut response) => {
                    let elapsed = std::time::Instant::now().elapsed().as_millis() as u64;
                    if response.duration_ms.is_none() {
                        response.duration_ms = Some(elapsed);
                    }
                    return Ok(response);
                }
                Err(err) => {
                    if attempt < max_retries && Self::is_transient_error(&err) {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            200 * (attempt as u64 + 1),
                        ))
                        .await;
                        last_error = Some(err);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::ApiError("Ollama execution failed after retries".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

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

    #[tokio::test]
    async fn test_ollama_provider_sends_expected_request() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "qwen",
                "response": "Hello from Ollama",
                "done": true,
                "total_duration": 1500000000,
                "prompt_eval_count": 10,
                "eval_count": 5,
            })))
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri(), "qwen");
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let response = provider.execute(request).await.unwrap();
        assert_eq!(response.task_id, "TASK-001");
        assert_eq!(response.provider, ProviderKind::Ollama);
        assert_eq!(response.output, "Hello from Ollama");
        assert_eq!(response.status, ExecutionStatus::Success);
        assert_eq!(response.duration_ms, Some(1500));
        assert_eq!(response.input_tokens, Some(10));
        assert_eq!(response.output_tokens, Some(5));
    }

    #[tokio::test]
    async fn test_ollama_provider_rejects_oversized_response() {
        let server = MockServer::start().await;

        let large_body = "x".repeat(2048);
        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(large_body.clone())
                    .append_header("content-length", "2048"),
            )
            .mount(&server)
            .await;

        let provider = OllamaProvider {
            name: "ollama".to_string(),
            base_url: server.uri(),
            default_model: "qwen".to_string(),
            client: Client::new(),
            timeout_seconds: 120,
            max_response_bytes: 1024,
            api_key: None,
            api_key_env_var: None,
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
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("too large"),
            "Expected error about oversized response, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_ollama_provider_handles_http_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri(), "qwen");
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let result = provider.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("500"));
    }

    #[tokio::test]
    async fn test_ollama_provider_retries_transient_errors() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Unavailable"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "qwen",
                "response": "Recovered",
                "done": true,
            })))
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri(), "qwen");
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let response = provider.execute(request).await.unwrap();
        assert_eq!(response.output, "Recovered");
    }

    #[tokio::test]
    async fn test_ollama_provider_no_auth_when_not_configured() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "qwen",
                "response": "No auth needed",
                "done": true,
            })))
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri(), "qwen");
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let response = provider.execute(request).await.unwrap();
        assert_eq!(response.output, "No auth needed");
    }

    #[tokio::test]
    async fn test_ollama_provider_sends_bearer_when_configured() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .and(header("Authorization", "Bearer ollama-cloud-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "kimi-k2.6",
                "response": "Cloud OK",
                "done": true,
            })))
            .mount(&server)
            .await;

        let provider = OllamaProvider {
            name: "ollama".to_string(),
            base_url: server.uri(),
            default_model: "kimi-k2.6".to_string(),
            client: Client::new(),
            timeout_seconds: 120,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            api_key: Some("ollama-cloud-key".to_string()),
            api_key_env_var: Some("OLLAMA_API_KEY".to_string()),
        };

        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "Hello".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };

        let response = provider.execute(request).await.unwrap();
        assert_eq!(response.output, "Cloud OK");
    }

    #[tokio::test]
    async fn test_ollama_provider_errors_when_env_var_missing() {
        let provider = OllamaProvider {
            name: "ollama".to_string(),
            base_url: "https://ollama.com".to_string(),
            default_model: "kimi-k2.6".to_string(),
            client: Client::new(),
            timeout_seconds: 120,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            api_key: None,
            api_key_env_var: Some("OLLAMA_API_KEY_DEFINITELY_MISSING".to_string()),
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
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("OLLAMA_API_KEY_DEFINITELY_MISSING"),
            "Expected error mentioning env var name, got: {}",
            err
        );
        assert!(
            err.contains("not set or empty"),
            "Expected clear missing-env error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_ollama_provider_errors_when_env_var_empty() {
        let provider = OllamaProvider {
            name: "ollama".to_string(),
            base_url: "https://ollama.com".to_string(),
            default_model: "kimi-k2.6".to_string(),
            client: Client::new(),
            timeout_seconds: 120,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            api_key: None,
            api_key_env_var: Some("OLLAMA_API_KEY".to_string()),
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
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not set or empty"),
            "Expected clear empty-env error, got: {}",
            err
        );
    }

    #[test]
    fn test_ollama_provider_secret_not_in_error_message() {
        // If an error were to accidentally contain the secret value,
        // redaction should catch it. Here we verify the error message
        // only contains the env var name, never the key value.
        use crate::utils::redact::redact_secrets;

        let err_msg = "Ollama API key environment variable 'OLLAMA_API_KEY' is not set or empty";
        let redacted = redact_secrets(err_msg);
        assert_eq!(
            redacted, err_msg,
            "Error message should not trigger redaction since it never contains the secret"
        );

        // Ensure that if someone accidentally includes the key, redaction works
        let leaky = "Authorization: Bearer sk-ollama-secret-123";
        let redacted_leaky = redact_secrets(leaky);
        assert!(
            !redacted_leaky.contains("sk-ollama-secret-123"),
            "Redaction should remove the secret value"
        );
    }
}
