use std::str::FromStr;

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

impl FromStr for TaskType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "summary" => Ok(TaskType::Summary),
            "compression" => Ok(TaskType::Compression),
            "memory-update" => Ok(TaskType::MemoryUpdate),
            "memoryupdate" => Ok(TaskType::MemoryUpdate),
            "docs" => Ok(TaskType::Docs),
            "architecture" => Ok(TaskType::Architecture),
            "decomposition" => Ok(TaskType::Decomposition),
            "risk-analysis" => Ok(TaskType::RiskAnalysis),
            "riskanalysis" => Ok(TaskType::RiskAnalysis),
            "planning" => Ok(TaskType::Planning),
            "implementation" => Ok(TaskType::Implementation),
            "tests" => Ok(TaskType::Tests),
            "refactor" => Ok(TaskType::Refactor),
            "review" => Ok(TaskType::Review),
            "research" => Ok(TaskType::Research),
            "debugging" => Ok(TaskType::Debugging),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
    Critical,
}

impl FromStr for TaskComplexity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TaskComplexity::Low),
            "medium" => Ok(TaskComplexity::Medium),
            "high" => Ok(TaskComplexity::High),
            "critical" => Ok(TaskComplexity::Critical),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskRisk {
    Low,
    Medium,
    High,
}

impl FromStr for TaskRisk {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TaskRisk::Low),
            "medium" => Ok(TaskRisk::Medium),
            "high" => Ok(TaskRisk::High),
            _ => Err(()),
        }
    }
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

impl FromStr for TaskStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TaskStatus::Pending),
            "in-progress" => Ok(TaskStatus::InProgress),
            "inprogress" => Ok(TaskStatus::InProgress),
            "complete" => Ok(TaskStatus::Complete),
            "failed" => Ok(TaskStatus::Failed),
            "blocked" => Ok(TaskStatus::Blocked),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(()),
        }
    }
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
