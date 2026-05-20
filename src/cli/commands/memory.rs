use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct MemoryCmd {
    #[command(subcommand)]
    pub command: MemorySubcommand,
}

#[derive(Subcommand)]
pub enum MemorySubcommand {
    #[command(about = "Update project memory after a task")]
    Update {
        #[arg(help = "Task ID")]
        task_id: String,
    },
}

pub fn run(cmd: MemoryCmd) -> Result<()> {
    match cmd.command {
        MemorySubcommand::Update { task_id } => {
            println!("Updating project memory for task {}...", task_id);
        }
    }
    Ok(())
}
