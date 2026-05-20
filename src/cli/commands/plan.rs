use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::config::schema::Config;
use crate::utils::fs::safe_write;

#[derive(Args)]
pub struct PlanArgs {
    #[arg(long, help = "Planner to use (claude, ollama, manual)")]
    pub planner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub task_type: String,
    #[serde(default)]
    pub complexity: String,
    #[serde(default)]
    pub risk: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    pub version: String,
    pub project: String,
    #[serde(default)]
    pub tasks: Vec<TaskNode>,
}

impl TaskGraph {
    pub fn new(project: impl Into<String>) -> Self {
        Self {
            version: "1.0".to_string(),
            project: project.into(),
            tasks: Vec::new(),
        }
    }

    pub fn manual_skeleton() -> Self {
        let mut graph = Self::new("Unnamed Project");
        graph.tasks = vec![
            TaskNode {
                id: "TASK-001".to_string(),
                title: "Define project structure".to_string(),
                description: "Set up directories and config.".to_string(),
                task_type: "planning".to_string(),
                complexity: "low".to_string(),
                risk: "low".to_string(),
                dependencies: Vec::new(),
                status: "pending".to_string(),
            },
            TaskNode {
                id: "TASK-002".to_string(),
                title: "Implement core feature".to_string(),
                description: "Build the main functionality.".to_string(),
                task_type: "implementation".to_string(),
                complexity: "medium".to_string(),
                risk: "medium".to_string(),
                dependencies: vec!["TASK-001".to_string()],
                status: "pending".to_string(),
            },
            TaskNode {
                id: "TASK-003".to_string(),
                title: "Add tests and review".to_string(),
                description: "Write tests and review output.".to_string(),
                task_type: "tests".to_string(),
                complexity: "medium".to_string(),
                risk: "low".to_string(),
                dependencies: vec!["TASK-002".to_string()],
                status: "pending".to_string(),
            },
        ];
        graph
    }
}

pub fn run(args: PlanArgs) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let planner = args.planner.as_deref().unwrap_or("manual");
    let graph = if planner == "manual" {
        TaskGraph::manual_skeleton()
    } else {
        // For non-manual planners, generate a skeleton with project name from config
        let mut graph = TaskGraph::new(&config.project.name);
        graph.tasks.push(TaskNode {
            id: "TASK-001".to_string(),
            title: "Plan generated task".to_string(),
            description: format!("Planner: {}", planner),
            task_type: "planning".to_string(),
            complexity: "medium".to_string(),
            risk: "low".to_string(),
            dependencies: Vec::new(),
            status: "pending".to_string(),
        });
        graph
    };

    let output_path = config.project.orca_dir.join("task-graph.yaml");
    let yaml =
        serde_yaml::to_string(&graph).with_context(|| "Failed to serialize task graph to YAML")?;
    safe_write(&output_path, &yaml)
        .with_context(|| format!("Failed to write task graph to {}", output_path.display()))?;

    println!(
        "Task graph written to {} ({} tasks)",
        output_path.display(),
        graph.tasks.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_skeleton_has_tasks() {
        let graph = TaskGraph::manual_skeleton();
        assert_eq!(graph.tasks.len(), 3);
        assert_eq!(graph.tasks[0].id, "TASK-001");
        assert!(!graph.tasks[1].dependencies.is_empty());
    }

    #[test]
    fn test_task_graph_roundtrip_yaml() {
        let mut graph = TaskGraph::new("Test");
        graph.tasks.push(TaskNode {
            id: "T1".to_string(),
            title: "Test task".to_string(),
            description: "".to_string(),
            task_type: "implementation".to_string(),
            complexity: "low".to_string(),
            risk: "low".to_string(),
            dependencies: Vec::new(),
            status: "pending".to_string(),
        });
        let yaml = serde_yaml::to_string(&graph).unwrap();
        let parsed: TaskGraph = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.tasks.len(), 1);
        assert_eq!(parsed.tasks[0].id, "T1");
    }
}
