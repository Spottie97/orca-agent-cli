use anyhow::Result;

use super::packet::ContextPacket;

pub fn render_markdown(packet: &ContextPacket) -> String {
    let mut md = String::new();
    md.push_str(&format!(
        "# Context Packet: {} — {}\n\n",
        packet.task_id, packet.task_title
    ));

    md.push_str(&format!(
        "## Task\n- **ID**: {}\n- **Title**: {}\n- **Type**: {}\n\n",
        packet.task_id, packet.task_title, packet.task_type
    ));

    if !packet.goal.is_empty() {
        md.push_str(&format!("## Goal\n{}\n\n", packet.goal));
    }

    if !packet.constraints.is_empty() {
        md.push_str("## Constraints\n");
        for c in &packet.constraints {
            md.push_str(&format!("- {}\n", c));
        }
        md.push('\n');
    }

    if !packet.relevant_files.is_empty() {
        md.push_str("## Relevant Files\n");
        for f in &packet.relevant_files {
            md.push_str(&format!("- `{}`: {}\n", f.path, f.reason));
        }
        md.push('\n');
    }

    if !packet.architecture_notes.is_empty() {
        md.push_str("## Architecture Notes\n");
        for n in &packet.architecture_notes {
            md.push_str(&format!("- {}\n", n));
        }
        md.push('\n');
    }

    if !packet.risks.is_empty() {
        md.push_str("## Risks\n");
        for r in &packet.risks {
            md.push_str(&format!("- {}\n", r));
        }
        md.push('\n');
    }

    if !packet.acceptance_criteria.is_empty() {
        md.push_str("## Acceptance Criteria\n");
        for ac in &packet.acceptance_criteria {
            md.push_str(&format!("- {}\n", ac));
        }
        md.push('\n');
    }

    if !packet.routing_recommendation.is_empty() {
        md.push_str(&format!(
            "## Routing Recommendation\n{}\n\n",
            packet.routing_recommendation
        ));
    }

    if !packet.suggested_provider.is_empty() {
        md.push_str(&format!(
            "## Suggested Provider\n{}\n\n",
            packet.suggested_provider
        ));
    }

    if !packet.test_commands.is_empty() {
        md.push_str("## Suggested Tests\n");
        for t in &packet.test_commands {
            md.push_str(&format!("- `{}`\n", t));
        }
        md.push('\n');
    }

    if !packet.memory_links.is_empty() {
        md.push_str("## Memory Links\n");
        for m in &packet.memory_links {
            md.push_str(&format!("- {}\n", m));
        }
        md.push('\n');
    }

    if !packet.prior_attempts.is_empty() {
        md.push_str("## Prior Attempts\n");
        for a in &packet.prior_attempts {
            md.push_str(&format!("- {}\n", a));
        }
        md.push('\n');
    }

    md
}

pub fn render_json(packet: &ContextPacket) -> Result<String> {
    Ok(serde_json::to_string_pretty(packet)?)
}

#[cfg(test)]
mod tests {
    use super::super::packet::FileRef;
    use super::*;

    #[test]
    fn test_render_markdown_contains_sections() {
        let mut packet = ContextPacket::new("TASK-001", "Test Task");
        packet.goal = "Implement feature".to_string();
        packet.constraints.push("Use Rust".to_string());
        packet.relevant_files.push(FileRef {
            path: "src/main.rs".to_string(),
            reason: "Entry point".to_string(),
        });
        packet.acceptance_criteria.push("Tests pass".to_string());
        let md = render_markdown(&packet);
        assert!(md.contains("Context Packet: TASK-001"));
        assert!(md.contains("Implement feature"));
        assert!(md.contains("src/main.rs"));
        assert!(md.contains("Tests pass"));
    }

    #[test]
    fn test_render_json_valid() {
        let packet = ContextPacket::new("TASK-001", "Test Task");
        let json = render_json(&packet).unwrap();
        assert!(json.contains("TASK-001"));
        assert!(json.contains("Test Task"));
    }
}
