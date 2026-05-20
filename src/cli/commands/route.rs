use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::router::route;
use crate::tasks::{resolve_task_from_graph, TaskType};

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

    let graph_path = config.project.orca_dir.join("task-graph.yaml");

    let mut task = match resolve_task_from_graph(&graph_path, &args.task_id)? {
        Some(t) => t,
        None => {
            return Err(anyhow::anyhow!(
                "task {} not found in {}",
                args.task_id,
                graph_path.display()
            ));
        }
    };

    // Allow CLI override of task type
    if let Some(override_type) = args.task_type.as_deref() {
        if let Ok(tt) = TaskType::from_str(override_type) {
            task.task_type = tt;
        }
    }

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
