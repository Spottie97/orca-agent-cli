use crate::providers::ProviderKind;
use crate::router::decision::RoutingDecision;
use crate::tasks::{Task, TaskRisk};

#[derive(Debug, Clone)]
pub struct ApprovalConfig {
    pub require_approval_for_premium: bool,
    pub require_approval_for_destructive: bool,
    pub require_approval_for_high_risk: bool,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            require_approval_for_premium: true,
            require_approval_for_destructive: true,
            require_approval_for_high_risk: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ApprovalCheck {
    Pass,
    Block(String),
}

pub fn check_approval(
    task: &Task,
    decision: &RoutingDecision,
    config: &ApprovalConfig,
    bypass: bool,
) -> ApprovalCheck {
    if bypass {
        return ApprovalCheck::Pass;
    }

    if config.require_approval_for_premium
        && decision.provider == ProviderKind::Cursor
        && decision.requires_approval
    {
        return ApprovalCheck::Block(
            "Premium Cursor provider requires manual approval. Use --yes to bypass.".to_string(),
        );
    }

    if config.require_approval_for_high_risk
        && (task.risk == TaskRisk::High
            || task.risk == TaskRisk::Medium
                && task.complexity == crate::tasks::TaskComplexity::Critical)
    {
        return ApprovalCheck::Block(
            "High-risk task requires manual approval. Use --yes to bypass.".to_string(),
        );
    }

    if config.require_approval_for_destructive {
        // In MVP, treat refactor/review with many files as potentially destructive
        if matches!(
            task.task_type,
            crate::tasks::TaskType::Refactor | crate::tasks::TaskType::Review
        ) && task.estimated_files_touched >= 5
        {
            return ApprovalCheck::Block(
                "Potentially destructive operation requires manual approval. Use --yes to bypass."
                    .to_string(),
            );
        }
    }

    ApprovalCheck::Pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::decision::RoutingDecision;
    use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskType};

    fn test_config() -> ApprovalConfig {
        ApprovalConfig::default()
    }

    fn test_task(task_type: TaskType) -> Task {
        Task {
            id: "TASK-001".to_string(),
            title: "Test Task".to_string(),
            description: "".to_string(),
            task_type,
            complexity: TaskComplexity::Medium,
            risk: TaskRisk::Low,
            status: crate::tasks::TaskStatus::Pending,
            requires_repo_search: false,
            estimated_files_touched: 1,
            context_is_exact: true,
            failure_count: 0,
            acceptance_criteria: Vec::new(),
        }
    }

    fn premium_decision() -> RoutingDecision {
        RoutingDecision {
            provider: ProviderKind::Cursor,
            model: "cursor-premium".to_string(),
            reason: "Premium".to_string(),
            requires_approval: true,
            risk: "critical".to_string(),
            estimated_cost_class: "premium".to_string(),
            fallback_provider: ProviderKind::Anthropic,
            notes: Vec::new(),
        }
    }

    fn standard_decision() -> RoutingDecision {
        RoutingDecision {
            provider: ProviderKind::Ollama,
            model: "qwen".to_string(),
            reason: "Cheap".to_string(),
            requires_approval: false,
            risk: "low".to_string(),
            estimated_cost_class: "cheap".to_string(),
            fallback_provider: ProviderKind::Mock,
            notes: Vec::new(),
        }
    }

    #[test]
    fn test_premium_cursor_blocked() {
        let task = test_task(TaskType::Implementation);
        let decision = premium_decision();
        let result = check_approval(&task, &decision, &test_config(), false);
        assert!(matches!(result, ApprovalCheck::Block(ref msg) if msg.contains("Premium Cursor")));
    }

    #[test]
    fn test_premium_cursor_bypassed_with_yes() {
        let task = test_task(TaskType::Implementation);
        let decision = premium_decision();
        let result = check_approval(&task, &decision, &test_config(), true);
        assert!(matches!(result, ApprovalCheck::Pass));
    }

    #[test]
    fn test_standard_decision_passes() {
        let task = test_task(TaskType::Summary);
        let decision = standard_decision();
        let result = check_approval(&task, &decision, &test_config(), false);
        assert!(matches!(result, ApprovalCheck::Pass));
    }

    #[test]
    fn test_high_risk_blocked() {
        let mut task = test_task(TaskType::Debugging);
        task.risk = TaskRisk::High;
        let decision = standard_decision();
        let result = check_approval(&task, &decision, &test_config(), false);
        assert!(matches!(result, ApprovalCheck::Block(ref msg) if msg.contains("High-risk")));
    }

    #[test]
    fn test_destructive_blocked() {
        let mut task = test_task(TaskType::Refactor);
        task.estimated_files_touched = 10;
        let decision = standard_decision();
        let result = check_approval(&task, &decision, &test_config(), false);
        assert!(matches!(result, ApprovalCheck::Block(ref msg) if msg.contains("destructive")));
    }
}
