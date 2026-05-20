use crate::config::schema::ContextConfig;
use crate::context::ContextPacket;
use crate::router::decision::RoutingDecision;
use crate::tasks::Task;

/// Metadata about what was included in the built prompt.
#[derive(Debug, Clone, Default)]
pub struct PromptMetadata {
    pub sections_included: Vec<String>,
    pub included_context: bool,
    pub truncated: bool,
    pub prompt_char_count: usize,
    pub estimated_tokens: u32,
}

/// Build a structured provider prompt from task metadata, context packet, and routing info.
///
/// Respects `context.default_max_tokens` and `context.hard_max_tokens`.
/// If the prompt exceeds the hard max, the context packet section is truncated.
/// If it still exceeds, the prompt is hard-truncated with a warning appended.
pub fn build_provider_prompt(
    task: &Task,
    context_packet: Option<&ContextPacket>,
    decision: &RoutingDecision,
    dry_run: bool,
    config: &ContextConfig,
) -> (String, PromptMetadata) {
    let mut metadata = PromptMetadata::default();
    let mut sections: Vec<String> = Vec::new();

    let mode = if dry_run { "dry-run" } else { "real" };

    // Header
    let mut prompt = String::from("# Orca Execution Request\n\n");

    // Task metadata section
    let task_section = format_task_section(task, decision, mode);
    prompt.push_str(&task_section);
    sections.push("task_metadata".to_string());

    // Acceptance criteria section
    if !task.acceptance_criteria.is_empty() {
        prompt.push_str("## Acceptance Criteria\n");
        for ac in &task.acceptance_criteria {
            prompt.push_str(&format!("- {}\n", ac));
        }
        prompt.push('\n');
        sections.push("acceptance_criteria".to_string());
    }

    // Context packet section
    let context_content = context_packet.map_or_else(
        || build_minimal_context(task),
        crate::context::render_markdown,
    );

    if !context_content.is_empty() {
        prompt.push_str("## Context Packet\n");
        prompt.push_str(&context_content);
        prompt.push('\n');
        sections.push("context_packet".to_string());
        metadata.included_context = true;
    }

    // Instructions section
    prompt.push_str("## Instructions\n");
    prompt.push_str("- Complete the task using only the supplied context.\n");
    prompt.push_str("- If context is insufficient, say exactly what is missing.\n");
    prompt.push_str("- Do not claim completion unless acceptance criteria are met.\n");
    prompt.push_str("- Return concise, actionable output.\n");
    prompt.push_str("- Do not leak secrets.\n");
    sections.push("execution_instructions".to_string());

    // Token safety: estimate and truncate if needed
    let estimated = estimate_tokens(&prompt);
    metadata.prompt_char_count = prompt.len();
    metadata.estimated_tokens = estimated;

    if estimated > config.hard_max_tokens {
        // Truncate context packet section if present
        if metadata.included_context {
            let max_context_len = (config.hard_max_tokens as usize)
                .saturating_mul(4)
                .saturating_sub(500);
            let truncated_context = if context_content.len() > max_context_len {
                format!(
                    "{}\n\n[Context truncated due to token limit]",
                    &context_content[..max_context_len]
                )
            } else {
                context_content
            };

            // Rebuild prompt with truncated context
            prompt = String::from("# Orca Execution Request\n\n");
            prompt.push_str(&task_section);
            if !task.acceptance_criteria.is_empty() {
                prompt.push_str("## Acceptance Criteria\n");
                for ac in &task.acceptance_criteria {
                    prompt.push_str(&format!("- {}\n", ac));
                }
                prompt.push('\n');
            }
            prompt.push_str("## Context Packet\n");
            prompt.push_str(&truncated_context);
            prompt.push('\n');
            prompt.push_str("## Instructions\n");
            prompt.push_str("- Complete the task using only the supplied context.\n");
            prompt.push_str("- If context is insufficient, say exactly what is missing.\n");
            prompt.push_str("- Do not claim completion unless acceptance criteria are met.\n");
            prompt.push_str("- Return concise, actionable output.\n");
            prompt.push_str("- Do not leak secrets.\n");

            metadata.truncated = true;
            metadata.prompt_char_count = prompt.len();
            metadata.estimated_tokens = estimate_tokens(&prompt);
        }

        // If still over hard max, hard-truncate the entire prompt
        if metadata.estimated_tokens > config.hard_max_tokens {
            let hard_limit_chars = (config.hard_max_tokens as usize).saturating_mul(4);
            if prompt.len() > hard_limit_chars {
                prompt.truncate(hard_limit_chars.saturating_sub(100));
                prompt.push_str("\n\n[PROMPT TRUNCATED: exceeded hard token limit]");
                metadata.truncated = true;
                metadata.prompt_char_count = prompt.len();
                metadata.estimated_tokens = estimate_tokens(&prompt);
            }
        }
    }

    metadata.sections_included = sections;
    (prompt, metadata)
}

