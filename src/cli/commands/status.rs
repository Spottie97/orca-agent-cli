use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::config::schema::Config;
use crate::state::{self, State};

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSummary {
    pub project_name: String,
    pub current_phase: String,
    pub total_tasks: usize,
    pub completed: usize,
    pub pending: usize,
    pub blocked: usize,
    pub failed: usize,
}

pub fn run(args: StatusArgs) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let state_path = config.project.orca_dir.join("state.json");
    let state = if state_path.exists() {
        state::load(&state_path)?
    } else {
        State::default()
    };

    let mut completed = 0;
    let mut pending = 0;
    let mut blocked = 0;
    let mut failed = 0;

    for task_state in state.tasks.values() {
        match task_state.status.as_str() {
            "complete" => completed += 1,
            "pending" => pending += 1,
            "blocked" => blocked += 1,
            "failed" => failed += 1,
            _ => pending += 1,
        }
    }

    let summary = StatusSummary {
        project_name: state.project_name.clone(),
        current_phase: state.current_phase.clone(),
        total_tasks: state.tasks.len(),
        completed,
        pending,
        blocked,
        failed,
    };

    if args.json {
        let json = serde_json::to_string_pretty(&summary)
            .with_context(|| "Failed to serialize status to JSON")?;
        println!("{}", json);
    } else {
        println!("Project: {}", summary.project_name);
        println!("Phase: {}", summary.current_phase);
        println!("Tasks: {} total", summary.total_tasks);
        println!("  Completed: {}", summary.completed);
        println!("  Pending: {}", summary.pending);
        println!("  Blocked: {}", summary.blocked);
        println!("  Failed: {}", summary.failed);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_summary_serialization() {
        let summary = StatusSummary {
            project_name: "Test".to_string(),
            current_phase: "MVP".to_string(),
            total_tasks: 5,
            completed: 2,
            pending: 2,
            blocked: 1,
            failed: 0,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("MVP"));
    }
}
