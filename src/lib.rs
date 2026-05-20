pub mod approvals;
pub mod cli;
pub mod config;
pub mod context;
pub mod memory;
pub mod prompts;
pub mod providers;
pub mod review;
pub mod router;
pub mod state;
pub mod tasks;
pub mod utils;

use anyhow::Result;

pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();
    cli::run()
}
