use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ScanArgs {
    #[arg(long, help = "Path to scan (defaults to repo path)")]
    pub path: Option<PathBuf>,
}

pub fn run(_args: ScanArgs) -> Result<()> {
    println!("Scanning repository...");
    Ok(())
}
