use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalBridgeModelConfig {
    #[serde(default)]
    pub enabled: bool,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_stdin_true")]
    pub stdin: bool,
    #[serde(default = "default_timeout_600")]
    pub timeout_seconds: u64,
    #[serde(default = "default_approval_true")]
    pub requires_approval: bool,
    pub model: Option<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_max_output_100000")]
    pub max_output_bytes: usize,
}

impl Default for ExternalBridgeModelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            command: None,
            args: Vec::new(),
            stdin: default_stdin_true(),
            timeout_seconds: default_timeout_600(),
            requires_approval: default_approval_true(),
            model: None,
            working_directory: None,
            env: HashMap::new(),
            max_output_bytes: default_max_output_100000(),
        }
    }
}

fn default_stdin_true() -> bool {
    true
}

fn default_timeout_600() -> u64 {
    600
}

fn default_approval_true() -> bool {
    true
}

fn default_max_output_100000() -> usize {
    100_000
}
