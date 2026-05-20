use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::review::review_task;
use crate::tasks::{fallback_task, resolve_task_from_graph};

#[derive(Args)]
pub struct ReviewArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,
}

pub fn run(args: ReviewArgs) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let graph_path = config.project.orca_dir.join("task-graph.yaml");
    let task = match resolve_task_from_graph(&graph_path, &args.task_id)? {
        Some(t) => t,
        None => {
            println!(
                "Warning: task {} not found in task graph. Using fallback.",
                args.task_id
            );
            fallback_task(&args.task_id)
        }
    };

    let result = review_task(&task);

    println!("Review for {}", args.task_id);
    println!("  Verdict: {}", result.verdict);
    println!("  Accepted: {}", if result.accepted { "yes" } else { "no" });

    if !result.reasons.is_empty() {
        println!("  Reasons:");
        for r in &result.reasons {
            println!("    - {}", r);
        }
    }

    if !result.missing_criteria.is_empty() {
        println!("  Missing criteria:");
        for m in &result.missing_criteria {
            println!("    - {}", m);
        }
    }

    if !result.risks.is_empty() {
        println!("  Risks:");
        for risk in &result.risks {
            println!("    - {}", risk);
        }
    }

    println!("  Recommended next step: {}", result.recommended_next_step);

    Ok(())
}
