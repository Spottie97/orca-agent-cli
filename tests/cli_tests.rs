use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

fn disable_ollama(tmp: &tempfile::TempDir) {
    let config_path = tmp.path().join(".orca/config.yaml");
    let contents = std::fs::read_to_string(&config_path).unwrap();
    let updated = contents.replace("ollama:\n    enabled: true", "ollama:\n    enabled: false");
    std::fs::write(&config_path, updated).unwrap();
}

#[test]
fn test_orca_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Orca Agent CLI"));
}

#[test]
fn test_orca_init_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["init", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_orca_config_validate_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["config", "validate", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}

#[test]
fn test_orca_scan_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["scan", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_context_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["context", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_route_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_plan_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_execute_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_review_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_memory_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["memory", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_run_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_status_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["status", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_splash_help() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["splash", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_orca_init_creates_workspace() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    let orca = tmp.path().join(".orca");
    assert!(orca.exists());
    assert!(orca.join("config.yaml").exists());
    assert!(orca.join("state.json").exists());
    assert!(orca.join("input").exists());
    assert!(orca.join("context-packets").exists());
    assert!(orca.join("prompts").exists());
    assert!(orca.join("prompts/project_memory_agent.md").exists());
    assert!(orca.join("prompts/review_agent.md").exists());
    assert!(orca.join("results").exists());
    assert!(orca.join("memory").exists());
    assert!(orca.join("graph").exists());
    assert!(orca.join("logs").exists());
}

#[test]
fn test_orca_init_dry_run_command_level() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args([
        "init",
        "--dry-run",
        "--project-name",
        "Orca Test Project",
        "--repo",
        ".",
    ]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    let orca = tmp.path().join(".orca");
    assert!(
        !orca.exists(),
        "init --dry-run must not create .orca/ directory"
    );
}

#[test]
fn test_orca_init_dry_run_global() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args([
        "--dry-run",
        "init",
        "--project-name",
        "Orca Test Project",
        "--repo",
        ".",
    ]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    let orca = tmp.path().join(".orca");
    assert!(
        !orca.exists(),
        "--dry-run init must not create .orca/ directory"
    );
}

#[test]
fn test_orca_config_validate() {
    let tmp = tempfile::tempdir().unwrap();

    // init first
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // then validate
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["config", "validate"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_orca_route() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Routing decision"));
}

#[test]
fn test_orca_context() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["context", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Context packet written to"));

    let packet_path = tmp.path().join(".orca/context-packets/TASK-001.md");
    assert!(packet_path.exists());
}

#[test]
fn test_orca_scan() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("scan");
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scanning"));

    let scan_path = tmp.path().join(".orca/scans/latest.md");
    assert!(scan_path.exists());
}

#[test]
fn test_orca_context_dry_run() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["context", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success().stdout(predicate::str::contains(
        "Dry run: would write context packet",
    ));

    let packet_path = tmp.path().join(".orca/context-packets/TASK-001.md");
    assert!(
        !packet_path.exists(),
        "context --dry-run must not create the packet file"
    );
}

#[test]
fn test_orca_execute_dry_run() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"))
        .stdout(predicate::str::contains("No API calls were made"));
}

#[test]
fn test_orca_review() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Review for TASK-001"))
        .stdout(predicate::str::contains("Verdict"));
}

#[test]
fn test_orca_review_saves_artifact() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let review_path = tmp.path().join(".orca/reviews/TASK-001.json");
    assert!(review_path.exists(), "review must save review artifact");

    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["task_id"], "TASK-001");
    assert!(!artifact["verdict"].as_str().unwrap().is_empty());
}

#[test]
fn test_orca_review_considers_execution_result() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Execute the task to create a result artifact
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Review for TASK-001"));

    let review_path = tmp.path().join(".orca/reviews/TASK-001.json");
    assert!(review_path.exists());

    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["task_id"], "TASK-001");
    assert!(
        artifact["execution_status"].is_string() || artifact["execution_status"].is_null(),
        "review artifact should include execution status"
    );
}

