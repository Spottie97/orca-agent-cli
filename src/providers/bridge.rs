use crate::bridge::subprocess::{run_subprocess, SubprocessError};
use crate::providers::traits::{
    CostEstimate, ExecutionStatus, Provider, ProviderCapability, ProviderError, ProviderKind,
    ProviderRequest, ProviderResponse,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct BridgeProvider {
    name: String,
    kind: ProviderKind,
    command: String,
    args: Vec<String>,
    use_stdin: bool,
    working_dir: Option<PathBuf>,
    env: HashMap<String, String>,
    timeout_seconds: u64,
    max_output_bytes: usize,
    model: String,
}

impl BridgeProvider {
    pub fn new(
        name: impl Into<String>,
        kind: ProviderKind,
        command: impl Into<String>,
        args: Vec<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            command: command.into(),
            args,
            use_stdin: true,
            working_dir: None,
            env: HashMap::new(),
            timeout_seconds: 600,
            max_output_bytes: 100_000,
            model: model.into(),
        }
    }

    pub fn with_stdin(mut self, use_stdin: bool) -> Self {
        self.use_stdin = use_stdin;
        self
    }

    pub fn with_working_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.working_dir = dir;
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }

    pub fn with_max_output(mut self, max: usize) -> Self {
        self.max_output_bytes = max;
        self
    }
}

impl BridgeProvider {
    pub fn from_claude_code_config(
        config: &crate::bridge::config::ExternalBridgeModelConfig,
    ) -> Option<Self> {
        let cmd = config.command.as_ref()?;
        Some(
            Self::new(
                "claude_code",
                ProviderKind::ClaudeCode,
                cmd,
                config.args.clone(),
                config
                    .model
                    .clone()
                    .unwrap_or_else(|| "claude-code".to_string()),
            )
            .with_stdin(config.stdin)
            .with_timeout(config.timeout_seconds)
            .with_max_output(config.max_output_bytes)
            .with_env(config.env.clone())
            .with_working_dir(config.working_directory.as_ref().map(PathBuf::from)),
        )
    }

    pub fn from_codex_config(config: &crate::config::schema::CodexModelConfig) -> Option<Self> {
        let cmd = config.command.as_ref()?;
        Some(
            Self::new(
                "codex",
                ProviderKind::Codex,
                cmd,
                config.args.clone(),
                config
                    .model
                    .clone()
                    .unwrap_or_else(|| config.default_model.clone()),
            )
            .with_stdin(config.stdin)
            .with_timeout(config.timeout_seconds)
            .with_max_output(config.max_output_bytes)
            .with_env(config.env.clone())
            .with_working_dir(config.working_directory.as_ref().map(PathBuf::from)),
        )
    }

    pub fn from_cursor_config(config: &crate::config::schema::CursorModelConfig) -> Option<Self> {
        let cmd = config.command.as_ref()?;
        Some(
            Self::new(
                "cursor",
                ProviderKind::Cursor,
                cmd,
                config.args.clone(),
                config
                    .model
                    .clone()
                    .unwrap_or_else(|| config.composer_model_id.clone()),
            )
            .with_stdin(config.stdin)
            .with_timeout(config.timeout_seconds)
            .with_max_output(config.max_output_bytes)
            .with_env(config.env.clone())
            .with_working_dir(config.working_directory.as_ref().map(PathBuf::from)),
        )
    }
}

