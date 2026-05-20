use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::approvals::{check_approval, ApprovalCheck, ApprovalConfig};
use crate::config::schema::Config;
use crate::context::{render_markdown, ContextPacket};
use crate::memory::MemoryStore;
use crate::providers::mock::MockProvider;
use crate::providers::traits::{Provider, ProviderRequest};
use crate::review::review_task;
use crate::router::route;
use crate::state::{self, State, TaskState};
use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskStatus, TaskType};
use crate::utils::fs::safe_write;

#[derive(Args)]
pub struct RunArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Provider to use")]
    pub provider: Option<String>,
}

pub fn run(args: RunArgs, dry_run: bool, yes: bool) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let orca_dir = &config.project.orca_dir;
    let state_path = orca_dir.join("state.json");

    println!("=== Orca Run: {} ===", args.task_id);

    // Step 1: Build task
    let task = Task {
        id: args.task_id.clone(),
        title: format!("Task {}", args.task_id),
        description: "".to_string(),
        task_type: TaskType::Implementation,
        complexity: TaskComplexity::Medium,
        risk: TaskRisk::Low,
        status: TaskStatus::Pending,
        requires_repo_search: false,
        estimated_files_touched: 1,
        context_is_exact: true,
        failure_count: 0,
        acceptance_criteria: Vec::new(),
    };
    println!("[1/5] Task created: {}", args.task_id);

    // Step 2: Generate context packet
    let decision = route(&task, &config);
    let mut packet = ContextPacket::new(&args.task_id, &task.title);
    packet.task_type = "implementation".to_string();
    packet.suggested_provider = format!("{}", decision.provider);
    packet.routing_recommendation = format!(
        "Provider: {} | Model: {} | Reason: {}",
        decision.provider, decision.model, decision.reason
    );
    packet
        .acceptance_criteria
        .push("Task completes successfully".to_string());

    let context_path = orca_dir
        .join("context-packets")
        .join(format!("{}.md", args.task_id));
    safe_write(&context_path, &render_markdown(&packet))?;
    println!("[2/5] Context packet written to {}", context_path.display());

    // Step 3: Check approval gates
    let approval_config = ApprovalConfig {
        require_approval_for_premium: config.execution.require_approval_for_premium,
        require_approval_for_destructive: true,
        require_approval_for_high_risk: true,
    };
    match check_approval(&task, &decision, &approval_config, yes) {
        ApprovalCheck::Block(reason) => {
            println!("[3/5] Approval gate blocked: {}", reason);
            println!("Loop halted.");
            return Ok(());
        }
        ApprovalCheck::Pass => {
            println!("[3/5] Approval gate passed");
        }
    }

    // Step 4: Execute (dry-run uses mock)
    if dry_run {
        println!("[4/5] Execution (dry-run):");
        let provider = MockProvider::default();
        let request = ProviderRequest {
            task_id: args.task_id.clone(),
            prompt: format!("Dry-run execution for task {}", args.task_id),
            model_id: Some(decision.model.clone()),
            context: None,
            max_tokens: None,
        };
        let cost = provider.estimate_cost(&request);
        println!("  Provider: {} (mock)", decision.provider);
        println!("  Model: {}", decision.model);
        println!(
            "  Estimated cost: {} input tokens, {} output tokens",
            cost.input_tokens, cost.output_tokens
        );
        println!("  No API calls were made.");
    } else {
        println!("[4/5] Execution (real mode not yet implemented in MVP)");
    }

    // Step 5: Review
    let review_result = review_task(&task);
    println!(
        "[5/5] Review: {} — {}",
        review_result.verdict, review_result.recommended_next_step
    );

    // Step 6: Update memory
    let mut state = if state_path.exists() {
        state::load(&state_path)?
    } else {
        State::default()
    };
    let now = format!("{:?}", std::time::SystemTime::now());
    let task_state = state
        .tasks
        .entry(args.task_id.clone())
        .or_insert_with(|| TaskState {
            status: "complete".to_string(),
            failure_count: 0,
            assigned_provider: None,
            assigned_model: None,
            context_packet_path: Some(context_path.to_string_lossy().to_string()),
            result_path: None,
            updated_at: Some(now.clone()),
        });
    task_state.status = "complete".to_string();
    task_state.updated_at = Some(now.clone());
    state::save(&state, &state_path)?;

    let store = MemoryStore::new(orca_dir);
    let history_entry = format!(
        "\n## Run {}\n\n- Status: complete\n- Verdict: {}\n\n",
        args.task_id, review_result.verdict
    );
    store.append_to_note("tasks", &args.task_id, &history_entry)?;

    println!("Memory updated.");
    println!("=== Run complete ===");

    Ok(())
}