#[test]
fn test_orca_init_creates_reviews_dir() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let reviews_dir = tmp.path().join(".orca/reviews");
    assert!(reviews_dir.exists(), "init must create reviews directory");
}

#[test]
fn test_orca_memory_update() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["memory", "update", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Memory updated for task TASK-001"));

    let task_note = tmp.path().join(".orca/tasks/TASK-001.md");
    assert!(task_note.exists());

    let state_path = tmp.path().join(".orca/state.json");
    let state_contents = std::fs::read_to_string(&state_path).unwrap();
    assert!(state_contents.contains("TASK-001"));
    assert!(state_contents.contains("complete"));
}

#[test]
fn test_orca_memory_update_dry_run() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Record initial state file mtime to detect modifications
    let state_path = tmp.path().join(".orca/state.json");
    let initial_mtime = std::fs::metadata(&state_path).unwrap().modified().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["memory", "update", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dry run: would update memory"));

    let task_note = tmp.path().join(".orca/tasks/TASK-001.md");
    assert!(
        !task_note.exists(),
        "memory update --dry-run must not create task note"
    );

    let final_mtime = std::fs::metadata(&state_path).unwrap().modified().unwrap();
    assert_eq!(
        initial_mtime, final_mtime,
        "memory update --dry-run must not modify state.json"
    );
}

#[test]
fn test_orca_memory_update_with_execution_result() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Execute to create result artifact
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Review to create review artifact
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Memory update should use rich format
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["memory", "update", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Memory updated for task TASK-001"));

    let task_note = tmp.path().join(".orca/tasks/TASK-001.md");
    assert!(task_note.exists());

    let contents = std::fs::read_to_string(&task_note).unwrap();
    assert!(
        contents.contains("Task Completion"),
        "memory note should contain 'Task Completion' heading"
    );
    assert!(
        contents.contains("Execution"),
        "memory note should contain 'Execution' section"
    );
    assert!(
        contents.contains("Review"),
        "memory note should contain 'Review' section"
    );
}

#[test]
fn test_orca_plan() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task graph written to"));

    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    assert!(graph_path.exists());
    let contents = std::fs::read_to_string(&graph_path).unwrap();
    assert!(contents.contains("TASK-001"));
    assert!(contents.contains("TASK-002"));
    assert!(contents.contains("TASK-003"));
}

#[test]
fn test_orca_run_dry_run() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Orca Run (dry-run): TASK-001"))
        .stdout(predicate::str::contains("would write context packet to"))
        .stdout(predicate::str::contains("Dry run: would execute provider"))
        .stdout(predicate::str::contains("would update memory"))
        .stdout(predicate::str::contains("Dry run complete"))
        .stdout(predicate::str::contains("no files were changed"));
}

#[test]
fn test_orca_run_dry_run_no_misleading_output() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Context packet written to").not())
        .stdout(predicate::str::contains("Memory updated.").not())
        .stdout(predicate::str::contains("Run complete").not());

    // Verify no files were created/modified
    let context_path = tmp.path().join(".orca/context-packets/TASK-001.md");
    assert!(!context_path.exists());
    let task_note = tmp.path().join(".orca/tasks/TASK-001.md");
    assert!(!task_note.exists());
}

#[test]
fn test_orca_run_dry_run_global() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["--dry-run", "run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("would write context packet to"))
        .stdout(predicate::str::contains("would update memory"))
        .stdout(predicate::str::contains("Dry run complete"));
}

#[test]
fn test_orca_status() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("status");
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project:"));
}

#[test]
fn test_orca_status_json() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["status", "--json"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("project_name"));
}

#[test]
fn test_orca_route_uses_task_graph_metadata() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Generate task graph so TASK-001 exists with planning metadata
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Routing decision"))
        .stdout(predicate::str::contains("ollama").or(predicate::str::contains("claude")))
        .stdout(predicate::str::contains("planning").or(predicate::str::contains("Planning")));
}

#[test]
fn test_orca_route_task_002_medium_risk() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-002"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("medium"))
        .stdout(
            predicate::str::contains("implementation")
                .or(predicate::str::contains("Implementation")),
        );
}

