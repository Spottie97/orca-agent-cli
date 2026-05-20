use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::router::route;
use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskStatus, TaskType};

#[derive(Args)]
pub struct RouteArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Task type override")]
    pub task_type: Option<String>,
}

pub fn run(args: RouteArgs) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    // In a full implementation, this would load the task from the task graph.
    // For now, we create a minimal task with the given ID.
    let task_type = parse_task_type(args.task_type.as_deref());
    let task = Task {
        id: args.task_id.clone(),
        title: format!("Task {}", args.task_id),
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
    };

    let decision = route(&task, &config);

    println!("Routing decision for {}", args.task_id);
    println!("  Provider: {}", decision.provider);
    println!("  Model: {}", decision.model);
    println!("  Reason: {}", decision.reason);
    println!(
        "  Approval required: {}",
        if decision.requires_approval {
            "yes"
        } else {
            "no"
        }
    );
    println!("  Risk: {}", decision.risk);
    println!("  Estimated cost class: {}", decision.estimated_cost_class);
    println!("  Fallback provider: {}", decision.fallback_provider);

    if !decision.notes.is_empty() {
        println!("  Notes:");
        for note in &decision.notes {
            println!("    - {}", note);
        }
    }

    Ok(())
}

fn parse_task_type(s: Option<&str>) -> TaskType {
    match s {
        Some("summary") => TaskType::Summary,
        Some("compression") => TaskType::Compression,
        Some("memory-update") => TaskType::MemoryUpdate,
        Some("docs") => TaskType::Docs,
        Some("architecture") => TaskType::Architecture,
        Some("decomposition") => TaskType::Decomposition,
        Some("risk-analysis") => TaskType::RiskAnalysis,
        Some("planning") => TaskType::Planning,
        Some("implementation") => TaskType::Implementation,
        Some("tests") => TaskType::Tests,
        Some("refactor") => TaskType::Refactor,
        Some("review") => TaskType::Review,
        Some("debugging") => TaskType::Debugging,
        Some("research") => TaskType::Research,
        _ => TaskType::Planning,
    }
}
