use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::approvals::{check_approval, ApprovalCheck, ApprovalConfig};
use crate::config::schema::Config;
use crate::context::ContextPacket;
use crate::patch;
use crate::prompts::build_provider_prompt;
use crate::providers::factory::create_provider;
use crate::providers::traits::{ExecutionStatus, ProviderRequest};
use crate::results;
use crate::router::route;
use crate::tasks::resolve_task_from_graph;

#[derive(Args)]
pub struct ExecuteArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Provider to use")]
    pub provider: Option<String>,
}

pub fn run(args: ExecuteArgs, dry_run: bool, yes: bool) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let orca_dir = &config.project.orca_dir;
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

    let decision = route(&task, &config);

    let approval_config = ApprovalConfig {
        require_approval_for_premium: config.execution.require_approval_for_premium,
        require_approval_for_destructive: true,
        require_approval_for_high_risk: true,
    };

    match check_approval(&task, &decision, &approval_config, yes) {
        ApprovalCheck::Block(reason) => {
            println!("Approval gate blocked: {}", reason);
            return Ok(());
        }
        ApprovalCheck::Pass => {}
    }

    // Try to load an existing context packet for this task
    let context_packet_path = orca_dir
        .join("context-packets")
        .join(format!("{}.md", args.task_id));
    let existing_context = if context_packet_path.exists() {
        std::fs::read_to_string(&context_packet_path).ok()
    } else {
        None
    };

    let context_packet = existing_context.as_ref().map(|_md| {
        // Reconstruct a minimal ContextPacket from existing markdown for prompt building
        let mut cp = ContextPacket::new(&args.task_id, &task.title);
        cp.task_type = format!("{:?}", task.task_type).to_lowercase();
        cp.goal = task.description.clone();
        cp.acceptance_criteria = task.acceptance_criteria.clone();
        cp.risks.push(format!("{:?}", task.risk));
        cp.routing_recommendation = format!(
            "Provider: {} | Model: {} | Reason: {}",
            decision.provider, decision.model, decision.reason
        );
        cp
    });

    let (prompt, prompt_meta) = build_provider_prompt(
        &task,
        context_packet.as_ref(),
        &decision,
        dry_run,
        &config.context,
    );

    if dry_run {
        println!("=== DRY RUN ===");
        println!("Task: {}", args.task_id);
        println!("Provider: {} (mock)", decision.provider);
        println!("Model: {}", decision.model);
        println!("Reason: {}", decision.reason);
        println!(
            "Approval required: {}",
            if decision.requires_approval {
                "yes"
            } else {
                "no"
            }
        );

        let provider = create_provider(decision.provider, &config);
        let request = ProviderRequest {
            task_id: args.task_id.clone(),
            prompt: prompt.clone(),
            model_id: Some(decision.model.clone()),
            context: None,
            max_tokens: None,
        };
        let cost = provider.estimate_cost(&request);
        println!(
            "Estimated cost: {} input tokens, {} output tokens, ${:.4}",
            cost.input_tokens, cost.output_tokens, cost.cost_usd
        );
        println!("No API calls were made.");
        println!("=== END DRY RUN ===");
    } else {
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
        response.context_packet_path = Some(context_packet_path.to_string_lossy().to_string());
        if response.status == ExecutionStatus::Success {
            if let Err(e) = results::save_result(orca_dir, &response) {
                eprintln!("Warning: failed to save execution result: {}", e);
            }
            if let Some(proposal) = patch::PatchProposal::from_response(&response) {
                if let Err(e) = patch::save_patch_proposal(orca_dir, &proposal) {
                    eprintln!("Warning: failed to save patch proposal: {}", e);
                }
            }
        }
        println!("Execution result for {}", args.task_id);
        println!("  Provider: {}", response.provider);
        println!("  Model: {}", response.model_id);
        println!("  Status: {}", response.status);
        println!("  Output: {}", response.output);
    }

    Ok(())
}