#[async_trait]
impl Provider for BridgeProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[
            ProviderCapability::Chat,
            ProviderCapability::Completion,
            ProviderCapability::Planning,
            ProviderCapability::Summarization,
        ]
    }

    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let stdin_input = if self.use_stdin {
            Some(request.prompt.as_str())
        } else {
            None
        };

        let result = run_subprocess(
            &self.command,
            &self.args,
            stdin_input,
            self.working_dir.as_deref(),
            &self.env,
            self.timeout_seconds,
            self.max_output_bytes,
        )
        .await;

        let subprocess_result = match result {
            Ok(r) => r,
            Err(SubprocessError::CommandNotFound(cmd)) => {
                return Err(ProviderError::NotConfigured(format!(
                    "Bridge command not found: {cmd}"
                )));
            }
            Err(SubprocessError::Timeout(secs)) => {
                return Err(ProviderError::ExecutionFailed(format!(
                    "Bridge timed out after {secs}s"
                )));
            }
            Err(SubprocessError::NonZeroExit(code)) => {
                return Err(ProviderError::ExecutionFailed(format!(
                    "Bridge exited with code {code}"
                )));
            }
            Err(SubprocessError::InvalidUtf8) => {
                return Err(ProviderError::ExecutionFailed(
                    "Bridge output contained invalid UTF-8".to_string(),
                ));
            }
            Err(SubprocessError::Io(msg)) => {
                return Err(ProviderError::Io(std::io::Error::other(msg)));
            }
            Err(SubprocessError::EmptyOutput) => {
                return Err(ProviderError::ExecutionFailed(
                    "Bridge produced empty output".to_string(),
                ));
            }
        };

        let status = if subprocess_result.exit_code == 0 {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failure
        };

        let input_chars = request.prompt.len();
        let output_chars = subprocess_result.stdout.len();

        Ok(ProviderResponse {
            task_id: request.task_id,
            provider: self.kind,
            model_id: request.model_id.unwrap_or_else(|| self.model.clone()),
            output: subprocess_result.stdout,
            status,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: Some(subprocess_result.duration_ms),
            input_tokens: Some((input_chars / 4) as u32),
            output_tokens: Some((output_chars / 4) as u32),
            context_packet_path: None,
            prompt_included_context: false,
            prompt_sections_included: Vec::new(),
        })
    }

    fn estimate_cost(&self, request: &ProviderRequest) -> CostEstimate {
        let input_tokens = (request.prompt.len() / 4) as u32;
        CostEstimate {
            input_tokens,
            output_tokens: 0,
            cost_usd: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_provider() -> BridgeProvider {
        BridgeProvider::new(
            "test-bridge",
            ProviderKind::ClaudeCode,
            if cfg!(windows) { "cmd" } else { "sh" },
            if cfg!(windows) {
                vec!["/C".to_string(), "echo hello from bridge".to_string()]
            } else {
                vec!["-c".to_string(), "echo 'hello from bridge'".to_string()]
            },
            "test-model",
        )
    }

    #[tokio::test]
    async fn test_bridge_provider_executes_command() {
        let provider = test_provider();
        let request = ProviderRequest {
            task_id: "TASK-001".to_string(),
            prompt: "ignored in this test".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };
        let response = provider.execute(request).await.unwrap();
        assert!(response.output.contains("hello from bridge"));
        assert_eq!(response.provider, ProviderKind::ClaudeCode);
        assert_eq!(response.status, ExecutionStatus::Success);
    }

    #[tokio::test]
    async fn test_bridge_provider_command_not_found() {
        let provider = BridgeProvider::new(
            "missing",
            ProviderKind::Codex,
            "definitely_not_real_12345",
            vec![],
            "codex",
        );
        let request = ProviderRequest {
            task_id: "TASK-002".to_string(),
            prompt: "test".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };
        let result = provider.execute(request).await;
        assert!(matches!(result, Err(ProviderError::NotConfigured(_))));
    }

    #[tokio::test]
    async fn test_bridge_provider_name_and_kind() {
        let provider = BridgeProvider::new(
            "my-bridge",
            ProviderKind::ClaudeCode,
            "echo",
            vec![],
            "claude-code",
        );
        assert_eq!(provider.name(), "my-bridge");
        assert_eq!(provider.kind(), ProviderKind::ClaudeCode);
    }

    #[test]
    fn test_bridge_provider_capabilities() {
        let provider = test_provider();
        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::Chat));
        assert!(caps.contains(&ProviderCapability::Completion));
    }

    #[test]
    fn test_bridge_provider_estimate_cost() {
        let provider = test_provider();
        let request = ProviderRequest {
            task_id: "TASK-003".to_string(),
            prompt: "abcd".to_string(),
            model_id: None,
            context: None,
            max_tokens: None,
        };
        let cost = provider.estimate_cost(&request);
        assert_eq!(cost.input_tokens, 1);
        assert_eq!(cost.cost_usd, 0.0);
    }
}
