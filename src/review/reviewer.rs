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
        }
    }
}

pub fn review_task(task: &crate::tasks::Task) -> ReviewResult {
    if task.failure_count >= 2 {
        return ReviewResult::mock_escalate(
            &task.id,
            &[format!("Task failed {} times.", task.failure_count)],
        );
    }

    if task.acceptance_criteria.is_empty() {
        return ReviewResult::mock_accept(&task.id);
    }

    // In a real implementation, this would analyze the actual task output.
    // For MVP, we deterministically accept if criteria exist.
    ReviewResult::mock_accept(&task.id)
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
        let result = review_task(&task);
        assert_eq!(result.verdict, ReviewVerdict::Escalate);
    }

    #[test]
    fn test_review_task_accepts_when_no_criteria() {
        let task = Task::new("T1", "Test", TaskType::Implementation);
        let result = review_task(&task);
        assert_eq!(result.verdict, ReviewVerdict::Accept);
    }
}
