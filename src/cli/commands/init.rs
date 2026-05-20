use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::config::schema::{
    Config, ContextConfig, ExecutionConfig, MemoryConfig, ModelsConfig, ProjectConfig,
    RoutingConfig,
};
use crate::prompts::write_prompts_to_dir;
use crate::state::State;
use crate::utils::fs;

#[derive(Args)]
pub struct InitArgs {
    #[arg(long, help = "Project name")]
    pub project_name: Option<String>,

    #[arg(long, help = "Path to the repository")]
    pub repo: Option<PathBuf>,

    #[arg(long, help = "Path to the Obsidian vault")]
    pub vault: Option<PathBuf>,
}

pub fn run(args: InitArgs, dry_run: bool) -> Result<()> {
    let repo_path = args.repo.unwrap_or_else(|| PathBuf::from("."));
    let repo_path = std::fs::canonicalize(&repo_path).unwrap_or(repo_path);

    let project_name = args.project_name.unwrap_or_else(|| {
        repo_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "orca-project".to_string())
    });

    let orca_dir = repo_path.join(".orca");

    if dry_run {
        println!(
            "Dry run: would initialize Orca project workspace at {}",
            orca_dir.display()
        );
        println!(
            "  - Create directories: input, context-packets, prompts, results, patches, memory, graph, logs"
        );
        println!("  - Write config.yaml with project name: {}", project_name);
        println!("  - Write state.json with initial state");
        println!("  - Write prompt library markdown files");
        return Ok(());
    }

    fs::ensure_dir(&orca_dir)?;
    fs::ensure_dir(&orca_dir.join("input"))?;
    fs::ensure_dir(&orca_dir.join("context-packets"))?;
    fs::ensure_dir(&orca_dir.join("prompts"))?;
    fs::ensure_dir(&orca_dir.join("results"))?;
    fs::ensure_dir(&orca_dir.join("patches"))?;
    fs::ensure_dir(&orca_dir.join("memory"))?;
    fs::ensure_dir(&orca_dir.join("graph"))?;
    fs::ensure_dir(&orca_dir.join("logs"))?;

    let config = Config {
        project: ProjectConfig {
            name: project_name.clone(),
            repo_path: repo_path.clone(),
            vault_path: args.vault,
            orca_dir: orca_dir.clone(),
        },
        execution: ExecutionConfig::default(),
        models: ModelsConfig::default(),
        routing: RoutingConfig::default(),
        context: ContextConfig::default(),
        memory: MemoryConfig::default(),
    };

    let config_yaml = serde_yaml::to_string(&config)?;
    fs::safe_write(&orca_dir.join("config.yaml"), &config_yaml)?;

    let state = State {
        project_name,
        current_phase: "MVP".to_string(),
        tasks: HashMap::new(),
    };

    let state_json = serde_json::to_string_pretty(&state)?;
    fs::safe_write(&orca_dir.join("state.json"), &state_json)?;

    write_prompts_to_dir(&orca_dir.join("prompts"))?;

    println!(
        "Initialized Orca project workspace at {}",
        orca_dir.display()
    );
    Ok(())
}
