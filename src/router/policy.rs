use crate::config::schema::{Config, ModelsConfig};
use crate::providers::ProviderKind;
use crate::router::decision::RoutingDecision;
use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskType};

pub fn route(task: &Task, config: &Config) -> RoutingDecision {
    let models = &config.models;

    // Escalation rule: if failed twice, require escalation review
    if task.failure_count >= config.routing.max_failures_before_escalation {
        return escalation_decision(task, models);
    }

    // Task type based routing
    match task.task_type {
        TaskType::Summary | TaskType::Compression | TaskType::MemoryUpdate | TaskType::Docs => {
            summary_decision(models)
        }

        TaskType::Architecture | TaskType::Decomposition | TaskType::RiskAnalysis => {
            architecture_decision(task, models)
        }

        TaskType::Planning => planning_decision(task, models, config),

        TaskType::Implementation | TaskType::Refactor => {
            implementation_decision(task, models, config)
        }

        TaskType::Tests => tests_decision(task, models, config),

        TaskType::Review => review_decision(task, models, config),

        TaskType::Debugging => debugging_decision(task, models, config),

        TaskType::Research => research_decision(models),
    }
}

fn provider_enabled(kind: ProviderKind, models: &ModelsConfig) -> bool {
    match kind {
        ProviderKind::Ollama => models.ollama.enabled,
        ProviderKind::Anthropic => models.claude.enabled,
        ProviderKind::OpenAi => models.codex.enabled,
        ProviderKind::Cursor => models.cursor.enabled,
        _ => true,
    }
}

fn provider_from_name(name: &str) -> ProviderKind {
    match name {
        "ollama" => ProviderKind::Ollama,
        "claude" | "anthropic" => ProviderKind::Anthropic,
        "codex" | "openai" => ProviderKind::OpenAi,
        "cursor" | "cursor_composer" => ProviderKind::Cursor,
        _ => ProviderKind::Manual,
    }
}

fn model_for_provider(kind: ProviderKind, models: &ModelsConfig) -> String {
    match kind {
        ProviderKind::Ollama => models.ollama.default_model.clone(),
        ProviderKind::Anthropic => models.claude.default_model.clone(),
        ProviderKind::OpenAi => models.codex.default_model.clone(),
        ProviderKind::Cursor => models.cursor.composer_model_id.clone(),
        ProviderKind::Manual => "manual".to_string(),
        _ => "mock".to_string(),
    }
}

fn manual_decision(reason: impl Into<String>) -> RoutingDecision {
    RoutingDecision {
        provider: ProviderKind::Manual,
        model: "manual".to_string(),
        reason: reason.into(),
        requires_approval: true,
        risk: "medium".to_string(),
        estimated_cost_class: "free".to_string(),
        fallback_provider: ProviderKind::Mock,
        notes: vec!["No automated provider is available; manual execution required.".to_string()],
    }
}

fn summary_decision(models: &ModelsConfig) -> RoutingDecision {
    RoutingDecision::simple(
        ProviderKind::Ollama,
        models.ollama.default_model.clone(),
        "Ollama is the cheapest option for summaries, compression, memory updates, and docs.",
    )
}

fn research_decision(models: &ModelsConfig) -> RoutingDecision {
    RoutingDecision::simple(
        ProviderKind::Ollama,
        models.ollama.default_model.clone(),
        "Ollama is the cheapest option for research and information gathering.",
    )
}

fn planning_decision(task: &Task, models: &ModelsConfig, config: &Config) -> RoutingDecision {
    let is_low = task.complexity == TaskComplexity::Low && task.risk == TaskRisk::Low;
    let is_high = task.complexity == TaskComplexity::High
        || task.complexity == TaskComplexity::Critical
        || task.risk == TaskRisk::High;

    if is_low {
        if provider_enabled(ProviderKind::Ollama, models) {
            return RoutingDecision::simple(
                ProviderKind::Ollama,
                models.ollama.default_model.clone(),
                "Low-risk planning tasks are routed to Ollama for cost efficiency.",
            );
        }
        return manual_decision("Low-risk planning task; no cheap provider enabled.");
    }

    if is_high {
        if provider_enabled(ProviderKind::Anthropic, models) {
            return RoutingDecision {
                provider: ProviderKind::Anthropic,
                model: models.claude.default_model.clone(),
                reason: "High-complexity or high-risk planning requires Claude for deep reasoning."
                    .to_string(),
                requires_approval: true,
                risk: "high".to_string(),
                estimated_cost_class: "premium".to_string(),
                fallback_provider: ProviderKind::Manual,
                notes: vec!["Claude provides the best reasoning for complex planning.".to_string()],
            };
        }
        return manual_decision("High-complexity planning task; no premium provider enabled.");
    }

    // Medium complexity: use default_planner from config
    let planner_name = config.routing.default_planner.as_str();
    let planner_kind = provider_from_name(planner_name);
    if planner_kind != ProviderKind::Manual && provider_enabled(planner_kind, models) {
        let model = model_for_provider(planner_kind, models);
        let reason = format!(
            "Medium-complexity planning routed to default planner ({})",
            planner_name
        );
        return RoutingDecision::simple(planner_kind, model, reason);
    }

    // Fallback: try ollama, then claude, then manual
    if provider_enabled(ProviderKind::Ollama, models) {
        return RoutingDecision::simple(
            ProviderKind::Ollama,
            models.ollama.default_model.clone(),
            "Medium-complexity planning routed to Ollama (default planner unavailable).",
        );
    }
    if provider_enabled(ProviderKind::Anthropic, models) {
        return RoutingDecision::simple(
            ProviderKind::Anthropic,
            models.claude.default_model.clone(),
            "Medium-complexity planning routed to Claude (default planner unavailable).",
        );
    }
    manual_decision("Medium-complexity planning task; no providers enabled.")
}

