use crate::config::schema::{Config, ModelsConfig};
use crate::providers::ProviderKind;
use crate::router::decision::RoutingDecision;
use crate::tasks::{Task, TaskComplexity, TaskType};

pub fn route(task: &Task, config: &Config) -> RoutingDecision {
    let models = &config.models;

    // Escalation rule: if failed twice, require escalation review
    if task.failure_count >= config.routing.max_failures_before_escalation {
        return escalation_decision(task, models);
    }

    // Task type based routing
    match task.task_type {
        TaskType::Summary | TaskType::Compression | TaskType::MemoryUpdate | TaskType::Docs => {
            ollama_decision(models)
        }

        TaskType::Architecture
        | TaskType::Decomposition
        | TaskType::RiskAnalysis
        | TaskType::Planning => claude_decision(task, models),

        TaskType::Implementation | TaskType::Tests | TaskType::Refactor | TaskType::Review => {
            if task.context_is_exact && task.estimated_files_touched <= 2 {
                codex_decision(task, models)
            } else if task.requires_repo_search || task.estimated_files_touched >= 3 {
                cursor_composer_decision(task, models)
            } else {
                codex_decision(task, models)
            }
        }

        TaskType::Debugging => {
            if task.risk == crate::tasks::TaskRisk::High
                || task.complexity == TaskComplexity::Critical
            {
                cursor_premium_decision(task, models, config)
            } else {
                cursor_composer_decision(task, models)
            }
        }

        TaskType::Research => ollama_decision(models),
    }
}

fn ollama_decision(models: &ModelsConfig) -> RoutingDecision {
    RoutingDecision::simple(
        ProviderKind::Ollama,
        models.ollama.default_model.clone(),
        "Ollama is the cheapest option for summaries, compression, memory updates, and docs.",
    )
}

fn claude_decision(task: &Task, models: &ModelsConfig) -> RoutingDecision {
    let requires_approval = task.complexity == TaskComplexity::Critical
        || matches!(
            task.task_type,
            TaskType::Architecture | TaskType::RiskAnalysis
        );

    RoutingDecision {
        provider: ProviderKind::Anthropic,
        model: models.claude.default_model.clone(),
        reason: "Claude Opus is best for architecture, planning, decomposition, and risk analysis."
            .to_string(),
        requires_approval,
        risk: if task.complexity == TaskComplexity::Critical {
            "critical".to_string()
        } else {
            "high".to_string()
        },
        estimated_cost_class: "premium".to_string(),
        fallback_provider: ProviderKind::OpenAi,
        notes: vec!["Claude requires large context for planning tasks.".to_string()],
    }
}

fn codex_decision(task: &Task, models: &ModelsConfig) -> RoutingDecision {
    RoutingDecision {
        provider: ProviderKind::OpenAi,
        model: models.codex.default_model.clone(),
        reason:
            "Codex is best for scoped implementation where exact files and instructions are known."
                .to_string(),
        requires_approval: false,
        risk: match task.risk {
            crate::tasks::TaskRisk::Low => "low".to_string(),
            crate::tasks::TaskRisk::Medium => "medium".to_string(),
            crate::tasks::TaskRisk::High => "high".to_string(),
        },
        estimated_cost_class: "standard".to_string(),
        fallback_provider: ProviderKind::Cursor,
        notes: vec!["Codex works best with exact context packets.".to_string()],
    }
}

fn cursor_composer_decision(task: &Task, models: &ModelsConfig) -> RoutingDecision {
    RoutingDecision {
        provider: ProviderKind::Cursor,
        model: models.cursor.composer_model_id.clone(),
        reason: "Cursor Composer is the default repo-aware coding executor.".to_string(),
        requires_approval: false,
        risk: match task.risk {
            crate::tasks::TaskRisk::Low => "low".to_string(),
            crate::tasks::TaskRisk::Medium => "medium".to_string(),
            crate::tasks::TaskRisk::High => "high".to_string(),
        },
        estimated_cost_class: "standard".to_string(),
        fallback_provider: ProviderKind::Anthropic,
        notes: vec!["Cursor Composer has abundant usage and supports repo search.".to_string()],
    }
}

