use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::review::review_task;
use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskStatus, TaskType};

#[derive(Args)]
pub struct ReviewArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,
}

pub fn run(args: ReviewArgs) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let _config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    // In a full implementation, this would load the task from state.
    // For MVP, we create a minimal task with the given ID.
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
