use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    #[arg(long, help = "Project name")]
    pub project_name: Option<String>,

    #[arg(long, help = "Path to the repository")]
    pub repo: Option<PathBuf>,

    #[arg(long, help = "Path to the Obsidian vault")]
    pub vault: Option<PathBuf>,
}

pub fn run(_args: InitArgs) -> Result<()> {
    println!("Initializing Orca project workspace...");
    Ok(())
}
