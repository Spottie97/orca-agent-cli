use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::context::{render_markdown, ContextPacket};
use crate::router::route;
use crate::tasks::resolve_task_from_graph;
use crate::utils::fs::safe_write;

#[derive(Args)]
pub struct ContextArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Maximum tokens for context packet")]
    pub max_tokens: Option<u32>,

    #[arg(long, help = "Regenerate context packet")]
    pub refresh: bool,
}

pub fn run(args: ContextArgs, dry_run: bool) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let orca_dir = &config.project.orca_dir;
    let output_path = orca_dir
        .join("context-packets")
        .join(format!("{}.md", args.task_id));

    if output_path.exists() && !args.refresh {
        println!(
            "Context packet already exists at {}. Use --refresh to regenerate.",
            output_path.display()
        );
        return Ok(());
    }

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

    let mut packet = ContextPacket::new(&args.task_id, &task.title);
    packet.task_type = format!("{:?}", task.task_type).to_lowercase();
    packet.goal = format!(
        "Execute task {} through provider {}",
        args.task_id, decision.provider
    );
    packet.suggested_provider = format!("{}", decision.provider);
    packet.routing_recommendation = format!(
        "Provider: {} | Model: {} | Reason: {} | Approval required: {}",
        decision.provider,
        decision.model,
        decision.reason,
        if decision.requires_approval {
            "yes"
        } else {
            "no"
        }
    );
    packet
        .acceptance_criteria
        .push("Task completes successfully".to_string());
    packet.constraints.push("Minimize token usage".to_string());

    let md = render_markdown(&packet);

    if dry_run {
        println!(
            "Dry run: would write context packet to {} ({} bytes)",
            output_path.display(),
            md.len()
        );
        return Ok(());
    }

    safe_write(&output_path, &md).with_context(|| {
        format!(
            "Failed to write context packet to {}",
            output_path.display()
        )
    })?;

    println!(
        "Context packet written to {} ({} bytes)",
        output_path.display(),
        md.len()
    );

    Ok(())
}
