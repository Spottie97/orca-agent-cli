use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;

use crate::approvals::{check_approval, ApprovalCheck, ApprovalConfig};
use crate::config::schema::Config;
use crate::context::{render_markdown, ContextPacket};
use crate::memory::obsidian::format_task_completion_entry;
use crate::memory::MemoryStore;
use crate::patch;
use crate::prompts::build_provider_prompt;
use crate::providers::factory::create_provider;
use crate::providers::mock::MockProvider;
use crate::providers::traits::{CostEstimate, ExecutionStatus, Provider, ProviderRequest};
use crate::results;
use crate::review::review_task;
use crate::router::route;
use crate::state::{self, State, TaskState};
use crate::tasks::resolve_task_from_graph;
use crate::utils::fs::safe_write;

#[derive(Args)]
pub struct RunArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Provider to use")]
    pub provider: Option<String>,

    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
}

#[derive(Serialize)]
struct RunOutput {
    task_id: String,
    mode: String,
    provider: String,
    model: String,
    approval_blocked: Option<String>,
    estimated_cost: Option<CostEstimate>,
    review_verdict: String,
    review_next_step: String,
}

pub fn run(args: RunArgs, dry_run: bool, yes: bool) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let orca_dir = &config.project.orca_dir;
    let state_path = orca_dir.join("state.json");

    if !args.json {
        if dry_run {
            println!("=== Orca Run (dry-run): {} ===", args.task_id);
        } else {
            println!("=== Orca Run: {} ===", args.task_id);
        }
    }

    // Step 1: Build task from task graph if available
    let graph_path = orca_dir.join("task-graph.yaml");
    let task = match resolve_task_from_graph(&graph_path, &args.task_id)? {
        Some(t) => t,
        None => {
            return Err(anyhow::anyhow!(
                "task {} not found in {}",
                args.task_id,
                graph_path.display()
            ));
        }
    };
    if !args.json {
        if dry_run {
            println!(
                "[1/5] Dry run: loaded task {} ({:?})",
                args.task_id, task.task_type
            );
        } else {
            println!("[1/5] Task loaded: {} ({:?})", args.task_id, task.task_type);
        }
    }

    // Step 2: Generate context packet
    let decision = route(&task, &config);
    let mut packet = ContextPacket::new(&args.task_id, &task.title);
    packet.task_type = format!("{:?}", task.task_type).to_lowercase();
    packet.suggested_provider = format!("{}", decision.provider);
    packet.routing_recommendation = format!(
        "Provider: {} | Model: {} | Reason: {}",
        decision.provider, decision.model, decision.reason
    );
    packet.acceptance_criteria = task.acceptance_criteria.clone();

    let context_path = orca_dir
        .join("context-packets")
        .join(format!("{}.md", args.task_id));
    if !dry_run {
        safe_write(&context_path, &render_markdown(&packet))?;
    }
    if !args.json {
        if dry_run {
            println!(
                "[2/5] Dry run: would write context packet to {}",
                context_path.display()
            );
        } else {
            println!("[2/5] Context packet written to {}", context_path.display());
        }
    }

    let mut output = RunOutput {
        task_id: args.task_id.clone(),
        mode: if dry_run {
            "dry-run".to_string()
        } else {
            "real".to_string()
        },
        provider: decision.provider.to_string(),
        model: decision.model.clone(),
        approval_blocked: None,
        estimated_cost: None,
        review_verdict: String::new(),
        review_next_step: String::new(),
    };

    // Step 3: Check approval gates
    let approval_config = ApprovalConfig {
        require_approval_for_premium: config.execution.require_approval_for_premium,
        require_approval_for_destructive: true,
        require_approval_for_high_risk: true,
    };
    match check_approval(&task, &decision, &approval_config, yes) {
        ApprovalCheck::Block(reason) => {
            output.approval_blocked = Some(reason.clone());
            if args.json {
                let json = serde_json::to_string_pretty(&output)
                    .with_context(|| "Failed to serialize run output to JSON")?;
                println!("{}", json);
            } else {
                if dry_run {
                    println!("[3/5] Dry run: approval gate would block: {}", reason);
                } else {
                    println!("[3/5] Approval gate blocked: {}", reason);
                }
                println!("Loop halted.");
            }
            return Ok(());
        }
        ApprovalCheck::Pass => {
            if !args.json {
                if dry_run {
                    println!("[3/5] Dry run: would check approval gate (would pass)");
                } else {
                    println!("[3/5] Approval gate passed");
                }
            }
        }
    }

    let mut result_path_option: Option<std::path::PathBuf> = None;
    let mut patch_path_option: Option<std::path::PathBuf> = None;
    let mut exec_output: Option<String> = None;
    let mut exec_status: Option<String> = None;
    let mut exec_duration_ms: Option<u64> = None;
    let mut exec_input_tokens: Option<u32> = None;
    let mut exec_output_tokens: Option<u32> = None;

    // Step 4: Execute (dry-run uses mock)
    let (prompt, prompt_meta) =
        build_provider_prompt(&task, Some(&packet), &decision, dry_run, &config.context);

    if dry_run {
        if !args.json {
            println!(
                "[4/5] Dry run: would execute provider {} (mock)",
                decision.provider
            );
        }
        let provider = MockProvider::default();
        let request = ProviderRequest {
            task_id: args.task_id.clone(),
            prompt: prompt.clone(),
            model_id: Some(decision.model.clone()),
            context: None,
            max_tokens: None,
        };
        let cost = provider.estimate_cost(&request);
        output.estimated_cost = Some(cost);
        if !args.json {
            println!("  Provider: {} (mock)", decision.provider);
            println!("  Model: {}", decision.model);
            println!(
                "  Estimated cost: {} input tokens, {} output tokens",
                output.estimated_cost.as_ref().unwrap().input_tokens,
                output.estimated_cost.as_ref().unwrap().output_tokens
            );
            println!("  No API calls were made.");
        }
    } else {
        if !args.json {
            println!("[4/5] Executing provider {} ...", decision.provider);
        }
        let rt = tokio::runtime::Runtime::new()?;
        let provider = create_provider(decision.provider, &config);
        let request = ProviderRequest {
            task_id: args.task_id.clone(),
            prompt,
            model_id: Some(decision.model.clone()),
            context: None,
            max_tokens: None,
        };
        let mut response = rt.block_on(provider.execute(request))?;
        response.prompt_included_context = prompt_meta.included_context;
        response.prompt_sections_included = prompt_meta.sections_included;
        response.context_packet_path = Some(context_path.to_string_lossy().to_string());
        exec_output = Some(response.output.clone());
        exec_status = Some(response.status.to_string());
        exec_duration_ms = response.duration_ms;
        exec_input_tokens = response.input_tokens;
        exec_output_tokens = response.output_tokens;
        if response.status == ExecutionStatus::Success {
            match results::save_result(orca_dir, &response) {
                Ok(path) => {
                    result_path_option = Some(path);
                }
                Err(e) => {
                    eprintln!("Warning: failed to save execution result: {}", e);
                }
            }
            if let Some(proposal) = patch::PatchProposal::from_response(&response) {
                match patch::save_patch_proposal(orca_dir, &proposal) {
                    Ok(path) => {
                        patch_path_option = Some(path);
                    }
                    Err(e) => {
                        eprintln!("Warning: failed to save patch proposal: {}", e);
                    }
                }
            }
        }
        output.estimated_cost = Some(provider.estimate_cost(&ProviderRequest {
            task_id: args.task_id.clone(),
            prompt: response.output.clone(),
            model_id: Some(decision.model.clone()),
            context: None,
            max_tokens: None,
        }));
        if !args.json {
            println!("  Provider: {}", response.provider);
            println!("  Model: {}", response.model_id);
            println!("  Status: {}", response.status);
        }
    }

    // Step 5: Review
    let review_result = review_task(
        &task,
        exec_output.as_deref(),
        exec_status.as_deref(),
        exec_duration_ms,
        exec_input_tokens,
        exec_output_tokens,
    );
    output.review_verdict = review_result.verdict.to_string();
    output.review_next_step = review_result.recommended_next_step.clone();
    if !args.json {
        if dry_run {
            println!(
                "[5/5] Dry run: would review result (verdict: {} — {})",
                review_result.verdict, review_result.recommended_next_step
            );
        } else {
            println!(
                "[5/5] Review: {} — {}",
                review_result.verdict, review_result.recommended_next_step
            );
        }
    }

    // Step 6: Save review artifact
    if !dry_run {
        if let Err(e) = crate::review::save_review(orca_dir, &review_result) {
            eprintln!("Warning: failed to save review artifact: {}", e);
        }
    }

    // Step 7: Update memory
    if !dry_run {
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
                result_path: result_path_option
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                updated_at: Some(now.clone()),
            });
        task_state.status = "complete".to_string();
        task_state.updated_at = Some(now.clone());
        state::save(&state, &state_path)?;

        let store = MemoryStore::new(orca_dir);
        let history_entry = format_task_completion_entry(
            &args.task_id,
            &decision.provider.to_string(),
            &decision.model,
            "complete",
            &review_result.verdict.to_string(),
            exec_output.as_deref(),
            exec_duration_ms,
            exec_input_tokens,
            exec_output_tokens,
            result_path_option
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .as_deref(),
            patch_path_option
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .as_deref(),
            Some(&format!(
                "{}/reviews/{}.json",
                orca_dir.display(),
                args.task_id
            )),
            &review_result.recommended_next_step,
        );
        let _memory_written = store.append_to_note("tasks", &args.task_id, &history_entry)?;
    }

    if args.json {
        let json = serde_json::to_string_pretty(&output)
            .with_context(|| "Failed to serialize run output to JSON")?;
        println!("{}", json);
    } else {
        if dry_run {
            println!("Dry run: would update memory (state + task history)");
            println!("=== Dry run complete; no files were changed ===");
        } else {
            println!(
                "Memory updated at {}/tasks/{}.md",
                orca_dir.display(),
                args.task_id
            );
            println!("=== Run complete ===");
        }
    }

    Ok(())
}