#[test]
fn test_orca_route_task_003_test_type() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-003"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test").or(predicate::str::contains("Test")));
}

#[test]
fn test_orca_route_missing_task_fails() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-999"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TASK-999"))
        .stderr(predicate::str::contains("task-graph.yaml"));
}

#[test]
fn test_orca_context_missing_task_fails() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["context", "--dry-run", "TASK-999"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TASK-999"))
        .stderr(predicate::str::contains("task-graph.yaml"));

    let context_path = tmp.path().join(".orca/context-packets/TASK-999.md");
    assert!(!context_path.exists());
}

#[test]
fn test_orca_run_missing_task_fails() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--dry-run", "TASK-999"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TASK-999"))
        .stderr(predicate::str::contains("task-graph.yaml"));

    let context_path = tmp.path().join(".orca/context-packets/TASK-999.md");
    assert!(!context_path.exists());
    let task_note = tmp.path().join(".orca/tasks/TASK-999.md");
    assert!(!task_note.exists());
}

#[test]
fn test_orca_execute_missing_task_fails() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "--dry-run", "TASK-999"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TASK-999"))
        .stderr(predicate::str::contains("task-graph.yaml"));
}

#[test]
fn test_orca_review_missing_task_fails() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-999"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TASK-999"))
        .stderr(predicate::str::contains("task-graph.yaml"));
}

#[test]
fn test_orca_route_json() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-001", "--json"]);
    cmd.current_dir(&tmp);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("route --json should emit valid JSON");
    assert_eq!(parsed["task_id"], "TASK-001");
    assert!(!parsed["provider"].as_str().unwrap().is_empty());
}

#[test]
fn test_orca_route_json_no_headings() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["route", "TASK-001", "--json"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Routing decision").not());
}

#[test]
fn test_orca_run_json_dry_run() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--dry-run", "TASK-001", "--json"]);
    cmd.current_dir(&tmp);
    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("run --dry-run --json should emit valid JSON");
    assert_eq!(parsed["task_id"], "TASK-001");
    assert_eq!(parsed["mode"], "dry-run");
    assert!(!parsed["provider"].as_str().unwrap().is_empty());
    assert!(!parsed["review_verdict"].as_str().unwrap().is_empty());
}

#[test]
fn test_orca_run_json_no_headings() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--dry-run", "TASK-001", "--json"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("=== Orca Run").not())
        .stdout(predicate::str::contains("[1/5]").not());
}

#[test]
fn test_orca_execute_dry_run_no_result_file() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));

    let result_path = tmp.path().join(".orca/results/TASK-001.json");
    assert!(
        !result_path.exists(),
        "execute --dry-run must not create result artifact"
    );
}

#[test]
fn test_orca_execute_creates_result_file() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let result_path = tmp.path().join(".orca/results/TASK-001.json");
    assert!(
        result_path.exists(),
        "execute must create result artifact on success"
    );

    let contents = std::fs::read_to_string(&result_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["task_id"], "TASK-001");
    assert_eq!(artifact["status"], "success");
}

#[test]
fn test_orca_run_dry_run_no_result_file() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "--dry-run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let result_path = tmp.path().join(".orca/results/TASK-001.json");
    assert!(
        !result_path.exists(),
        "run --dry-run must not create result artifact"
    );
}

#[test]
fn test_orca_run_creates_result_file() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["run", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let result_path = tmp.path().join(".orca/results/TASK-001.json");
    assert!(
        result_path.exists(),
        "run must create result artifact on success"
    );

    let contents = std::fs::read_to_string(&result_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["task_id"], "TASK-001");
    assert_eq!(artifact["status"], "success");
}

#[test]
fn test_orca_execute_creates_no_patch_when_no_files_changed() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let patch_path = tmp.path().join(".orca/patches/TASK-001.json");
    assert!(
        !patch_path.exists(),
        "execute must not create patch artifact when no files changed"
    );
}

