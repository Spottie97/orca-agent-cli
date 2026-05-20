pub mod commands;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orca")]
#[command(version)]
#[command(about = "Orca Agent CLI — AI coding orchestration tool")]
#[command(long_about = r#"
Orca Agent CLI is a command-line orchestration tool for coordinating multiple
AI coding and reasoning systems while minimizing token usage and avoiding
context loss.

It learns the project, not the user.
"#)]
pub struct Cli {
    #[arg(short, long, value_name = "PATH", help = "Path to config file")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Dry run mode")]
    pub dry_run: bool,

    #[arg(long, global = true, help = "Output in JSON format")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new Orca project workspace")]
    Init(commands::init::InitArgs),

    #[command(about = "Configuration commands")]
    Config(commands::config::ConfigCmd),

    #[command(about = "Scan the repository and update project memory")]
    Scan(commands::scan::ScanArgs),

    #[command(about = "Generate a context packet for a task")]
    Context(commands::context::ContextArgs),

    #[command(about = "Show routing decision for a task")]
    Route(commands::route::RouteArgs),

    #[command(about = "Generate a task graph from project plan")]
    Plan(commands::plan::PlanArgs),

    #[command(about = "Execute a task through a provider")]
    Execute(commands::execute::ExecuteArgs),

    #[command(about = "Review a task result")]
    Review(commands::review::ReviewArgs),

    #[command(about = "Memory management commands")]
    Memory(commands::memory::MemoryCmd),

    #[command(about = "Run the full orchestration loop for a task")]
    Run(commands::run::RunArgs),

    #[command(about = "Show project status")]
    Status(commands::status::StatusArgs),
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => commands::init::run(args),
        Commands::Config(cmd) => commands::config::run(cmd),
        Commands::Scan(args) => commands::scan::run(args),
        Commands::Context(args) => commands::context::run(args),
        Commands::Route(args) => commands::route::run(args),
        Commands::Plan(args) => commands::plan::run(args),
        Commands::Execute(args) => commands::execute::run(args, cli.dry_run),
        Commands::Review(args) => commands::review::run(args),
        Commands::Memory(cmd) => commands::memory::run(cmd),
        Commands::Run(args) => commands::run::run(args),
        Commands::Status(args) => commands::status::run(args),
    }
}
