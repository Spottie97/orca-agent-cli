use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::config::schema::Config;
use crate::memory::MemoryStore;
use crate::state::{self, State, TaskState};

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

fn simple_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let since = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since.as_secs();
    // Format as YYYY-MM-DDTHH:MM:SSZ (UTC, no leap-second handling)
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    // Approximate year/month/day from days since epoch (sufficient for logs)
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    )
}

pub fn run(cmd: MemoryCmd) -> Result<()> {
    match cmd.command {
        MemorySubcommand::Update { task_id } => {
            let config_path = PathBuf::from(".orca").join("config.yaml");
            let config: Config = crate::config::load(&config_path)
                .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

            let orca_dir = &config.project.orca_dir;
            let state_path = orca_dir.join("state.json");

            let mut state = if state_path.exists() {
                state::load(&state_path)?
            } else {
                State::default()
            };

            // Update task state
            let now = simple_timestamp();
            let task_state = state
                .tasks
                .entry(task_id.clone())
                .or_insert_with(|| TaskState {
                    status: "complete".to_string(),
                    failure_count: 0,
                    assigned_provider: None,
                    assigned_model: None,
                    context_packet_path: None,
                    result_path: None,
                    updated_at: Some(now.clone()),
                });
            task_state.status = "complete".to_string();
            task_state.updated_at = Some(now.clone());

            // Save state atomically
            state::save(&state, &state_path).with_context(|| "Failed to save state")?;

            // Append to task history in memory store
            let store = MemoryStore::new(orca_dir);
            let history_entry = format!(
                "\n## Update {}\n\n- Status: complete\n- Task: {}\n\n",
                now, task_id
            );
            store.append_to_note("tasks", &task_id, &history_entry)?;

            println!("Memory updated for task {}.", task_id);
            println!("State saved to {}", state_path.display());
        }
    }
    Ok(())
}