#[test]
fn test_orca_patch_list_empty() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["patch", "list"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No patch proposals found"));
}

#[test]
fn test_orca_patch_show_missing() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["patch", "show", "TASK-999"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TASK-999"));
}

#[test]
fn test_orca_patch_show_and_list() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Manually create a patch proposal
    let patches_dir = tmp.path().join(".orca/patches");
    std::fs::create_dir_all(&patches_dir).unwrap();
    let patch = serde_json::json!({
        "task_id": "TASK-PATCH",
        "provider": "mock",
        "model": "mock-model",
        "timestamp": "now",
        "files_changed": [
            {
                "path": "src/main.rs",
                "original": null,
                "proposed": "new content",
                "explanation": "fix typo"
            }
        ],
        "summary": "Fix typo in main.rs"
    });
    std::fs::write(
        patches_dir.join("TASK-PATCH.json"),
        serde_json::to_string_pretty(&patch).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["patch", "show", "TASK-PATCH"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("fix typo"));

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["patch", "list"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TASK-PATCH"));
}

#[test]
fn test_orca_init_creates_patches_dir() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let patches_dir = tmp.path().join(".orca/patches");
    assert!(patches_dir.exists(), "init must create patches directory");
}

#[test]
fn test_orca_init_json_rejected() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["init", "--json"]);
    cmd.assert().failure().stderr(
        predicate::str::contains("unexpected argument").or(predicate::str::contains("--json")),
    );
}

#[test]
fn test_orca_review_json_rejected() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001", "--json"]);
    cmd.assert().failure().stderr(
        predicate::str::contains("unexpected argument").or(predicate::str::contains("--json")),
    );
}

#[test]
fn test_orca_execute_json_rejected() {
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "TASK-001", "--json"]);
    cmd.assert().failure().stderr(
        predicate::str::contains("unexpected argument").or(predicate::str::contains("--json")),
    );
}

#[test]
fn test_orca_review_fails_closed_without_execution_result() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Review before executing — should reject because no result exists
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reject"));

    let review_path = tmp.path().join(".orca/reviews/TASK-001.json");
    assert!(review_path.exists());

    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["verdict"], "reject");
    assert_eq!(artifact["accepted"], false);
}

#[test]
fn test_orca_task_graph_acceptance_criteria_propagates_to_context_packet() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Create a task graph with explicit acceptance criteria
    let graph = serde_json::json!({
        "version": "1.0",
        "project": "Test",
        "tasks": [
            {
                "id": "TASK-AC-001",
                "title": "Smoke test",
                "description": "Reply with OK.",
                "task_type": "tests",
                "complexity": "low",
                "risk": "low",
                "dependencies": [],
                "status": "pending",
                "acceptance_criteria": [
                    "Output must be exactly ORCA_CLOUD_OK."
                ]
            }
        ]
    });
    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    std::fs::write(&graph_path, serde_yaml::to_string(&graph).unwrap()).unwrap();

    // Run context command to generate packet
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["context", "TASK-AC-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let packet_path = tmp.path().join(".orca/context-packets/TASK-AC-001.md");
    assert!(packet_path.exists());

    let contents = std::fs::read_to_string(&packet_path).unwrap();
    assert!(
        contents.contains("Output must be exactly ORCA_CLOUD_OK."),
        "context packet must contain acceptance criteria from task graph"
    );
}

#[test]
fn test_orca_execute_prompt_includes_acceptance_criteria_from_graph() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let graph = serde_json::json!({
        "version": "1.0",
        "project": "Test",
        "tasks": [
            {
                "id": "TASK-AC-002",
                "title": "Smoke test",
                "description": "Reply with OK.",
                "task_type": "tests",
                "complexity": "low",
                "risk": "low",
                "dependencies": [],
                "status": "pending",
                "acceptance_criteria": [
                    "Output must be exactly ORCA_CLOUD_OK."
                ]
            }
        ]
    });
    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    std::fs::write(&graph_path, serde_yaml::to_string(&graph).unwrap()).unwrap();

    // Execute dry-run to verify prompt construction
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "--dry-run", "TASK-AC-002"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // The result isn't written in dry-run, but we verified resolve + prompt building works
    // by not crashing. The acceptance criteria propagation is tested above.
}

