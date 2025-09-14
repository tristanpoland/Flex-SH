use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flex-SH"))
        .stdout(predicate::str::contains("Built-in commands"));
}

#[test]
fn test_echo_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("echo hello world");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_pwd_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("pwd");
    cmd.assert().success();
}

#[test]
fn test_ls_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file
    fs::write(temp_path.join("test.txt"), "test content").unwrap();

    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.current_dir(temp_path);
    cmd.arg("-c").arg("ls");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_clear_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("clear");
    cmd.assert().success();
}

#[test]
fn test_which_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("which nonexistent_command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("no nonexistent_command in PATH"));
}

#[test]
fn test_env_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("env");
    cmd.assert().success();
}

#[test]
fn test_cd_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg(&format!("cd {}", temp_path.display()));
    cmd.assert().success();
}

#[test]
fn test_exit_command() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("-c").arg("exit 0");
    cmd.assert().success();
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("flex-sh"));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("high-performance, modern system shell"));
}

#[test]
fn test_invalid_flag() {
    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.arg("--invalid-flag");
    cmd.assert().failure();
}

#[test]
fn test_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("file1.txt"), "line1\nline2\nline3").unwrap();

    let mut cmd = Command::cargo_bin("flex-sh").unwrap();
    cmd.current_dir(temp_path);
    cmd.arg("-c").arg("echo 'test' | echo 'pipeline'");
    cmd.assert().success();
}