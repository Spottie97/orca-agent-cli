pub mod loader;
pub mod schema;
pub mod validation;

use std::path::Path;

use anyhow::Result;

pub use schema::Config;

pub fn load(path: &Path) -> Result<Config> {
    let config = loader::load(path)?;
    validation::validate(&config)?;
    Ok(config)
}