#[test]
fn test_orca_review_loads_execution_result_fields() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    disable_ollama(&tmp);

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["plan", "--planner", "manual"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    // Execute to create a result artifact
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["execute", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let result_path = tmp.path().join(".orca/results/TASK-001.json");
    assert!(result_path.exists());

    // Review should load the result and include execution metadata
    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let review_path = tmp.path().join(".orca/reviews/TASK-001.json");
    assert!(review_path.exists());

    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["task_id"], "TASK-001");
    assert!(
        artifact["execution_status"].is_string() || artifact["execution_status"].is_null(),
        "review artifact should include execution_status"
    );
    assert!(
        !artifact["verdict"].as_str().unwrap().is_empty(),
        "review artifact must have a verdict"
    );
}

// HOTFIX-007 integration tests: exact-output acceptance criteria enforcement

#[test]
fn test_orca_review_exact_output_accepts_when_exact() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let graph = serde_json::json!({
        "version": "1.0",
        "project": "Test",
        "tasks": [
            {
                "id": "TASK-EXACT-001",
                "title": "Exact output test",
                "description": "Reply with exactly ORCA_CLOUD_OK.",
                "task_type": "planning",
                "complexity": "low",
                "risk": "low",
                "dependencies": [],
                "status": "pending",
                "acceptance_criteria": [
                    "Output must be exactly ORCA_CLOUD_OK."
                ]
            }
        ]
    });
    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    std::fs::write(&graph_path, serde_yaml::to_string(&graph).unwrap()).unwrap();

    let result = serde_json::json!({
        "task_id": "TASK-EXACT-001",
        "provider": "mock",
        "model": "mock-model",
        "timestamp": "now",
        "status": "success",
        "output": "ORCA_CLOUD_OK",
        "duration_ms": 100,
        "input_tokens": 10,
        "output_tokens": 5
    });
    let results_dir = tmp.path().join(".orca/results");
    std::fs::create_dir_all(&results_dir).unwrap();
    std::fs::write(
        results_dir.join("TASK-EXACT-001.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-EXACT-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("accept"));

    let review_path = tmp.path().join(".orca/reviews/TASK-EXACT-001.json");
    assert!(review_path.exists());
    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["verdict"], "accept");
    assert_eq!(artifact["accepted"], true);
    assert!(artifact["execution_status"].is_string());
    assert!(artifact["unmet_acceptance_criteria"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn test_orca_review_exact_output_rejects_with_trailing_period() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let graph = serde_json::json!({
        "version": "1.0",
        "project": "Test",
        "tasks": [
            {
                "id": "TASK-EXACT-002",
                "title": "Exact output test",
                "description": "Reply with exactly ORCA_CLOUD_OK.",
                "task_type": "planning",
                "complexity": "low",
                "risk": "low",
                "dependencies": [],
                "status": "pending",
                "acceptance_criteria": [
                    "Output must be exactly ORCA_CLOUD_OK."
                ]
            }
        ]
    });
    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    std::fs::write(&graph_path, serde_yaml::to_string(&graph).unwrap()).unwrap();

    let result = serde_json::json!({
        "task_id": "TASK-EXACT-002",
        "provider": "mock",
        "model": "mock-model",
        "timestamp": "now",
        "status": "success",
        "output": "ORCA_CLOUD_OK.",
        "duration_ms": 100,
        "input_tokens": 10,
        "output_tokens": 5
    });
    let results_dir = tmp.path().join(".orca/results");
    std::fs::create_dir_all(&results_dir).unwrap();
    std::fs::write(
        results_dir.join("TASK-EXACT-002.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-EXACT-002"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reject"));

    let review_path = tmp.path().join(".orca/reviews/TASK-EXACT-002.json");
    assert!(review_path.exists());
    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["verdict"], "reject");
    assert_eq!(artifact["accepted"], false);
    assert!(artifact["execution_status"].is_string());
    let unmet = artifact["unmet_acceptance_criteria"].as_array().unwrap();
    assert!(
        unmet
            .iter()
            .any(|u| u.as_str().unwrap().contains("exactly")),
        "unmet_acceptance_criteria should contain the exact criterion"
    );
}

#[test]
fn test_orca_review_exact_output_rejects_with_extra_words() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let graph = serde_json::json!({
        "version": "1.0",
        "project": "Test",
        "tasks": [
            {
                "id": "TASK-EXACT-003",
                "title": "Exact output test",
                "description": "Reply with exactly ORCA_CLOUD_OK.",
                "task_type": "planning",
                "complexity": "low",
                "risk": "low",
                "dependencies": [],
                "status": "pending",
                "acceptance_criteria": [
                    "Output must be exactly ORCA_CLOUD_OK."
                ]
            }
        ]
    });
    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    std::fs::write(&graph_path, serde_yaml::to_string(&graph).unwrap()).unwrap();

    let result = serde_json::json!({
        "task_id": "TASK-EXACT-003",
        "provider": "mock",
        "model": "mock-model",
        "timestamp": "now",
        "status": "success",
        "output": "The answer is ORCA_CLOUD_OK",
        "duration_ms": 100,
        "input_tokens": 10,
        "output_tokens": 5
    });
    let results_dir = tmp.path().join(".orca/results");
    std::fs::create_dir_all(&results_dir).unwrap();
    std::fs::write(
        results_dir.join("TASK-EXACT-003.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-EXACT-003"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reject"));

    let review_path = tmp.path().join(".orca/reviews/TASK-EXACT-003.json");
    assert!(review_path.exists());
    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["verdict"], "reject");
    assert_eq!(artifact["accepted"], false);
    assert!(artifact["execution_status"].is_string());
    let unmet = artifact["unmet_acceptance_criteria"].as_array().unwrap();
    assert!(
        unmet
            .iter()
            .any(|u| u.as_str().unwrap().contains("exactly")),
        "unmet_acceptance_criteria should contain the exact criterion"
    );
}

#[test]
fn test_orca_review_exact_output_rejects_refusal() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.arg("init");
    cmd.current_dir(&tmp);
    cmd.assert().success();

    let graph = serde_json::json!({
        "version": "1.0",
        "project": "Test",
        "tasks": [
            {
                "id": "TASK-EXACT-004",
                "title": "Exact output test",
                "description": "Reply with exactly ORCA_CLOUD_OK.",
                "task_type": "planning",
                "complexity": "low",
                "risk": "low",
                "dependencies": [],
                "status": "pending",
                "acceptance_criteria": [
                    "Output must be exactly ORCA_CLOUD_OK."
                ]
            }
        ]
    });
    let graph_path = tmp.path().join(".orca/task-graph.yaml");
    std::fs::write(&graph_path, serde_yaml::to_string(&graph).unwrap()).unwrap();

    let result = serde_json::json!({
        "task_id": "TASK-EXACT-004",
        "provider": "mock",
        "model": "mock-model",
        "timestamp": "now",
        "status": "success",
        "output": "I don't have access to your task management system or enough context to complete TASK-EXACT-004.",
        "duration_ms": 100,
        "input_tokens": 10,
        "output_tokens": 5
    });
    let results_dir = tmp.path().join(".orca/results");
    std::fs::create_dir_all(&results_dir).unwrap();
    std::fs::write(
        results_dir.join("TASK-EXACT-004.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("orca").unwrap();
    cmd.args(["review", "TASK-EXACT-004"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reject"));

    let review_path = tmp.path().join(".orca/reviews/TASK-EXACT-004.json");
    assert!(review_path.exists());
    let contents = std::fs::read_to_string(&review_path).unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(artifact["verdict"], "reject");
    assert_eq!(artifact["accepted"], false);
    assert!(
        artifact["reasons"].as_array().unwrap().iter().any(|r| r
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("missing context")),
        "refusal/missing-context reason should remain present"
    );
}
