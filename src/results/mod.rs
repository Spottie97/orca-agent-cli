use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::providers::traits::ProviderResponse;
use crate::utils::fs::safe_write;
use crate::utils::redact::redact_secrets;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub task_id: String,
    pub provider: String,
    pub model: String,
    pub timestamp: String,
    pub status: String,
    pub output: String,
    pub duration_ms: Option<u64>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    #[serde(default)]
    pub context_packet_path: Option<String>,
    #[serde(default)]
    pub prompt_included_context: bool,
    #[serde(default)]
    pub prompt_sections_included: Vec<String>,
}

impl ExecutionResult {
    pub fn from_response(response: &ProviderResponse) -> Self {
        let sanitized_output = redact_secrets(&response.output);
        // Size-limit stored output to avoid giant artifacts
        let max_output_len = 100_000;
        let output = if sanitized_output.len() > max_output_len {
            format!(
                "{}\n\n[truncated from {} chars to {}]",
                &sanitized_output[..max_output_len],
                sanitized_output.len(),
                max_output_len
            )
        } else {
            sanitized_output
        };

        Self {
            task_id: response.task_id.clone(),
            provider: response.provider.to_string(),
            model: response.model_id.clone(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            status: response.status.to_string(),
            output,
            duration_ms: response.duration_ms,
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
            context_packet_path: response.context_packet_path.clone(),
            prompt_included_context: response.prompt_included_context,
            prompt_sections_included: response.prompt_sections_included.clone(),
        }
    }
}

/// Save an execution result artifact under the given orca directory.
/// Uses a deterministic filename (task_id.json) in the results/ subdirectory.
/// Overwrites any existing result for the same task.
pub fn save_result(orca_dir: &Path, response: &ProviderResponse) -> Result<std::path::PathBuf> {
    let results_dir = orca_dir.join("results");
    let path = results_dir.join(format!("{}.json", response.task_id));
    let artifact = ExecutionResult::from_response(response);
    let json = serde_json::to_string_pretty(&artifact)?;
    safe_write(&path, &json)?;
    Ok(path)
}

/// Load an execution result for the given task ID.
pub fn load_result(orca_dir: &Path, task_id: &str) -> Result<ExecutionResult> {
    let path = orca_dir.join("results").join(format!("{}.json", task_id));
    let contents = std::fs::read_to_string(&path)?;
    let artifact = serde_json::from_str(&contents)?;
    Ok(artifact)
}

/// List all execution result task IDs.
pub fn list_results(orca_dir: &Path) -> Result<Vec<String>> {
    let results_dir = orca_dir.join("results");
    if !results_dir.exists() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&results_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "json" {
                if let Some(stem) = path.file_stem() {
                    ids.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::traits::{ExecutionStatus, ProviderKind};

    #[test]
    fn test_execution_result_from_response() {
        let response = ProviderResponse {
            task_id: "TASK-001".to_string(),
            provider: ProviderKind::OpenAi,
            model_id: "llama3".to_string(),
            output: "Hello world".to_string(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: Some(123),
            input_tokens: Some(10),
            output_tokens: Some(5),
            context_packet_path: None,
            prompt_included_context: true,
            prompt_sections_included: vec!["task_metadata".to_string()],
        };

        let result = ExecutionResult::from_response(&response);
        assert_eq!(result.task_id, "TASK-001");
        assert_eq!(result.provider, "openai");
        assert_eq!(result.model, "llama3");
        assert_eq!(result.status, "success");
        assert_eq!(result.duration_ms, Some(123));
        assert_eq!(result.input_tokens, Some(10));
        assert_eq!(result.output_tokens, Some(5));
    }

    #[test]
    fn test_execution_result_redacts_secrets() {
        let response = ProviderResponse {
            task_id: "TASK-001".to_string(),
            provider: ProviderKind::OpenAi,
            model_id: "llama3".to_string(),
            output: "API key is sk-abc123def456".to_string(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
            context_packet_path: None,
            prompt_included_context: false,
            prompt_sections_included: Vec::new(),
        };

        let result = ExecutionResult::from_response(&response);
        assert!(!result.output.contains("sk-abc123def456"));
        assert!(result.output.contains("[REDACTED]"));
    }

    #[test]
    fn test_execution_result_truncates_large_output() {
        let large_output = "x".repeat(200_000);
        let response = ProviderResponse {
            task_id: "TASK-001".to_string(),
            provider: ProviderKind::OpenAi,
            model_id: "llama3".to_string(),
            output: large_output.clone(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
            context_packet_path: None,
            prompt_included_context: false,
            prompt_sections_included: Vec::new(),
        };

        let result = ExecutionResult::from_response(&response);
        assert!(result.output.len() < large_output.len());
        assert!(result.output.contains("[truncated"));
    }

    #[test]
    fn test_save_result_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        std::fs::create_dir_all(&orca_dir).unwrap();

        let response = ProviderResponse {
            task_id: "TASK-001".to_string(),
            provider: ProviderKind::OpenAi,
            model_id: "llama3".to_string(),
            output: "test output".to_string(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: Some(100),
            input_tokens: Some(10),
            output_tokens: Some(5),
            context_packet_path: None,
            prompt_included_context: true,
            prompt_sections_included: vec![
                "task_metadata".to_string(),
                "execution_instructions".to_string(),
            ],
        };

        let path = save_result(&orca_dir, &response).unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "TASK-001.json");

        let contents = std::fs::read_to_string(&path).unwrap();
        let artifact: ExecutionResult = serde_json::from_str(&contents).unwrap();
        assert_eq!(artifact.task_id, "TASK-001");
        assert_eq!(artifact.duration_ms, Some(100));
    }
}
