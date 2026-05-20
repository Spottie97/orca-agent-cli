use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ExecuteArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,

    #[arg(long, help = "Provider to use")]
    pub provider: Option<String>,
}

pub fn run(_args: ExecuteArgs) -> Result<()> {
    println!("Executing task...");
    Ok(())
}
