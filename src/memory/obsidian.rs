use std::collections::HashMap;

use anyhow::Result;

#[allow(clippy::too_many_arguments)]
pub fn write_task_note(
    task_id: &str,
    title: &str,
    status: &str,
    task_type: &str,
    complexity: &str,
    provider: &str,
    model: &str,
    files: &[impl AsRef<str>],
    summary: &str,
) -> Result<String> {
    let mut frontmatter = HashMap::new();
    frontmatter.insert("task_id".to_string(), task_id.to_string());
    frontmatter.insert("title".to_string(), title.to_string());
    frontmatter.insert("status".to_string(), status.to_string());
    frontmatter.insert("type".to_string(), task_type.to_string());
    frontmatter.insert("complexity".to_string(), complexity.to_string());
    frontmatter.insert("assigned_provider".to_string(), provider.to_string());
    frontmatter.insert("assigned_model".to_string(), model.to_string());

    let files_yaml = files
        .iter()
        .map(|f| format!("  - {}", f.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");

    let note = format!(
        r#"---
task_id: {}
title: {}
status: {}
type: {}
complexity: {}
assigned_provider: {}
assigned_model: {}
files_touched:
{}
---

# {} — {}

## Summary
{}
"#,
        task_id,
        title,
        status,
        task_type,
        complexity,
        provider,
        model,
        files_yaml,
        task_id,
        title,
        summary
    );

    Ok(note)
}

pub fn write_decision_note(
    decision_id: &str,
    title: &str,
    status: &str,
    context: &str,
    decision: &str,
    consequences: &[impl AsRef<str>],
) -> Result<String> {
    let consequences_list = consequences
        .iter()
        .map(|c| format!("- {}", c.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");

    let note = format!(
        r#"# {} — {}

## Status
{}

## Context
{}

## Decision
{}

## Consequences
{}
"#,
        decision_id, title, status, context, decision, consequences_list
    );

    Ok(note)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_note_format() {
        let note = write_task_note(
            "TASK-001",
            "Implement provider",
            "complete",
            "implementation",
            "medium",
            "cursor_sdk",
            "composer_2_5",
            &["src/providers/cursor.rs"],
            "Implemented cursor provider.",
        )
        .unwrap();
        assert!(note.contains("task_id: TASK-001"));
        assert!(note.contains("status: complete"));
        assert!(note.contains("Implemented cursor provider."));
    }

    #[test]
    fn test_decision_note_format() {
        let note = write_decision_note(
            "DEC-001",
            "Use context packets",
            "Accepted",
            "Chat histories are fragile.",
            "Use compact context packets.",
            &["Reduced token usage", "Easier model switching"],
        )
        .unwrap();
        assert!(note.contains("DEC-001"));
        assert!(note.contains("Accepted"));
        assert!(note.contains("Reduced token usage"));
    }
}
