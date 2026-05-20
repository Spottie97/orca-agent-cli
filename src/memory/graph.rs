use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::utils::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Project,
    Task,
    File,
    Concept,
    Decision,
    Error,
    Model,
    Provider,
    Skill,
    Hook,
    Rule,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Project => write!(f, "project"),
            NodeKind::Task => write!(f, "task"),
            NodeKind::File => write!(f, "file"),
            NodeKind::Concept => write!(f, "concept"),
            NodeKind::Decision => write!(f, "decision"),
            NodeKind::Error => write!(f, "error"),
            NodeKind::Model => write!(f, "model"),
            NodeKind::Provider => write!(f, "provider"),
            NodeKind::Skill => write!(f, "skill"),
            NodeKind::Hook => write!(f, "hook"),
            NodeKind::Rule => write!(f, "rule"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    DependsOn,
    Touches,
    Implements,
    AffectedBy,
    Caused,
    FixedBy,
    SucceededOn,
    FailedOn,
    Requires,
    Validates,
    Summarizes,
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeKind::DependsOn => write!(f, "depends_on"),
            EdgeKind::Touches => write!(f, "touches"),
            EdgeKind::Implements => write!(f, "implements"),
            EdgeKind::AffectedBy => write!(f, "affected_by"),
            EdgeKind::Caused => write!(f, "caused"),
            EdgeKind::FixedBy => write!(f, "fixed_by"),
            EdgeKind::SucceededOn => write!(f, "succeeded_on"),
            EdgeKind::FailedOn => write!(f, "failed_on"),
            EdgeKind::Requires => write!(f, "requires"),
            EdgeKind::Validates => write!(f, "validates"),
            EdgeKind::Summarizes => write!(f, "summarizes"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: impl Into<String>, kind: NodeKind, label: impl Into<String>) {
        self.nodes.push(Node {
            id: id.into(),
            kind,
            label: label.into(),
            metadata: HashMap::new(),
        });
    }

    pub fn add_edge(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: EdgeKind,
    ) {
        self.edges.push(Edge {
            source: source.into(),
            target: target.into(),
            kind,
            metadata: HashMap::new(),
        });
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let nodes_json = serde_json::to_string_pretty(&self.nodes)?;
        let edges_json = serde_json::to_string_pretty(&self.edges)?;
        fs::safe_write(&dir.join("nodes.json"), &nodes_json)?;
        fs::safe_write(&dir.join("edges.json"), &edges_json)?;
        Ok(())
    }

    pub fn load(dir: &Path) -> Result<Self> {
        let nodes_raw = std::fs::read_to_string(dir.join("nodes.json"))?;
        let edges_raw = std::fs::read_to_string(dir.join("edges.json"))?;
        let nodes: Vec<Node> = serde_json::from_str(&nodes_raw)?;
        let edges: Vec<Edge> = serde_json::from_str(&edges_raw)?;
        Ok(Self { nodes, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let mut graph = Graph::new();
        graph.add_node("TASK-001", NodeKind::Task, "Implement provider");
        graph.add_node("src/providers/cursor.rs", NodeKind::File, "Cursor provider");
        graph.add_edge("TASK-001", "src/providers/cursor.rs", EdgeKind::Touches);

        graph.save(tmp.path()).unwrap();
        let loaded = Graph::load(tmp.path()).unwrap();
        assert_eq!(loaded.nodes.len(), 2);
        assert_eq!(loaded.edges.len(), 1);
    }
}
