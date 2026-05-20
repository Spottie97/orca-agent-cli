use anyhow::Result;
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

pub fn run(cmd: ConfigCmd) -> Result<()> {
    match cmd.command {
        ConfigSubcommand::Validate => {
            println!("Validating Orca configuration...");
        }
        ConfigSubcommand::Show => {
            println!("Showing Orca configuration...");
        }
    }
    Ok(())
}
