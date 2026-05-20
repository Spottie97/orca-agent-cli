use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
}

pub fn run(_args: StatusArgs) -> Result<()> {
    println!("Showing project status...");
    Ok(())
}
