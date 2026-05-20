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
    pub unmet_acceptance_criteria: Vec<String>,
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
            unmet_acceptance_criteria: Vec::new(),
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
            unmet_acceptance_criteria: missing.to_vec(),
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
            unmet_acceptance_criteria: Vec::new(),
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

/// Check if the output contains refusal or insufficiency patterns.
fn contains_refusal_patterns(output: &str) -> bool {
    let lower = output.to_lowercase();
    [
        "i don't have access",
        "i do not have access",
        "please provide details",
        "i need more context",
        "i need more information",
        "i cannot complete",
        "i can't complete",
        "i am unable to",
        "i'm unable to",
        "insufficient context",
        "not enough information",
        "i don't know",
        "i do not know",
        "i don't have enough context",
    ]
    .iter()
    .any(|kw| lower.contains(kw))
}

/// Check whether the output appears to satisfy the acceptance criteria.
/// This is a heuristic: for each criterion, check if key phrases from it appear in the output.
fn check_acceptance_criteria(output: &str, criteria: &[String]) -> Vec<String> {
    let lower_output = output.to_lowercase();
    let mut unmet = Vec::new();
    for criterion in criteria {
        // Extract key words (skip common stop words)
        let key_phrase = criterion.to_lowercase();
        // Simple heuristic: if the criterion text is not found verbatim or partially,
        // treat it as potentially unmet. For stricter checking, we look for at least
        // one distinctive word from the criterion.
        let words: Vec<&str> = key_phrase
            .split_whitespace()
            .filter(|w| {
                !matches!(
                    *w,
                    "the"
                        | "a"
                        | "an"
                        | "is"
                        | "are"
                        | "must"
                        | "should"
                        | "to"
                        | "be"
                        | "and"
                        | "or"
                        | "of"
                        | "in"
                        | "on"
                        | "with"
                        | "for"
                        | "it"
                )
            })
            .collect();
        let matched = words.iter().any(|w| lower_output.contains(w));
        if !matched {
            unmet.push(criterion.clone());
        }
    }
    unmet
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
            unmet_acceptance_criteria: Vec::new(),
            risks: vec!["Repeated failure indicates systemic issue.".to_string()],
            recommended_next_step: "Escalate to premium model for manual review.".to_string(),
            execution_output: execution_output.map(|s| s.to_string()),
            execution_status: execution_status.map(|s| s.to_string()),
            duration_ms,
            input_tokens,
            output_tokens,
        };
    }

    // Refusal / insufficient context detection
    if let Some(output) = execution_output {
        if contains_refusal_patterns(output) {
            let mut reasons = vec![
                "Execution output indicates missing context or inability to complete.".to_string(),
            ];
            let unmet = check_acceptance_criteria(output, &task.acceptance_criteria);
            if !unmet.is_empty() {
                reasons.push(format!("Unmet acceptance criteria: {}.", unmet.join(", ")));
            }
            return ReviewResult {
                task_id: task.id.clone(),
                accepted: false,
                verdict: ReviewVerdict::Reject,
                reasons,
                missing_criteria: unmet.clone(),
                unmet_acceptance_criteria: unmet,
                risks: vec![
                    "Provider could not complete the task with supplied context.".to_string(),
                ],
                recommended_next_step: "Re-execute the task with additional context.".to_string(),
                execution_output: Some(output.to_string()),
                execution_status: execution_status.map(|s| s.to_string()),
                duration_ms,
                input_tokens,
                output_tokens,
            };
        }
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
                unmet_acceptance_criteria: Vec::new(),
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

    // Check acceptance criteria even on success
    let unmet_criteria = if let Some(output) = execution_output {
        check_acceptance_criteria(output, &task.acceptance_criteria)
    } else {
        Vec::new()
    };

    if !unmet_criteria.is_empty() {
        return ReviewResult {
            task_id: task.id.clone(),
            accepted: false,
            verdict: ReviewVerdict::Reject,
            reasons: vec![format!(
                "Execution succeeded but did not satisfy all acceptance criteria: {}.",
                unmet_criteria.join(", ")
            )],
            missing_criteria: unmet_criteria.clone(),
            unmet_acceptance_criteria: unmet_criteria.clone(),
            risks: vec!["Output may be incomplete or off-target.".to_string()],
            recommended_next_step: "Re-execute the task with clearer instructions.".to_string(),
            execution_output: execution_output.map(|s| s.to_string()),
            execution_status: execution_status.map(|s| s.to_string()),
            duration_ms,
            input_tokens,
            output_tokens,
        };
    }

    if let Some(output) = execution_output {
        if contains_error_keywords(output) && !contains_success_keywords(output) {
            return ReviewResult {
                task_id: task.id.clone(),
                accepted: false,
                verdict: ReviewVerdict::Reject,
                reasons: vec!["Execution output contains error keywords.".to_string()],
                missing_criteria: Vec::new(),
                unmet_acceptance_criteria: Vec::new(),
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
        unmet_acceptance_criteria: Vec::new(),
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

    #[test]
    fn test_review_rejects_refusal_patterns() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(
            &task,
            Some("I don't have access to your task management system"),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Reject);
        assert!(!result.accepted);
        assert!(result.reasons.iter().any(|r| r.contains("missing context")));
    }

    #[test]
    fn test_review_rejects_need_more_context() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(
            &task,
            Some("I need more context to complete this task."),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Reject);
        assert!(!result.accepted);
    }

    #[test]
    fn test_review_rejects_cannot_complete() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(
            &task,
            Some("I cannot complete this request without additional details."),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Reject);
        assert!(!result.accepted);
    }

    #[test]
    fn test_review_does_not_accept_solely_on_success_status() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.acceptance_criteria
            .push("Output must contain ORCA_OK".to_string());
        let result = review_task(
            &task,
            Some("Some generic success message without the required phrase."),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Reject);
        assert!(!result.accepted);
        assert!(!result.unmet_acceptance_criteria.is_empty());
    }

    #[test]
    fn test_review_accepts_when_criteria_explicitly_satisfied() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.acceptance_criteria
            .push("Output must contain ORCA_OK".to_string());
        let result = review_task(
            &task,
            Some("The result is ORCA_OK and all checks pass."),
            Some("success"),
            None,
            None,
            None,
        );
        assert_eq!(result.verdict, ReviewVerdict::Accept);
        assert!(result.accepted);
        assert!(result.unmet_acceptance_criteria.is_empty());
    }
}
