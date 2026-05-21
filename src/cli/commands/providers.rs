use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::config::schema::Config;

#[derive(Args)]
pub struct ProvidersArgs {
    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiag {
    pub name: String,
    pub enabled: bool,
    pub command: Option<String>,
    pub command_on_path: bool,
    pub model: Option<String>,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersSummary {
    pub providers: Vec<ProviderDiag>,
}

pub fn run(args: ProvidersArgs, dry_run: bool) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let mut providers = Vec::new();

    let claude_cmd = config.models.claude_code.command.clone();
    providers.push(ProviderDiag {
        name: "claude_code".to_string(),
        enabled: config.models.claude_code.enabled,
        command_on_path: command_on_path(claude_cmd.as_deref()),
        command: claude_cmd,
        model: config.models.claude_code.model.clone(),
        requires_approval: config.models.claude_code.requires_approval,
    });

    let codex_cmd = config.models.codex.command.clone();
    providers.push(ProviderDiag {
        name: "codex".to_string(),
        enabled: config.models.codex.enabled,
        command_on_path: command_on_path(codex_cmd.as_deref()),
        command: codex_cmd,
        model: config
            .models
            .codex
            .model
            .clone()
            .or(Some(config.models.codex.default_model.clone())),
        requires_approval: config.models.codex.requires_approval,
    });

    let cursor_cmd = config.models.cursor.command.clone();
    providers.push(ProviderDiag {
        name: "cursor".to_string(),
        enabled: config.models.cursor.enabled,
        command_on_path: command_on_path(cursor_cmd.as_deref()),
        command: cursor_cmd,
        model: config
            .models
            .cursor
            .model
            .clone()
            .or(Some(config.models.cursor.composer_model_id.clone())),
        requires_approval: config.models.cursor.requires_approval,
    });

    let summary = ProvidersSummary { providers };

    if args.json {
        let json = serde_json::to_string_pretty(&summary)
            .with_context(|| "Failed to serialize providers to JSON")?;
        println!("{}", json);
    } else {
        println!("Bridge Provider Diagnostics");
        println!("============================");
        if dry_run {
            println!("(dry-run mode — no commands executed)");
        }
        println!();
        for p in &summary.providers {
            println!("Provider: {}", p.name);
            println!("  Enabled:           {}", p.enabled);
            println!(
                "  Command:           {}",
                p.command.as_deref().unwrap_or("(not configured)")
            );
            println!(
                "  Command on PATH:   {}",
                if p.command_on_path { "yes" } else { "no" }
            );
            println!(
                "  Model:             {}",
                p.model.as_deref().unwrap_or("(default)")
            );
            println!(
                "  Requires approval: {}",
                if p.requires_approval { "yes" } else { "no" }
            );
            println!();
        }
    }

    Ok(())
}

fn command_on_path(cmd: Option<&str>) -> bool {
    let cmd = match cmd {
        Some(c) if !c.is_empty() => c,
        _ => return false,
    };

    let path_var = match std::env::var("PATH") {
        Ok(v) => v,
        Err(_) => return false,
    };

    let separator = if cfg!(windows) { ';' } else { ':' };
    for dir in path_var.split(separator) {
        let base = PathBuf::from(dir);
        if base.join(cmd).is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            let with_exe = format!("{}.exe", cmd);
            if base.join(&with_exe).is_file() {
                return true;
            }
            let with_cmd = format!("{}.cmd", cmd);
            if base.join(&with_cmd).is_file() {
                return true;
            }
            let with_bat = format!("{}.bat", cmd);
            if base.join(&with_bat).is_file() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_on_path_with_real_command() {
        // cmd exists on Windows PATH; echo/cat exist on Unix
        let cmd = if cfg!(windows) { "cmd" } else { "echo" };
        assert!(command_on_path(Some(cmd)));
    }

    #[test]
    fn test_command_on_path_missing() {
        assert!(!command_on_path(Some("definitely_not_real_12345")));
    }

    #[test]
    fn test_command_on_path_none() {
        assert!(!command_on_path(None));
    }

    #[test]
    fn test_command_on_path_empty() {
        assert!(!command_on_path(Some("")));
    }

    #[test]
    fn test_providers_summary_serialization() {
        let summary = ProvidersSummary {
            providers: vec![ProviderDiag {
                name: "claude_code".to_string(),
                enabled: true,
                command: Some("claude".to_string()),
                command_on_path: true,
                model: Some("claude-code".to_string()),
                requires_approval: true,
            }],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("claude_code"));
        assert!(json.contains("claude"));
    }
}
