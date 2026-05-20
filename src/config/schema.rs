use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
    #[serde(default)]
    pub models: ModelsConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub repo_path: PathBuf,
    #[serde(default)]
    pub vault_path: Option<PathBuf>,
    #[serde(default = "default_orca_dir")]
    pub orca_dir: PathBuf,
}

fn default_orca_dir() -> PathBuf {
    PathBuf::from(".orca")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionConfig {
    #[serde(default = "default_execution_mode")]
    pub default_mode: String,
    #[serde(default = "default_true")]
    pub require_approval_for_premium: bool,
    #[serde(default = "default_true")]
    pub require_clean_git_for_cursor: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            default_mode: default_execution_mode(),
            require_approval_for_premium: true,
            require_clean_git_for_cursor: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModelsConfig {
    #[serde(default)]
    pub ollama: OllamaModelConfig,
    #[serde(default)]
    pub claude: ClaudeModelConfig,
    #[serde(default)]
    pub codex: CodexModelConfig,
    #[serde(default)]
    pub cursor: CursorModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OllamaModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ollama_base_url")]
    pub base_url: String,
    #[serde(default = "default_ollama_model")]
    pub default_model: String,
    #[serde(default = "default_ollama_role")]
    pub role: String,
}

impl Default for OllamaModelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: default_ollama_base_url(),
            default_model: default_ollama_model(),
            role: default_ollama_role(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_claude_provider")]
    pub provider: String,
    #[serde(default = "default_claude_model")]
    pub default_model: String,
    #[serde(default = "default_claude_role")]
    pub role: String,
    #[serde(default = "default_true")]
    pub approval_required_for_large_context: bool,
}

impl Default for ClaudeModelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: default_claude_provider(),
            default_model: default_claude_model(),
            role: default_claude_role(),
            approval_required_for_large_context: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_codex_provider")]
    pub provider: String,
    #[serde(default = "default_codex_model")]
    pub default_model: String,
    #[serde(default = "default_codex_role")]
    pub role: String,
}

impl Default for CodexModelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: default_codex_provider(),
            default_model: default_codex_model(),
            role: default_codex_role(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorModelConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cursor_composer_model")]
    pub composer_model_id: String,
    #[serde(default = "default_cursor_premium_model")]
    pub premium_model_id: String,
    #[serde(default = "default_cursor_provider")]
    pub provider: String,
    #[serde(default = "default_cursor_runtime")]
    pub default_runtime: String,
    #[serde(default = "default_false")]
    pub cloud_auto_create_pr: bool,
}

impl Default for CursorModelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            composer_model_id: default_cursor_composer_model(),
            premium_model_id: default_cursor_premium_model(),
            provider: default_cursor_provider(),
            default_runtime: default_cursor_runtime(),
            cloud_auto_create_pr: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingConfig {
    #[serde(default = "default_repo_executor")]
    pub default_repo_executor: String,
    #[serde(default = "default_scoped_executor")]
    pub default_scoped_executor: String,
    #[serde(default = "default_planner")]
    pub default_planner: String,
    #[serde(default = "default_summarizer")]
    pub default_summarizer: String,
    #[serde(default = "default_max_failures")]
    pub max_failures_before_escalation: u32,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            default_repo_executor: default_repo_executor(),
            default_scoped_executor: default_scoped_executor(),
            default_planner: default_planner(),
            default_summarizer: default_summarizer(),
            max_failures_before_escalation: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextConfig {
    #[serde(default = "default_max_tokens")]
    pub default_max_tokens: u32,
    #[serde(default = "default_hard_max_tokens")]
    pub hard_max_tokens: u32,
    #[serde(default = "default_true")]
    pub prefer_summaries_over_raw_files: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            default_max_tokens: 3000,
            hard_max_tokens: 8000,
            prefer_summaries_over_raw_files: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryConfig {
    #[serde(default = "default_true")]
    pub update_after_each_task: bool,
    #[serde(default = "default_true")]
    pub write_obsidian: bool,
    #[serde(default = "default_true")]
    pub write_graph: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            update_after_each_task: true,
            write_obsidian: true,
            write_graph: true,
        }
    }
}

fn default_execution_mode() -> String {
    "manual".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "qwen3.5:4b".to_string()
}

fn default_ollama_role() -> String {
    "context_and_memory".to_string()
}

fn default_claude_provider() -> String {
    "claude_code".to_string()
}

fn default_claude_model() -> String {
    "opus-4.7".to_string()
}

fn default_claude_role() -> String {
    "planning".to_string()
}

fn default_codex_provider() -> String {
    "codex".to_string()
}

fn default_codex_model() -> String {
    "gpt-5.5".to_string()
}

fn default_codex_role() -> String {
    "scoped_execution".to_string()
}

fn default_cursor_composer_model() -> String {
    std::env::var("CURSOR_COMPOSER_MODEL_ID").unwrap_or_else(|_| "composer-2.5".to_string())
}

fn default_cursor_premium_model() -> String {
    std::env::var("CURSOR_PREMIUM_MODEL_ID").unwrap_or_else(|_| "cursor-premium".to_string())
}

fn default_cursor_provider() -> String {
    "cursor_sdk".to_string()
}

fn default_cursor_runtime() -> String {
    "local".to_string()
}

fn default_repo_executor() -> String {
    "cursor_composer".to_string()
}

fn default_scoped_executor() -> String {
    "codex".to_string()
}

fn default_planner() -> String {
    "claude".to_string()
}

fn default_summarizer() -> String {
    "ollama".to_string()
}

fn default_max_failures() -> u32 {
    2
}

fn default_max_tokens() -> u32 {
    3000
}

fn default_hard_max_tokens() -> u32 {
    8000
}
