use std::path::Path;

use anyhow::{ensure, Result};

use super::schema::Config;

pub fn validate(config: &Config) -> Result<()> {
    ensure!(
        !config.project.name.trim().is_empty(),
        "Project name cannot be empty"
    );

    ensure!(
        Path::new(&config.project.repo_path).exists(),
        "Repository path '{}' does not exist",
        config.project.repo_path.display()
    );

    ensure!(
        Path::new(&config.project.repo_path).is_dir(),
        "Repository path '{}' is not a directory",
        config.project.repo_path.display()
    );

    if let Some(ref vault) = config.project.vault_path {
        ensure!(
            Path::new(vault).exists(),
            "Vault path '{}' does not exist",
            vault.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        Config, ContextConfig, ExecutionConfig, MemoryConfig, ModelsConfig, ProjectConfig,
        RoutingConfig,
    };

    fn valid_config() -> Config {
        Config {
            project: ProjectConfig {
                name: "Test Project".to_string(),
                repo_path: PathBuf::from("."),
                vault_path: None,
                orca_dir: PathBuf::from(".orca"),
            },
            execution: ExecutionConfig::default(),
            models: ModelsConfig::default(),
            routing: RoutingConfig::default(),
            context: ContextConfig::default(),
            memory: MemoryConfig::default(),
        }
    }

    #[test]
    fn test_valid_config() {
        let config = valid_config();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_empty_project_name() {
        let mut config = valid_config();
        config.project.name = "".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_missing_repo_path() {
        let mut config = valid_config();
        config.project.repo_path = PathBuf::from("/definitely/does/not/exist");
        assert!(validate(&config).is_err());
    }
}
