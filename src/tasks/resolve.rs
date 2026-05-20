use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};

use crate::cli::commands::plan::{TaskGraph, TaskNode};
use crate::tasks::{Task, TaskComplexity, TaskRisk, TaskStatus, TaskType};

/// Load a task graph from a YAML file.
pub fn load_task_graph(path: &Path) -> Result<TaskGraph> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read task graph from {}", path.display()))?;
    let graph: TaskGraph = serde_yaml::from_str(&contents)
        .with_context(|| format!("Failed to parse task graph YAML from {}", path.display()))?;
    Ok(graph)
}

/// Resolve a task by ID from the task graph.
/// Returns `None` if the graph does not exist or the task ID is not found.
pub fn resolve_task_from_graph(graph_path: &Path, task_id: &str) -> Result<Option<Task>> {
    if !graph_path.exists() {
        return Ok(None);
    }
    let graph = load_task_graph(graph_path)?;
    let node = graph.tasks.into_iter().find(|t| t.id == task_id);
    Ok(node.map(task_node_to_task))
}

/// Convert a TaskNode (from YAML graph) into a Task (internal model).
pub fn task_node_to_task(node: TaskNode) -> Task {
    Task {
        id: node.id,
        title: node.title,
        description: node.description,
        task_type: TaskType::from_str(&node.task_type).unwrap_or(TaskType::Planning),
        complexity: TaskComplexity::from_str(&node.complexity).unwrap_or(TaskComplexity::Medium),
        risk: TaskRisk::from_str(&node.risk).unwrap_or(TaskRisk::Low),
        status: TaskStatus::from_str(&node.status).unwrap_or(TaskStatus::Pending),
        requires_repo_search: false,
        estimated_files_touched: node.dependencies.len() as u32 + 1,
        context_is_exact: node.dependencies.is_empty(),
        failure_count: 0,
        acceptance_criteria: Vec::new(),
    }
}

/// Create a minimal fallback task when the task graph does not contain the requested ID.
pub fn fallback_task(task_id: &str) -> Task {
    Task::new(task_id, format!("Task {}", task_id), TaskType::Planning)
}

/// Strictly resolve a task by ID from the task graph.
/// Returns an error if the graph exists but the task ID is not found.
pub fn require_task_from_graph(graph_path: &Path, task_id: &str) -> Result<Task> {
    if !graph_path.exists() {
        return Err(anyhow::anyhow!(
            "task graph not found at {}",
            graph_path.display()
        ));
    }
    let graph = load_task_graph(graph_path)?;
    match graph.tasks.into_iter().find(|t| t.id == task_id) {
        Some(node) => Ok(task_node_to_task(node)),
        None => Err(anyhow::anyhow!(
            "task {} not found in {}",
            task_id,
            graph_path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_task_graph() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmpfile,
            r#"
version: "1.0"
project: Test
tasks:
  - id: TASK-001
    title: Define structure
    description: Set up dirs.
    task_type: planning
    complexity: low
    risk: low
    dependencies: []
    status: pending
"#
        )
        .unwrap();
        let graph = load_task_graph(tmpfile.path()).unwrap();
        assert_eq!(graph.tasks.len(), 1);
        assert_eq!(graph.tasks[0].id, "TASK-001");
        assert_eq!(graph.tasks[0].task_type, "planning");
    }

    #[test]
    fn test_resolve_task_from_graph_found() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmpfile,
            r#"
version: "1.0"
project: Test
tasks:
  - id: TASK-001
    title: Define structure
    description: Set up dirs.
    task_type: planning
    complexity: low
    risk: low
    dependencies: []
    status: pending
  - id: TASK-002
    title: Implement core
    description: Build feature.
    task_type: implementation
    complexity: medium
    risk: medium
    dependencies:
      - TASK-001
    status: in-progress
"#
        )
        .unwrap();
        let task = resolve_task_from_graph(tmpfile.path(), "TASK-002").unwrap();
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.id, "TASK-002");
        assert_eq!(task.task_type, TaskType::Implementation);
        assert_eq!(task.complexity, TaskComplexity::Medium);
        assert_eq!(task.risk, TaskRisk::Medium);
    }

    #[test]
    fn test_resolve_task_from_graph_not_found() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmpfile,
            r#"
version: "1.0"
project: Test
tasks:
  - id: TASK-001
    title: Define structure
    task_type: planning
    complexity: low
    risk: low
    dependencies: []
    status: pending
"#
        )
        .unwrap();
        let task = resolve_task_from_graph(tmpfile.path(), "TASK-999").unwrap();
        assert!(task.is_none());
    }

    #[test]
    fn test_task_type_from_str() {
        assert_eq!(TaskType::from_str("planning"), Ok(TaskType::Planning));
        assert_eq!(
            TaskType::from_str("implementation"),
            Ok(TaskType::Implementation)
        );
        assert_eq!(TaskType::from_str("tests"), Ok(TaskType::Tests));
        assert_eq!(TaskType::from_str("UNKNOWN"), Err(()));
    }

    #[test]
    fn test_complexity_from_str() {
        assert_eq!(
            TaskComplexity::from_str("medium"),
            Ok(TaskComplexity::Medium)
        );
        assert_eq!(
            TaskComplexity::from_str("critical"),
            Ok(TaskComplexity::Critical)
        );
        assert_eq!(TaskComplexity::from_str("UNKNOWN"), Err(()));
    }

    #[test]
    fn test_risk_from_str() {
        assert_eq!(TaskRisk::from_str("high"), Ok(TaskRisk::High));
        assert_eq!(TaskRisk::from_str("medium"), Ok(TaskRisk::Medium));
        assert_eq!(TaskRisk::from_str("UNKNOWN"), Err(()));
    }
}