fn format_task_section(task: &Task, decision: &RoutingDecision, mode: &str) -> String {
    let mut s = String::from("## Task\n");
    s.push_str(&format!("- **ID**: {}\n", task.id));
    s.push_str(&format!("- **Title**: {}\n", task.title));
    s.push_str(&format!("- **Kind**: {:?}\n", task.task_type));
    s.push_str(&format!("- **Risk**: {:?}\n", task.risk));
    if !task.description.is_empty() {
        s.push_str(&format!("- **Description**: {}\n", task.description));
    }
    s.push_str(&format!("- **Provider**: {}\n", decision.provider));
    s.push_str(&format!("- **Model**: {}\n", decision.model));
    s.push_str(&format!("- **Execution Mode**: {}\n", mode));
    s.push('\n');
    s
}

fn build_minimal_context(task: &Task) -> String {
    let mut s = String::new();
    s.push_str(&format!("- **Task ID**: {}\n", task.id));
    s.push_str(&format!("- **Title**: {}\n", task.title));
    s.push_str(&format!("- **Type**: {:?}\n", task.task_type));
    s.push_str(&format!("- **Risk**: {:?}\n", task.risk));
    if !task.description.is_empty() {
        s.push_str(&format!("- **Description**: {}\n", task.description));
    }
    if !task.acceptance_criteria.is_empty() {
        s.push_str("- **Acceptance Criteria**:\n");
        for ac in &task.acceptance_criteria {
            s.push_str(&format!("  - {}\n", ac));
        }
    }
    s
}

