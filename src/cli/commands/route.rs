use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct RouteArgs {
    #[arg(help = "Task ID")]
    pub task_id: String,
}

pub fn run(_args: RouteArgs) -> Result<()> {
    println!("Computing routing decision...");
    Ok(())
}