fn cursor_premium_decision(
    _task: &Task,
    models: &ModelsConfig,
    config: &Config,
) -> RoutingDecision {
    RoutingDecision {
        provider: ProviderKind::Cursor,
        model: models.cursor.premium_model_id.clone(),
        reason: "Cursor premium is used for high-risk tasks or after cheaper options failed."
            .to_string(),
        requires_approval: config.execution.require_approval_for_premium,
        risk: "critical".to_string(),
        estimated_cost_class: "premium".to_string(),
        fallback_provider: ProviderKind::Anthropic,
        notes: vec!["Premium models require manual approval.".to_string()],
    }
}

fn escalation_decision(task: &Task, models: &ModelsConfig) -> RoutingDecision {
    RoutingDecision {
        provider: ProviderKind::Cursor,
        model: models.cursor.premium_model_id.clone(),
        reason: format!(
            "Task failed {} times. Escalating to premium model for diagnosis.",
            task.failure_count
        ),
        requires_approval: true,
        risk: "critical".to_string(),
        estimated_cost_class: "premium".to_string(),
        fallback_provider: ProviderKind::Anthropic,
        notes: vec![
            "Previous attempts may have lacked context or been routed incorrectly.".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{
        Config, ContextConfig, ExecutionConfig, MemoryConfig, ModelsConfig, ProjectConfig,
        RoutingConfig,
    };
    use crate::providers::ProviderKind;
    use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskType};
    use std::path::PathBuf;

    fn test_config() -> Config {
        Config {
            project: ProjectConfig {
                name: "Test".to_string(),
                repo_path: PathBuf::from("."),
                vault_path: None,
                orca_dir: PathBuf::from(".orca"),
            },
            execution: ExecutionConfig::default(),
            models: ModelsConfig::default(),
            routing: RoutingConfig::default(),
            context: ContextConfig::default(),
            memory: MemoryConfig::default(),
        }
    }

    fn test_task(task_type: TaskType) -> Task {
        Task {
            id: "TASK-001".to_string(),
            title: "Test Task".to_string(),
            description: "".to_string(),
            task_type,
            complexity: TaskComplexity::Medium,
            risk: TaskRisk::Low,
            requires_repo_search: false,
            estimated_files_touched: 1,
            context_is_exact: true,
            failure_count: 0,
        }
    }

    #[test]
    fn test_route_summary_to_ollama() {
        let config = test_config();
        let task = test_task(TaskType::Summary);
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Ollama);
    }

    #[test]
    fn test_route_architecture_to_claude() {
        let config = test_config();
        let task = test_task(TaskType::Architecture);
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Anthropic);
    }

    #[test]
    fn test_route_repo_aware_to_cursor() {
        let config = test_config();
        let mut task = test_task(TaskType::Implementation);
        task.requires_repo_search = true;
        task.estimated_files_touched = 4;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Cursor);
        assert_eq!(decision.model, config.models.cursor.composer_model_id);
    }

    #[test]
    fn test_route_scoped_exact_to_codex() {
        let config = test_config();
        let mut task = test_task(TaskType::Implementation);
        task.context_is_exact = true;
        task.estimated_files_touched = 1;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::OpenAi);
    }

    #[test]
    fn test_route_failure_escalation() {
        let config = test_config();
        let mut task = test_task(TaskType::Implementation);
        task.failure_count = 2;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Cursor);
        assert!(decision.requires_approval);
    }

    #[test]
    fn test_route_high_risk_debugging_to_premium() {
        let config = test_config();
        let mut task = test_task(TaskType::Debugging);
        task.risk = TaskRisk::High;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Cursor);
        assert!(decision.requires_approval);
    }
}
