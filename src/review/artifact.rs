use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::review::reviewer::ReviewResult;
use crate::utils::fs::safe_write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewArtifact {
    pub task_id: String,
    pub verdict: String,
    pub accepted: bool,
    pub reasons: Vec<String>,
    pub missing_criteria: Vec<String>,
    pub risks: Vec<String>,
    pub recommended_next_step: String,
    pub execution_output_summary: Option<String>,
    pub execution_status: Option<String>,
    pub duration_ms: Option<u64>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub timestamp: String,
}

impl ReviewArtifact {
    pub fn from_review(result: &ReviewResult) -> Self {
        let output_summary = result.execution_output.as_ref().map(|o| {
            let max_len = 500;
            if o.len() > max_len {
                format!("{}{}", &o[..max_len], "...")
            } else {
                o.clone()
            }
        });

        Self {
            task_id: result.task_id.clone(),
            verdict: result.verdict.to_string(),
            accepted: result.accepted,
            reasons: result.reasons.clone(),
            missing_criteria: result.missing_criteria.clone(),
            risks: result.risks.clone(),
            recommended_next_step: result.recommended_next_step.clone(),
            execution_output_summary: output_summary,
            execution_status: result.execution_status.clone(),
            duration_ms: result.duration_ms,
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
            timestamp: format!("{:?}", std::time::SystemTime::now()),
        }
    }
}

/// Save a review artifact under the given orca directory.
pub fn save_review(orca_dir: &Path, result: &ReviewResult) -> Result<std::path::PathBuf> {
    let reviews_dir = orca_dir.join("reviews");
    let path = reviews_dir.join(format!("{}.json", result.task_id));
    let artifact = ReviewArtifact::from_review(result);
    let json = serde_json::to_string_pretty(&artifact)?;
    safe_write(&path, &json)?;
    Ok(path)
}

/// Load a review artifact for the given task ID.
pub fn load_review(orca_dir: &Path, task_id: &str) -> Result<ReviewArtifact> {
    let path = orca_dir.join("reviews").join(format!("{}.json", task_id));
    let contents = std::fs::read_to_string(&path)?;
    let artifact = serde_json::from_str(&contents)?;
    Ok(artifact)
}

/// List all review artifact task IDs.
pub fn list_reviews(orca_dir: &Path) -> Result<Vec<String>> {
    let reviews_dir = orca_dir.join("reviews");
    if !reviews_dir.exists() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&reviews_dir)? {
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
    use crate::review::reviewer::{ReviewResult, ReviewVerdict};

    #[test]
    fn test_review_artifact_from_review() {
        let result = ReviewResult {
            task_id: "TASK-001".to_string(),
            accepted: true,
            verdict: ReviewVerdict::Accept,
            reasons: vec!["All good".to_string()],
            missing_criteria: Vec::new(),
            risks: Vec::new(),
            recommended_next_step: "Proceed".to_string(),
            execution_output: Some("output".to_string()),
            execution_status: Some("success".to_string()),
            duration_ms: Some(100),
            input_tokens: Some(10),
            output_tokens: Some(5),
        };

        let artifact = ReviewArtifact::from_review(&result);
        assert_eq!(artifact.task_id, "TASK-001");
        assert!(artifact.accepted);
        assert_eq!(artifact.verdict, "accept");
        assert_eq!(artifact.execution_status, Some("success".to_string()));
        assert_eq!(artifact.duration_ms, Some(100));
    }

    #[test]
    fn test_save_and_load_review() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        std::fs::create_dir_all(&orca_dir).unwrap();

        let result = ReviewResult {
            task_id: "TASK-001".to_string(),
            accepted: true,
            verdict: ReviewVerdict::Accept,
            reasons: vec!["All good".to_string()],
            missing_criteria: Vec::new(),
            risks: Vec::new(),
            recommended_next_step: "Proceed".to_string(),
            execution_output: None,
            execution_status: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        };

        let path = save_review(&orca_dir, &result).unwrap();
        assert!(path.exists());

        let loaded = load_review(&orca_dir, "TASK-001").unwrap();
        assert_eq!(loaded.task_id, "TASK-001");
        assert!(loaded.accepted);
    }

    #[test]
    fn test_list_reviews() {
        let tmp = tempfile::tempdir().unwrap();
        let orca_dir = tmp.path().join(".orca");
        std::fs::create_dir_all(&orca_dir).unwrap();

        let r1 = ReviewResult {
            task_id: "TASK-A".to_string(),
            accepted: true,
            verdict: ReviewVerdict::Accept,
            reasons: vec![],
            missing_criteria: vec![],
            risks: vec![],
            recommended_next_step: "Next".to_string(),
            execution_output: None,
            execution_status: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        };
        let r2 = ReviewResult {
            task_id: "TASK-B".to_string(),
            accepted: false,
            verdict: ReviewVerdict::Reject,
            reasons: vec![],
            missing_criteria: vec![],
            risks: vec![],
            recommended_next_step: "Retry".to_string(),
            execution_output: None,
            execution_status: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        };

        save_review(&orca_dir, &r1).unwrap();
        save_review(&orca_dir, &r2).unwrap();

        let ids = list_reviews(&orca_dir).unwrap();
        assert_eq!(ids, vec!["TASK-A", "TASK-B"]);
    }
}
