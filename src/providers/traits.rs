use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Mock,
    Ollama,
    Anthropic,
    OpenAi,
    Cursor,
    Command,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderKind::Mock => write!(f, "mock"),
            ProviderKind::Ollama => write!(f, "ollama"),
            ProviderKind::Anthropic => write!(f, "anthropic"),
            ProviderKind::OpenAi => write!(f, "openai"),
            ProviderKind::Cursor => write!(f, "cursor"),
            ProviderKind::Command => write!(f, "command"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderCapability {
    Chat,
    Completion,
    Streaming,
    Embeddings,
    RepoAware,
    MultiFileEdit,
    Planning,
    Summarization,
    Compression,
    ImageInput,
    ToolUse,
}

impl std::fmt::Display for ProviderCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderCapability::Chat => write!(f, "chat"),
            ProviderCapability::Completion => write!(f, "completion"),
            ProviderCapability::Streaming => write!(f, "streaming"),
            ProviderCapability::Embeddings => write!(f, "embeddings"),
            ProviderCapability::RepoAware => write!(f, "repo-aware"),
            ProviderCapability::MultiFileEdit => write!(f, "multi-file-edit"),
            ProviderCapability::Planning => write!(f, "planning"),
            ProviderCapability::Summarization => write!(f, "summarization"),
            ProviderCapability::Compression => write!(f, "compression"),
            ProviderCapability::ImageInput => write!(f, "image-input"),
            ProviderCapability::ToolUse => write!(f, "tool-use"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub task_id: String,
    pub prompt: String,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub task_id: String,
    pub provider: ProviderKind,
    pub model_id: String,
    pub output: String,
    pub status: ExecutionStatus,
    #[serde(default)]
    pub files_changed: Vec<String>,
    #[serde(default)]
    pub suggested_memory_update: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure,
    Partial,
    Cancelled,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "success"),
            ExecutionStatus::Failure => write!(f, "failure"),
            ExecutionStatus::Partial => write!(f, "partial"),
            ExecutionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
}

impl Default for CostEstimate {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0.0,
        }
    }
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Provider not configured: {0}")]
    NotConfigured(String),
    #[error("API request failed: {0}")]
    ApiError(String),
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Unsupported capability: {0}")]
    UnsupportedCapability(ProviderCapability),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> ProviderKind;
    fn capabilities(&self) -> &[ProviderCapability];
    async fn execute(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError>;
    fn estimate_cost(&self, _request: &ProviderRequest) -> CostEstimate {
        CostEstimate::default()
    }
    fn supports(&self, capability: ProviderCapability) -> bool {
        self.capabilities().contains(&capability)
    }
}