fn architecture_decision(task: &Task, models: &ModelsConfig) -> RoutingDecision {
    let requires_approval = task.complexity == TaskComplexity::Critical
        || matches!(
            task.task_type,
            TaskType::Architecture | TaskType::RiskAnalysis
        );

    RoutingDecision {
        provider: ProviderKind::Anthropic,
        model: models.claude.default_model.clone(),
        reason: "Claude Opus is best for architecture, decomposition, and risk analysis."
            .to_string(),
        requires_approval,
        risk: if task.complexity == TaskComplexity::Critical {
            "critical".to_string()
        } else {
            "high".to_string()
        },
        estimated_cost_class: "premium".to_string(),
        fallback_provider: ProviderKind::OpenAi,
        notes: vec!["Claude requires large context for architecture tasks.".to_string()],
    }
}

fn implementation_decision(task: &Task, models: &ModelsConfig, config: &Config) -> RoutingDecision {
    let repo_aware = task.requires_repo_search || task.estimated_files_touched >= 3;
    let config_name = if repo_aware {
        config.routing.default_repo_executor.as_str()
    } else {
        config.routing.default_scoped_executor.as_str()
    };
    let kind = provider_from_name(config_name);
    let model = model_for_provider(kind, models);

    let reason = if repo_aware {
        format!(
            "Implementation task with repo-wide scope routed to {}",
            config_name
        )
    } else {
        format!(
            "Scoped implementation task with exact context routed to {}",
            config_name
        )
    };

    RoutingDecision {
        provider: kind,
        model,
        reason,
        requires_approval: false,
        risk: match task.risk {
            TaskRisk::Low => "low".to_string(),
            TaskRisk::Medium => "medium".to_string(),
            TaskRisk::High => "high".to_string(),
        },
        estimated_cost_class: "standard".to_string(),
        fallback_provider: if kind == ProviderKind::Cursor {
            ProviderKind::Anthropic
        } else {
            ProviderKind::Cursor
        },
        notes: if repo_aware {
            vec![format!(
                "{} supports repo search and multi-file edits.",
                config_name
            )]
        } else {
            vec![format!(
                "{} works best with exact context packets.",
                config_name
            )]
        },
    }
}

fn tests_decision(task: &Task, models: &ModelsConfig, config: &Config) -> RoutingDecision {
    let repo_aware = task.requires_repo_search || task.estimated_files_touched >= 3;
    let config_name = if repo_aware {
        config.routing.default_repo_executor.as_str()
    } else {
        config.routing.default_scoped_executor.as_str()
    };
    let kind = provider_from_name(config_name);
    let model = model_for_provider(kind, models);

    let reason = if repo_aware {
        format!("Testing task with broad coverage routed to {}", config_name)
    } else {
        format!("Focused testing task routed to {}", config_name)
    };

    RoutingDecision {
        provider: kind,
        model,
        reason,
        requires_approval: false,
        risk: match task.risk {
            TaskRisk::Low => "low".to_string(),
            TaskRisk::Medium => "medium".to_string(),
            TaskRisk::High => "high".to_string(),
        },
        estimated_cost_class: "standard".to_string(),
        fallback_provider: if kind == ProviderKind::Cursor {
            ProviderKind::Anthropic
        } else {
            ProviderKind::Cursor
        },
        notes: vec!["Test execution requires exact file context or repo search.".to_string()],
    }
}

