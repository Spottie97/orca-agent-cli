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
    assert!(orca.join("results").exists());
    assert!(orca.join("memory").exists());
    assert!(orca.join("graph").exists());
    assert!(orca.join("logs").exists());
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
