use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Summary,
    Compression,
    MemoryUpdate,
    Docs,
    Architecture,
    Decomposition,
    RiskAnalysis,
    Planning,
    Implementation,
    Tests,
    Refactor,
    Review,
    Research,
    Debugging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Complete,
    Failed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub task_type: TaskType,
    pub complexity: TaskComplexity,
    pub risk: TaskRisk,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(default)]
    pub requires_repo_search: bool,
    #[serde(default)]
    pub estimated_files_touched: u32,
    #[serde(default)]
    pub context_is_exact: bool,
    #[serde(default)]
    pub failure_count: u32,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

impl Task {
    pub fn new(id: impl Into<String>, title: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
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
        }
    }
}
