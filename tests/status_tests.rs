mod helpers;

use helpers::*;
use serial_test::serial;
use std::env;
use std::fmt::Display;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

/// Check that the health status returns 'healthy' or 'unhealthy' as expected.
pub async fn assert_status(expected_status: bool) {
    let (mut status_command, stdout, _stderr) =
        execute_status_command(StatusCommands::Get, [].to_vec()).await;

    // The status command should exit with an error code if the application is not healthy.
    let status = status_command
        .wait()
        .await
        .expect("The status command should exit.");

    assert_eq!(
        expected_status,
        status.success(),
        "The status command should exit with code {}.",
        if expected_status { "0" } else { "1" }
    );

    let expected_lines = if expected_status {
        vec!["healthy"]
    } else {
        vec!["unhealthy"]
    };
    check_log_output(stdout.clone(), expected_lines).await;
}

async fn execute_status_command(
    subcommand: StatusCommands,
    options: Vec<String>,
) -> (
    tokio::process::Child,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
) {
    let mut status_command = Command::new("cargo")
        .args(["run", "--", "status", &subcommand.to_string()])
        .args(options)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("The command should spawn a child process.");

    // Capture the server's log output.
    let stdout = status_command
        .stdout
        .take()
        .expect("Stdout output should be captured.");
    let stdout_lines = Arc::new(Mutex::new(BufReader::new(stdout).lines()));

    let stderr = status_command
        .stderr
        .take()
        .expect("Stderr output should be captured.");
    let stderr_lines = Arc::new(Mutex::new(BufReader::new(stderr).lines()));

    (status_command, stdout_lines, stderr_lines)
}

enum StatusCommands {
    Get,
    Set,
}

impl Display for StatusCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusCommands::Get => write!(f, "get"),
            StatusCommands::Set => write!(f, "set"),
        }
    }
}

#[tokio::test]
#[serial]
async fn test_status() {
    // Make sure the environment variable from the other test is not set, we cannot control the
    // order of tests.
    env::set_var("HEALTHMONITOR_FILECHECK_FILES", "");

    // When we run `cargo run -- status get` without a running server, we should get an error code.
    let (status_command, _, lines) = execute_status_command(StatusCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["Failed to get status: Request error: error sending request for url"];
    check_log_output_regex(lines.clone(), expected_lines).await;
    assert_exit_code(status_command, 1).await;

    // Start the server. Now `cargo run -- status` should return `healthy`.
    let mut server = TestServer::start().await;
    assert_status(true).await;

    // Manually toggle the status to unhealthy. The status should then be reported as unhealthy.
    let options = vec!["unhealthy".to_string()];
    let (update_status_command, _stdout, _stderr) =
        execute_status_command(StatusCommands::Set, options).await;
    assert_exit_code(update_status_command, 0).await;
    assert_status(false).await;

    // Toggle back to healthy.
    let options = vec!["healthy".to_string()];
    let (update_status_command, _stdout, _stderr) =
        execute_status_command(StatusCommands::Set, options).await;
    assert_exit_code(update_status_command, 0).await;
    assert_status(true).await;

    // Toggle to unhealthy with a custom message.
    let options = vec![
        "unhealthy".to_string(),
        "--message=Apache is not running".to_string(),
    ];
    let (update_status_command, _stdout, _stderr) =
        execute_status_command(StatusCommands::Set, options).await;
    assert_exit_code(update_status_command, 0).await;

    // When we now get the status, we should see the custom message.
    let (status_command, stdout, _stderr) =
        execute_status_command(StatusCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["unhealthy: Apache is not running"];
    check_log_output(stdout.clone(), expected_lines).await;
    assert_exit_code(status_command, 1).await;

    // Stop the server. We should get an error when we try to get the status.
    server.stop().await;

    let (status_command, _, lines) = execute_status_command(StatusCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["Failed to get status: Request error: error sending request for url"];
    check_log_output_regex(lines.clone(), expected_lines).await;
    assert_exit_code(status_command, 1).await;
}

#[tokio::test]
#[serial]
async fn test_file_goes_missing() {
    // Create a test file and configure the server to check it.
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Test").unwrap();
    env::set_var(
        "HEALTHMONITOR_FILECHECK_FILES",
        file.path().to_str().unwrap(),
    );

    // Set a quick interval for the file check.
    env::set_var("HEALTHMONITOR_FILECHECK_INTERVAL", "1");

    // Start the server. It should be healthy.
    let _server = TestServer::start().await;
    assert_status(true).await;

    // Delete the file. The server should become unhealthy.
    file.close().unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let (status_command, stdout, _stderr) =
        execute_status_command(StatusCommands::Get, [].to_vec()).await;
    let expected_lines =
        vec!["unhealthy: FileCheck: Failed to access .*: No such file or directory"];
    check_log_output_regex(stdout.clone(), expected_lines).await;
    assert_exit_code(status_command, 1).await;
}
