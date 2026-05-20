use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::approvals::{check_approval, ApprovalCheck, ApprovalConfig};
use crate::config::schema::Config;
use crate::providers::factory::create_provider;
use crate::providers::traits::ProviderRequest;
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

    let graph_path = config.project.orca_dir.join("task-graph.yaml");
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
            prompt: format!("Dry-run execution for task {}", args.task_id),
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
            prompt: format!("Execute task {}", args.task_id),
            model_id: Some(decision.model.clone()),
            context: None,
            max_tokens: None,
        };

        let response = rt.block_on(provider.execute(request))?;
        println!("Execution result for {}", args.task_id);
        println!("  Provider: {}", response.provider);
        println!("  Model: {}", response.model_id);
        println!("  Status: {}", response.status);
        println!("  Output: {}", response.output);
    }

    Ok(())
}