fn review_decision(task: &Task, models: &ModelsConfig, config: &Config) -> RoutingDecision {
    let repo_aware = task.requires_repo_search || task.estimated_files_touched >= 3;
    let config_name = if repo_aware {
        config.routing.default_repo_executor.as_str()
    } else {
        config.routing.default_scoped_executor.as_str()
    };
    let kind = provider_from_name(config_name);
    let model = model_for_provider(kind, models);

    let reason = if repo_aware {
        format!("Review task with broad scope routed to {}", config_name)
    } else {
        format!("Focused review task routed to {}", config_name)
    };

    RoutingDecision {
        provider: kind,
        model,
        reason,
        requires_approval: false,
        risk: match task.risk {
            TaskRisk::Low => "low".to_string(),
            TaskRisk::Medium => "medium".to_string(),
            TaskRisk::High => "high".to_string(),
        },
        estimated_cost_class: "standard".to_string(),
        fallback_provider: if kind == ProviderKind::Cursor {
            ProviderKind::Anthropic
        } else {
            ProviderKind::Cursor
        },
        notes: vec!["Review requires reading and reasoning about code changes.".to_string()],
    }
}

fn debugging_decision(task: &Task, models: &ModelsConfig, config: &Config) -> RoutingDecision {
    if task.risk == TaskRisk::High || task.complexity == TaskComplexity::Critical {
        cursor_premium_decision(task, models, config)
    } else {
        let config_name = config.routing.default_repo_executor.as_str();
        let kind = provider_from_name(config_name);
        let model = if kind == ProviderKind::Cursor {
            models.cursor.composer_model_id.clone()
        } else {
            model_for_provider(kind, models)
        };
        RoutingDecision {
            provider: kind,
            model,
            reason: "Cursor Composer is the default repo-aware debugging executor.".to_string(),
            requires_approval: false,
            risk: match task.risk {
                TaskRisk::Low => "low".to_string(),
                TaskRisk::Medium => "medium".to_string(),
                TaskRisk::High => "high".to_string(),
            },
            estimated_cost_class: "standard".to_string(),
            fallback_provider: ProviderKind::Anthropic,
            notes: vec!["Cursor Composer has abundant usage and supports repo search.".to_string()],
        }
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
    use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskStatus, TaskType};
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
            status: TaskStatus::Pending,
            requires_repo_search: false,
            estimated_files_touched: 1,
            context_is_exact: true,
            failure_count: 0,
            acceptance_criteria: Vec::new(),
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

    #[test]
    fn test_route_low_risk_planning_to_ollama() {
        let config = test_config();
        let mut task = test_task(TaskType::Planning);
        task.complexity = TaskComplexity::Low;
        task.risk = TaskRisk::Low;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Ollama);
        assert!(decision.reason.to_lowercase().contains("planning"));
    }

    #[test]
    fn test_route_high_complexity_planning_to_claude() {
        let config = test_config();
        let mut task = test_task(TaskType::Planning);
        task.complexity = TaskComplexity::High;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Anthropic);
        assert!(decision.requires_approval);
        assert!(decision.reason.to_lowercase().contains("planning"));
    }

    #[test]
    fn test_route_medium_planning_uses_default_planner() {
        let config = test_config();
        let task = test_task(TaskType::Planning);
        // Default planner is "claude"
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Anthropic);
        assert!(decision.reason.to_lowercase().contains("planning"));
    }

    #[test]
    fn test_route_planning_fallback_to_manual_when_no_providers_enabled() {
        let mut config = test_config();
        config.models.ollama.enabled = false;
        config.models.claude.enabled = false;
        config.models.codex.enabled = false;
        config.models.cursor.enabled = false;
        let mut task = test_task(TaskType::Planning);
        task.complexity = TaskComplexity::Low;
        task.risk = TaskRisk::Low;
        let decision = route(&task, &config);
        assert_eq!(decision.provider, ProviderKind::Manual);
    }

    #[test]
    fn test_route_planning_reason_mentions_planning_not_implementation() {
        let config = test_config();
        let task = test_task(TaskType::Planning);
        let decision = route(&task, &config);
        assert!(
            !decision.reason.to_lowercase().contains("implementation"),
            "Planning reason must not mention implementation: {}",
            decision.reason
        );
        assert!(decision.reason.to_lowercase().contains("planning"));
    }

    #[test]
    fn test_route_tests_has_test_reason() {
        let config = test_config();
        let mut task = test_task(TaskType::Tests);
        task.estimated_files_touched = 1;
        let decision = route(&task, &config);
        assert!(decision.reason.to_lowercase().contains("test"));
    }

    #[test]
    fn test_route_review_has_review_reason() {
        let config = test_config();
        let mut task = test_task(TaskType::Review);
        task.estimated_files_touched = 1;
        let decision = route(&task, &config);
        assert!(decision.reason.to_lowercase().contains("review"));
    }
}
