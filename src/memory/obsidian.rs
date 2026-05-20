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

/// Format a rich task completion entry for the memory store.
#[allow(clippy::too_many_arguments)]
pub fn format_task_completion_entry(
    task_id: &str,
    provider: &str,
    model: &str,
    status: &str,
    verdict: &str,
    output_summary: Option<&str>,
    duration_ms: Option<u64>,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    result_path: Option<&str>,
    patch_path: Option<&str>,
    review_path: Option<&str>,
    next_step: &str,
) -> String {
    let mut lines = vec![
        format!("## Task Completion: {}", task_id),
        String::new(),
        "### Execution".to_string(),
        format!("- **Provider**: {}", provider),
        format!("- **Model**: {}", model),
        format!("- **Status**: {}", status),
    ];

    if let Some(ms) = duration_ms {
        lines.push(format!("- **Duration**: {} ms", ms));
    }
    if let Some(t) = input_tokens {
        lines.push(format!("- **Input tokens**: {}", t));
    }
    if let Some(t) = output_tokens {
        lines.push(format!("- **Output tokens**: {}", t));
    }

    lines.push(String::new());
    lines.push("### Review".to_string());
    lines.push(format!("- **Verdict**: {}", verdict));
    lines.push(format!("- **Next step**: {}", next_step));

    if let Some(summary) = output_summary {
        let truncated = if summary.len() > 500 {
            format!("{}...", &summary[..500])
        } else {
            summary.to_string()
        };
        lines.push(String::new());
        lines.push("### Output Summary".to_string());
        lines.push(format!("```\n{}\n```", truncated));
    }

    let mut artifacts = Vec::new();
    if let Some(p) = result_path {
        artifacts.push(format!("- **Result**: {}", p));
    }
    if let Some(p) = patch_path {
        artifacts.push(format!("- **Patch**: {}", p));
    }
    if let Some(p) = review_path {
        artifacts.push(format!("- **Review**: {}", p));
    }
    if !artifacts.is_empty() {
        lines.push(String::new());
        lines.push("### Artifacts".to_string());
        lines.extend(artifacts);
    }

    lines.push(String::new());
    lines.join("\n")
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
