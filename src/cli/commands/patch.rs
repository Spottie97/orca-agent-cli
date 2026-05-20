use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::config::schema::Config;

#[derive(Parser)]
pub struct PatchCmd {
    #[command(subcommand)]
    pub command: PatchSubcommand,
}

#[derive(Subcommand)]
pub enum PatchSubcommand {
    #[command(about = "Show patch proposal for a task")]
    Show(PatchShowArgs),

    #[command(about = "List all patch proposals")]
    List,
}

#[derive(Args)]
pub struct PatchShowArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,
}

pub fn run(cmd: PatchCmd) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let orca_dir = &config.project.orca_dir;

    match cmd.command {
        PatchSubcommand::Show(args) => {
            let proposal = crate::patch::load_patch_proposal(orca_dir, &args.task_id)
                .with_context(|| format!("No patch proposal found for task {}", args.task_id))?;

            println!("Patch proposal for {}:", args.task_id);
            println!("  Provider: {}", proposal.provider);
            println!("  Model: {}", proposal.model);
            println!("  Files changed: {}", proposal.files_changed.len());
            for change in &proposal.files_changed {
                println!("    - {}", change.path);
                if let Some(ref explanation) = change.explanation {
                    println!("      {}", explanation);
                }
            }
            if let Some(ref summary) = proposal.summary {
                println!("  Summary: {}", summary);
            }
        }
        PatchSubcommand::List => {
            let ids = crate::patch::list_patch_proposals(orca_dir)?;
            if ids.is_empty() {
                println!("No patch proposals found.");
            } else {
                println!("Patch proposals:");
                for id in ids {
                    println!("  - {}", id);
                }
            }
        }
    }

    Ok(())
}