/// Rough token estimator: ~4 characters per token for English text.
fn estimate_tokens(text: &str) -> u32 {
    ((text.len() as f64) / 4.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::{Task, TaskType};

    fn test_decision() -> RoutingDecision {
        RoutingDecision {
            provider: crate::providers::traits::ProviderKind::Ollama,
            model: "qwen3.5:4b".to_string(),
            reason: "Low-risk planning task".to_string(),
            requires_approval: false,
            risk: "low".to_string(),
            estimated_cost_class: "low".to_string(),
            fallback_provider: crate::providers::traits::ProviderKind::Mock,
            notes: Vec::new(),
        }
    }

    fn test_config() -> ContextConfig {
        ContextConfig {
            default_max_tokens: 3000,
            hard_max_tokens: 8000,
            prefer_summaries_over_raw_files: true,
        }
    }

    #[test]
    fn test_prompt_includes_task_id() {
        let task = Task::new("TASK-001", "Test task", TaskType::Planning);
        let decision = test_decision();
        let config = test_config();
        let (prompt, meta) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("TASK-001"));
        assert!(meta
            .sections_included
            .contains(&"task_metadata".to_string()));
    }

    #[test]
    fn test_prompt_includes_task_title() {
        let task = Task::new("TASK-001", "Implement auth", TaskType::Implementation);
        let decision = test_decision();
        let config = test_config();
        let (prompt, _) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("Implement auth"));
    }

    #[test]
    fn test_prompt_includes_description() {
        let mut task = Task::new("TASK-001", "Test", TaskType::Planning);
        task.description = "Set up directories.".to_string();
        let decision = test_decision();
        let config = test_config();
        let (prompt, _) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("Set up directories."));
    }

    #[test]
    fn test_prompt_includes_acceptance_criteria() {
        let mut task = Task::new("TASK-001", "Test", TaskType::Planning);
        task.acceptance_criteria.push("Tests pass".to_string());
        let decision = test_decision();
        let config = test_config();
        let (prompt, meta) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("Tests pass"));
        assert!(meta
            .sections_included
            .contains(&"acceptance_criteria".to_string()));
    }

    #[test]
    fn test_prompt_includes_context_packet() {
        let task = Task::new("TASK-001", "Test", TaskType::Planning);
        let decision = test_decision();
        let config = test_config();
        let packet = ContextPacket::new("TASK-001", "Test");
        let (prompt, meta) = build_provider_prompt(&task, Some(&packet), &decision, false, &config);
        assert!(prompt.contains("Context Packet"));
        assert!(meta.included_context);
        assert!(meta
            .sections_included
            .contains(&"context_packet".to_string()));
    }

    #[test]
    fn test_prompt_includes_instructions() {
        let task = Task::new("TASK-001", "Test", TaskType::Planning);
        let decision = test_decision();
        let config = test_config();
        let (prompt, meta) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("Complete the task using only the supplied context."));
        assert!(meta
            .sections_included
            .contains(&"execution_instructions".to_string()));
    }

    #[test]
    fn test_prompt_includes_provider_and_model() {
        let task = Task::new("TASK-001", "Test", TaskType::Planning);
        let decision = test_decision();
        let config = test_config();
        let (prompt, _) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("ollama"));
        assert!(prompt.contains("qwen3.5:4b"));
    }

    #[test]
    fn test_prompt_includes_execution_mode() {
        let task = Task::new("TASK-001", "Test", TaskType::Planning);
        let decision = test_decision();
        let config = test_config();
        let (prompt, _) = build_provider_prompt(&task, None, &decision, true, &config);
        assert!(prompt.contains("dry-run"));
    }

    #[test]
    fn test_prompt_includes_risk() {
        let task = Task::new("TASK-001", "Test", TaskType::Planning);
        let decision = test_decision();
        let config = test_config();
        let (prompt, _) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("Low"));
    }

    #[test]
    fn test_prompt_token_estimate_non_trivial() {
        let mut task = Task::new("TASK-001", "Test task with metadata", TaskType::Planning);
        task.description =
            "This is a longer description that should increase token count.".to_string();
        task.acceptance_criteria.push("Criterion one".to_string());
        task.acceptance_criteria.push("Criterion two".to_string());
        let decision = test_decision();
        let config = test_config();
        let (prompt, meta) = build_provider_prompt(&task, None, &decision, false, &config);
        // A task with metadata should have more than ~5 tokens (task-id-only is ~3-5 chars / 4 = ~1 token, but realistically task-id string is ~10 chars)
        assert!(
            meta.estimated_tokens > 5,
            "Expected >5 tokens for metadata-rich task, got {}",
            meta.estimated_tokens
        );
        assert!(
            prompt.len() > 50,
            "Expected prompt longer than 50 chars, got {}",
            prompt.len()
        );
    }

    #[test]
    fn test_prompt_truncation_respects_hard_max() {
        let task = Task::new("TASK-001", "Test", TaskType::Planning);
        let decision = test_decision();
        let mut config = test_config();
        config.hard_max_tokens = 50; // Very low to force truncation
        let mut packet = ContextPacket::new("TASK-001", "Test");
        packet.goal = "x".repeat(1000);
        let (_prompt, meta) =
            build_provider_prompt(&task, Some(&packet), &decision, false, &config);
        assert!(meta.truncated);
        assert!(
            meta.estimated_tokens <= config.hard_max_tokens,
            "Expected <= {} tokens, got {}",
            config.hard_max_tokens,
            meta.estimated_tokens
        );
    }

    #[test]
    fn test_smoke_test_fixture_prompt() {
        // TASK-CLOUD-001 equivalent
        let mut task = Task::new(
            "TASK-CLOUD-001",
            "Ollama Cloud smoke test",
            TaskType::Planning,
        );
        task.description = "Reply with exactly ORCA_CLOUD_OK.".to_string();
        task.acceptance_criteria
            .push("Output must be exactly ORCA_CLOUD_OK.".to_string());
        let decision = test_decision();
        let config = test_config();
        let (prompt, _) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(prompt.contains("TASK-CLOUD-001"));
        assert!(prompt.contains("Ollama Cloud smoke test"));
        assert!(prompt.contains("Reply with exactly ORCA_CLOUD_OK."));
        assert!(prompt.contains("Output must be exactly ORCA_CLOUD_OK."));
    }

    #[test]
    fn test_minimal_context_when_no_packet() {
        let mut task = Task::new("TASK-001", "Test", TaskType::Planning);
        task.description = "Do something.".to_string();
        let decision = test_decision();
        let config = test_config();
        let (prompt, meta) = build_provider_prompt(&task, None, &decision, false, &config);
        assert!(meta.included_context); // minimal context counts as included
        assert!(prompt.contains("Do something."));
        assert!(prompt.contains("Context Packet"));
    }
}
