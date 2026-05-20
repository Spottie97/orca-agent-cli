use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::config::schema::Config;
use crate::memory::MemoryStore;
use crate::scan::{render_scan_summary, scan_repo};

#[derive(Args)]
pub struct ScanArgs {
    #[arg(long, help = "Path to scan (defaults to repo path)")]
    pub path: Option<PathBuf>,
}

pub fn run(args: ScanArgs) -> Result<()> {
    let config_path = PathBuf::from(".orca").join("config.yaml");
    let config: Config = crate::config::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    let scan_path = args
        .path
        .clone()
        .unwrap_or_else(|| config.project.repo_path.clone());

    println!("Scanning {} ...", scan_path.display());
    let result =
        scan_repo(&scan_path).with_context(|| format!("Failed to scan {}", scan_path.display()))?;

    let summary = render_scan_summary(&result);

    // Write to memory
    let store = MemoryStore::new(&config.project.orca_dir);
    store.write_note("scans", "latest", &summary)?;

    println!(
        "Found {} files in {} directories. Project type: {}",
        result.total_files, result.total_dirs, result.project_type
    );
    println!(
        "Scan summary written to {}",
        config
            .project
            .orca_dir
            .join("scans")
            .join("latest.md")
            .display()
    );

    Ok(())
}
