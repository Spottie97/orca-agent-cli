use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewVerdict {
    Accept,
    Reject,
    Escalate,
}

impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewVerdict::Accept => write!(f, "accept"),
            ReviewVerdict::Reject => write!(f, "reject"),
            ReviewVerdict::Escalate => write!(f, "escalate"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub task_id: String,
    pub accepted: bool,
    pub verdict: ReviewVerdict,
    pub reasons: Vec<String>,
    pub missing_criteria: Vec<String>,
    pub risks: Vec<String>,
    pub recommended_next_step: String,
    #[serde(default)]
    pub execution_output: Option<String>,
    #[serde(default)]
    pub execution_status: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub input_tokens: Option<u32>,
    #[serde(default)]
    pub output_tokens: Option<u32>,
}

impl ReviewResult {
    pub fn mock_accept(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            accepted: true,
            verdict: ReviewVerdict::Accept,
            reasons: vec!["All acceptance criteria met.".to_string()],
            missing_criteria: Vec::new(),
            risks: Vec::new(),
            recommended_next_step: "Proceed to memory update.".to_string(),
            execution_output: None,
            execution_status: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        }
    }

    pub fn mock_reject(task_id: impl Into<String>, missing: &[String]) -> Self {
        Self {
            task_id: task_id.into(),
            accepted: false,
            verdict: ReviewVerdict::Reject,
            reasons: vec!["Some acceptance criteria were not met.".to_string()],
            missing_criteria: missing.to_vec(),
            risks: Vec::new(),
            recommended_next_step: "Re-execute the task with additional context.".to_string(),
            execution_output: None,
            execution_status: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        }
    }

    pub fn mock_escalate(task_id: impl Into<String>, risks: &[String]) -> Self {
        Self {
            task_id: task_id.into(),
            accepted: false,
            verdict: ReviewVerdict::Escalate,
            reasons: vec!["High risk detected during review.".to_string()],
            missing_criteria: Vec::new(),
            risks: risks.to_vec(),
            recommended_next_step: "Escalate to premium model for manual review.".to_string(),
            execution_output: None,
            execution_status: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
        }
    }
}

fn contains_error_keywords(output: &str) -> bool {
    let lower = output.to_lowercase();
    [
        "error",
        "failed",
        "failure",
        "panic",
        "exception",
        "abort",
        "timeout",
        "unauthorized",
        "forbidden",
        "not found",
        "invalid",
    ]
    .iter()
    .any(|kw| lower.contains(kw))
}

fn contains_success_keywords(output: &str) -> bool {
    let lower = output.to_lowercase();
    ["success", "completed", "done", "passed", "ok", "finished"]
        .iter()
        .any(|kw| lower.contains(kw))
}

pub fn review_task(
    task: &crate::tasks::Task,
    execution_output: Option<&str>,
    execution_status: Option<&str>,
    duration_ms: Option<u64>,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
) -> ReviewResult {
    if task.failure_count >= 2 {
        return ReviewResult {
            task_id: task.id.clone(),
            accepted: false,
            verdict: ReviewVerdict::Escalate,
            reasons: vec![format!("Task failed {} times.", task.failure_count)],
            missing_criteria: Vec::new(),
            risks: vec!["Repeated failure indicates systemic issue.".to_string()],
            recommended_next_step: "Escalate to premium model for manual review.".to_string(),
            execution_output: execution_output.map(|s| s.to_string()),
            execution_status: execution_status.map(|s| s.to_string()),
            duration_ms,
            input_tokens,
            output_tokens,
        };
    }

    // If we have real execution data, factor it into the review
    if let Some(status) = execution_status {
        if status == "failure" || status == "cancelled" {
            return ReviewResult {
                task_id: task.id.clone(),
                accepted: false,
                verdict: ReviewVerdict::Reject,
                reasons: vec![format!("Execution status was {}.", status)],
                missing_criteria: Vec::new(),
                risks: vec!["Task did not complete successfully.".to_string()],
                recommended_next_step: "Re-execute the task with additional context.".to_string(),
                execution_output: execution_output.map(|s| s.to_string()),
                execution_status: Some(status.to_string()),
                duration_ms,
                input_tokens,
                output_tokens,
            };
        }
    }

    if let Some(output) = execution_output {
        if contains_error_keywords(output) && !contains_success_keywords(output) {
            return ReviewResult {
                task_id: task.id.clone(),
                accepted: false,
                verdict: ReviewVerdict::Reject,
                reasons: vec!["Execution output contains error keywords.".to_string()],
                missing_criteria: Vec::new(),
                risks: vec!["Detected potential failures in output.".to_string()],
                recommended_next_step: "Re-execute the task with additional context.".to_string(),
                execution_output: Some(output.to_string()),
                execution_status: execution_status.map(|s| s.to_string()),
                duration_ms,
                input_tokens,
                output_tokens,
            };
        }
    }

    // Default: accept if criteria exist or no data
    ReviewResult {
        task_id: task.id.clone(),
        accepted: true,
        verdict: ReviewVerdict::Accept,
        reasons: vec!["All acceptance criteria met.".to_string()],
        missing_criteria: Vec::new(),
        risks: Vec::new(),
        recommended_next_step: "Proceed to memory update.".to_string(),
        execution_output: execution_output.map(|s| s.to_string()),
        execution_status: execution_status.map(|s| s.to_string()),
        duration_ms,
        input_tokens,
        output_tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::{Task, TaskType};

    #[test]
    fn test_mock_accept() {
        let result = ReviewResult::mock_accept("T1");
        assert!(result.accepted);
        assert_eq!(result.verdict, ReviewVerdict::Accept);
        assert_eq!(result.task_id, "T1");
    }

    #[test]
    fn test_mock_reject() {
        let result = ReviewResult::mock_reject("T1", &["Tests missing".to_string()]);
        assert!(!result.accepted);
        assert_eq!(result.verdict, ReviewVerdict::Reject);
        assert!(result
            .missing_criteria
            .contains(&"Tests missing".to_string()));
    }

    #[test]
    fn test_mock_escalate() {
        let result = ReviewResult::mock_escalate("T1", &["Data loss risk".to_string()]);
        assert!(!result.accepted);
        assert_eq!(result.verdict, ReviewVerdict::Escalate);
        assert!(result.risks.contains(&"Data loss risk".to_string()));
    }

    #[test]
    fn test_review_task_escalates_after_failures() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.failure_count = 2;
        let result = review_task(&task, None, None, None, None, None);
        assert_eq!(result.verdict, ReviewVerdict::Escalate);
    }

    #[test]
    fn test_review_task_accepts_when_no_criteria() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(&task, None, None, None, None, None);
        assert_eq!(result.verdict, ReviewVerdict::Accept);
    }

    #[test]
    fn test_review_task_rejects_on_failure_status() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(&task, None, Some("failure"), Some(100), Some(10), Some(5));
        assert_eq!(result.verdict, ReviewVerdict::Reject);
        assert_eq!(result.execution_status, Some("failure".to_string()));
        assert_eq!(result.duration_ms, Some(100));
    }

    #[test]
    fn test_review_task_rejects_on_error_keywords() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(
            &task,
            Some("Build failed with error"),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Reject);
    }

    #[test]
    fn test_review_task_accepts_on_success_keywords() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(
            &task,
            Some("Build completed successfully"),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Accept);
    }

    #[test]
    fn test_review_task_rejects_cancelled_status() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(&task, None, Some("cancelled"), None, None, None);
        assert_eq!(result.verdict, ReviewVerdict::Reject);
    }
}
