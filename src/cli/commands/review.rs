use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::review::review_task;
use crate::tasks::resolve_task_from_graph;

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
            return Err(anyhow::anyhow!(
                "task {} not found in {}",
                args.task_id,
                graph_path.display()
            ));
        }
    };

    // Try to load execution result for richer review
    let result_artifact = crate::results::load_result(&config.project.orca_dir, &args.task_id).ok();

    let review = if let Some(ref artifact) = result_artifact {
        review_task(
            &task,
            Some(&artifact.output),
            Some(&artifact.status),
            artifact.duration_ms,
            artifact.input_tokens,
            artifact.output_tokens,
        )
    } else {
        review_task(&task, None, None, None, None, None)
    };

    println!("Review for {}", args.task_id);
    println!("  Verdict: {}", review.verdict);
    println!("  Accepted: {}", if review.accepted { "yes" } else { "no" });

    if !review.reasons.is_empty() {
        println!("  Reasons:");
        for r in &review.reasons {
            println!("    - {}", r);
        }
    }

    if !review.missing_criteria.is_empty() {
        println!("  Missing criteria:");
        for m in &review.missing_criteria {
            println!("    - {}", m);
        }
    }

    if !review.risks.is_empty() {
        println!("  Risks:");
        for risk in &review.risks {
            println!("    - {}", risk);
        }
    }

    if let Some(ref status) = review.execution_status {
        println!("  Execution status: {}", status);
    }
    if let Some(ms) = review.duration_ms {
        println!("  Duration: {} ms", ms);
    }

    println!("  Recommended next step: {}", review.recommended_next_step);

    // Save review artifact
    if let Err(e) = crate::review::save_review(&config.project.orca_dir, &review) {
        eprintln!("Warning: failed to save review artifact: {}", e);
    }

    Ok(())
}
