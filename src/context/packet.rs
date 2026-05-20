use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    pub task_id: String,
    pub task_title: String,
    pub task_type: String,
    pub goal: String,
    pub constraints: Vec<String>,
    pub relevant_files: Vec<FileRef>,
    pub architecture_notes: Vec<String>,
    pub risks: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub routing_recommendation: String,
    pub suggested_provider: String,
    pub test_commands: Vec<String>,
    pub memory_links: Vec<String>,
    pub prior_attempts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub path: String,
    pub reason: String,
}

impl ContextPacket {
    pub fn new(task_id: impl Into<String>, task_title: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            task_title: task_title.into(),
            task_type: "implementation".to_string(),
            goal: "".to_string(),
            constraints: Vec::new(),
            relevant_files: Vec::new(),
            architecture_notes: Vec::new(),
            risks: Vec::new(),
            acceptance_criteria: Vec::new(),
            routing_recommendation: "".to_string(),
            suggested_provider: "".to_string(),
            test_commands: Vec::new(),
            memory_links: Vec::new(),
            prior_attempts: Vec::new(),
        }
    }
}
