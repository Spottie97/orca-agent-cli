use std::io::Write;

use orca_agent_cli::config::schema::Config;

#[test]
fn test_config_backward_compat_without_openai_field() {
    // Older configs generated before Phase 2 may not have the `openai` section.
    let yaml = r#"
project:
  name: legacy-project
  repo_path: .
  orca_dir: .orca
execution:
  default_mode: manual
  require_approval_for_premium: true
  require_clean_git_for_cursor: true
models:
  ollama:
    enabled: true
    base_url: "http://localhost:11434"
    default_model: qwen3.5:4b
    role: context_and_memory
  claude:
    enabled: true
    provider: claude_code
    default_model: opus-4.7
    role: planning
  codex:
    enabled: true
    provider: codex
    default_model: gpt-5.5
    role: scoped_execution
  cursor:
    enabled: true
    composer_model_id: composer-2.5
    premium_model_id: cursor-premium
    provider: cursor_sdk
    default_runtime: local
    cloud_auto_create_pr: false
routing:
  default_repo_executor: cursor_composer
  default_scoped_executor: codex
  default_planner: claude
  default_summarizer: ollama
  max_failures_before_escalation: 2
context:
  default_max_tokens: 3000
  hard_max_tokens: 8000
  prefer_summaries_over_raw_files: true
memory:
  update_after_each_task: true
  write_obsidian: true
  write_graph: true
"#;

    let parsed: Config = serde_yaml::from_str(yaml).expect("legacy config should parse");
    assert!(parsed.models.openai.enabled);
    assert_eq!(parsed.models.openai.base_url, "http://localhost:8080");
    assert_eq!(parsed.models.openai.default_model, "llama3.1");
}

#[test]
fn test_config_with_openai_section() {
    let yaml = r#"
project:
  name: test-project
  repo_path: .
  orca_dir: .orca
models:
  openai:
    enabled: true
    base_url: "http://localhost:8080"
    default_model: llama3.1
    api_key_env_var: OPENAI_API_KEY
    role: generic
    timeout_seconds: 60
    max_input_tokens: 4096
    max_output_tokens: 2048
"#;

    let parsed: Config =
        serde_yaml::from_str(yaml).expect("config with openai section should parse");
    assert_eq!(parsed.models.openai.base_url, "http://localhost:8080");
    assert_eq!(parsed.models.openai.timeout_seconds, 60);
    assert_eq!(parsed.models.openai.max_input_tokens, 4096);
}

#[test]
fn test_config_roundtrip_preserves_openai() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    let config = Config {
        project: orca_agent_cli::config::schema::ProjectConfig {
            name: "roundtrip".to_string(),
            repo_path: std::path::PathBuf::from("."),
            vault_path: None,
            orca_dir: std::path::PathBuf::from(".orca"),
        },
        execution: orca_agent_cli::config::schema::ExecutionConfig::default(),
        models: orca_agent_cli::config::schema::ModelsConfig::default(),
        routing: orca_agent_cli::config::schema::RoutingConfig::default(),
        context: orca_agent_cli::config::schema::ContextConfig::default(),
        memory: orca_agent_cli::config::schema::MemoryConfig::default(),
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(yaml.as_bytes()).unwrap();

    let loaded = orca_agent_cli::config::load(&path).unwrap();
    assert_eq!(loaded.models.openai.base_url, config.models.openai.base_url);
    assert_eq!(
        loaded.models.openai.default_model,
        config.models.openai.default_model
    );
    assert_eq!(
        loaded.models.openai.timeout_seconds,
        config.models.openai.timeout_seconds
    );
}
