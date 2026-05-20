use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::providers::traits::ProviderResponse;
use crate::utils::fs::safe_write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub original: Option<String>,
    pub proposed: String,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    pub task_id: String,
    pub provider: String,
    pub model: String,
    pub timestamp: String,
    pub files_changed: Vec<FileChange>,
    pub summary: Option<String>,
}

impl PatchProposal {
    pub fn from_response(response: &ProviderResponse) -> Option<Self> {
        if response.files_changed.is_empty() {
            return None;
        }

        let files_changed = response
            .files_changed
            .iter()
            .map(|path| FileChange {
                path: path.clone(),
                original: None,
                proposed: String::new(),
                explanation: None,
            })
            .collect();

        Some(Self {
            task_id: response.task_id.clone(),
            provider: response.provider.to_string(),
            model: response.model_id.clone(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            files_changed,
            summary: response.suggested_memory_update.clone(),
        })
    }
}

/// Save a patch proposal artifact under the given orca directory.
/// Uses a deterministic filename (task_id.json) in the patches/ subdirectory.
/// Overwrites any existing patch for the same task.
pub fn save_patch_proposal(
    orca_dir: &Path,
    proposal: &PatchProposal,
) -> Result<std::path::PathBuf> {
    let patches_dir = orca_dir.join("patches");
    let path = patches_dir.join(format!("{}.json", proposal.task_id));
    let json = serde_json::to_string_pretty(proposal)?;
    safe_write(&path, &json)?;
    Ok(path)
}

/// Load a patch proposal for the given task ID.
pub fn load_patch_proposal(orca_dir: &Path, task_id: &str) -> Result<PatchProposal> {
    let path = orca_dir.join("patches").join(format!("{}.json", task_id));
    let contents = std::fs::read_to_string(&path)?;
    let proposal = serde_json::from_str(&contents)?;
    Ok(proposal)
}

/// List all patch proposal task IDs.
pub fn list_patch_proposals(orca_dir: &Path) -> Result<Vec<String>> {
    let patches_dir = orca_dir.join("patches");
    if !patches_dir.exists() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&patches_dir)? {
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
    fn test_patch_proposal_from_response_with_files() {
        let response = ProviderResponse {
            task_id: "TASK-001".to_string(),
            provider: ProviderKind::OpenAi,
            model_id: "llama3".to_string(),
            output: "Changed files".to_string(),
            status: ExecutionStatus::Success,
            files_changed: vec!["src/main.rs".to_string()],
            suggested_memory_update: Some("Update memory".to_string()),
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        };

        let proposal = PatchProposal::from_response(&response).unwrap();
        assert_eq!(proposal.task_id, "TASK-001");
        assert_eq!(proposal.files_changed.len(), 1);
        assert_eq!(proposal.files_changed[0].path, "src/main.rs");
        assert_eq!(proposal.summary, Some("Update memory".to_string()));
    }

    #[test]
    fn test_patch_proposal_none_when_no_files_changed() {
        let response = ProviderResponse {
            task_id: "TASK-001".to_string(),
            provider: ProviderKind::OpenAi,
            model_id: "llama3".to_string(),
            output: "No changes".to_string(),
            status: ExecutionStatus::Success,
            files_changed: Vec::new(),
            suggested_memory_update: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        };

        assert!(PatchProposal::from_response(&response).is_none());
    }

    #[test]
    fn test_save_and_load_patch_proposal() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        std::fs::create_dir_all(&orca_dir).unwrap();

        let proposal = PatchProposal {
            task_id: "TASK-001".to_string(),
            provider: "openai".to_string(),
            model: "llama3".to_string(),
            timestamp: "now".to_string(),
            files_changed: vec![FileChange {
                path: "src/main.rs".to_string(),
                original: Some("old".to_string()),
                proposed: "new".to_string(),
                explanation: Some("fix bug".to_string()),
            }],
            summary: Some("summary".to_string()),
        };

        let path = save_patch_proposal(&orca_dir, &proposal).unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "TASK-001.json");

        let loaded = load_patch_proposal(&orca_dir, "TASK-001").unwrap();
        assert_eq!(loaded.task_id, "TASK-001");
        assert_eq!(loaded.files_changed.len(), 1);
        assert_eq!(loaded.files_changed[0].path, "src/main.rs");
        assert_eq!(loaded.files_changed[0].original, Some("old".to_string()));
        assert_eq!(loaded.files_changed[0].proposed, "new".to_string());
    }

    #[test]
    fn test_list_patch_proposals() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        std::fs::create_dir_all(&orca_dir).unwrap();

        let p1 = PatchProposal {
            task_id: "TASK-A".to_string(),
            provider: "openai".to_string(),
            model: "llama3".to_string(),
            timestamp: "now".to_string(),
            files_changed: vec![],
            summary: None,
        };
        let p2 = PatchProposal {
            task_id: "TASK-B".to_string(),
            provider: "openai".to_string(),
            model: "llama3".to_string(),
            timestamp: "now".to_string(),
            files_changed: vec![],
            summary: None,
        };

        save_patch_proposal(&orca_dir, &p1).unwrap();
        save_patch_proposal(&orca_dir, &p2).unwrap();

        let ids = list_patch_proposals(&orca_dir).unwrap();
        assert_eq!(ids, vec!["TASK-A", "TASK-B"]);
    }

    #[test]
    fn test_list_patch_proposals_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        std::fs::create_dir_all(&orca_dir).unwrap();

        let ids = list_patch_proposals(&orca_dir).unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn test_list_patch_proposals_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        // Do not create patches subdir

        let ids = list_patch_proposals(&orca_dir).unwrap();
        assert!(ids.is_empty());
    }
}
