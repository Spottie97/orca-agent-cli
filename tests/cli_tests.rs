use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

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
    cmd.args(["review", "TASK-001"]);
    cmd.current_dir(&tmp);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Review for TASK-001"))
        .stdout(predicate::str::contains("Verdict"));
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
