use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Provider to use")]
    pub provider: Option<String>,
}

pub fn run(_args: RunArgs) -> Result<()> {
    println!("Running full orchestration loop...");
    Ok(())
}
