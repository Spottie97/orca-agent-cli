use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub project_name: String,
    pub current_phase: String,
    pub tasks: HashMap<String, TaskState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub status: String,
    #[serde(default)]
    pub failure_count: u32,
    #[serde(default)]
    pub assigned_provider: Option<String>,
    #[serde(default)]
    pub assigned_model: Option<String>,
    #[serde(default)]
    pub context_packet_path: Option<String>,
    #[serde(default)]
    pub result_path: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            project_name: "Unnamed Project".to_string(),
            current_phase: "MVP".to_string(),
            tasks: HashMap::new(),
        }
    }
}

pub fn load(path: &Path) -> Result<State> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read state file: {}", path.display()))?;
    let state: State = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse state file: {}", path.display()))?;
    Ok(state)
}

pub fn save(state: &State, path: &Path) -> Result<()> {
    let raw = serde_json::to_string_pretty(state)?;
    crate::utils::fs::safe_write(path, &raw)?;
    Ok(())
}
