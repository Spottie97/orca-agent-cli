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
    pub requires_repo_search: bool,
    #[serde(default)]
    pub estimated_files_touched: u32,
    #[serde(default)]
    pub context_is_exact: bool,
    #[serde(default)]
    pub failure_count: u32,
}
