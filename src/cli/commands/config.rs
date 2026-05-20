use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct ConfigCmd {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    #[command(about = "Validate the Orca configuration")]
    Validate,

    #[command(about = "Show the current configuration")]
    Show,
}

fn default_config_path() -> PathBuf {
    PathBuf::from(".orca").join("config.yaml")
}

pub fn run(cmd: ConfigCmd) -> Result<()> {
    let config_path = default_config_path();

    match cmd.command {
        ConfigSubcommand::Validate => {
            validate(&config_path)?;
        }
        ConfigSubcommand::Show => {
            show(&config_path)?;
        }
    }
    Ok(())
}

fn validate(path: &std::path::Path) -> Result<()> {
    let config = crate::config::load(path)
        .with_context(|| format!("Config validation failed for {}", path.display()))?;
    println!("Configuration is valid.");
    println!("Project: {}", config.project.name);
    println!("Repo: {}", config.project.repo_path.display());
    Ok(())
}

fn show(path: &std::path::Path) -> Result<()> {
    let config = crate::config::load(path)
        .with_context(|| format!("Failed to load config from {}", path.display()))?;
    let yaml = serde_yaml::to_string(&config)?;
    print!("{}", yaml);
    Ok(())
}
