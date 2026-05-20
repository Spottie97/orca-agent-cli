use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct PlanArgs {
    #[arg(long, help = "Planner to use (claude, ollama, manual)")]
    pub planner: Option<String>,
}

pub fn run(_args: PlanArgs) -> Result<()> {
    println!("Generating task graph...");
    Ok(())
}
