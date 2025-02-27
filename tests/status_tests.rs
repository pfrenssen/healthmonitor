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
use tokio::time::sleep;

/// Check that the health state returns 'healthy' or 'unhealthy' as expected.
pub async fn assert_state(expected_state: bool) {
    let (mut state_command, stdout, _stderr) =
        execute_state_command(SubCommands::Get, [].to_vec()).await;

    // The state command should exit with an error code if the application is not healthy.
    let state = state_command
        .wait()
        .await
        .expect("The state command should exit.");

    assert_eq!(
        expected_state,
        state.success(),
        "The state command should exit with code {}.",
        if expected_state { "0" } else { "1" }
    );

    let expected_lines = if expected_state {
        vec!["healthy"]
    } else {
        vec!["unhealthy"]
    };
    check_log_output(stdout.clone(), expected_lines).await;
}

/// Check that the deployment phase returns 'deploying' or 'online' as expected.
async fn assert_phase(expected_phase: DeploymentPhase) {
    let (mut phase_command, stdout, _stderr) =
        execute_phase_command(SubCommands::Get, [].to_vec()).await;

    // The phase command should exit with an error code if the application is not healthy.
    let phase = phase_command
        .wait()
        .await
        .expect("The phase command should exit.");

    assert!(
        phase.success(),
        "The phase command should exit successfully.",
    );

    let expected_lines = match expected_phase {
        DeploymentPhase::Deploying => vec!["deploying"],
        DeploymentPhase::Online => vec!["online"],
    };
    check_log_output(stdout.clone(), expected_lines).await;
}

enum DeploymentPhase {
    Deploying,
    Online,
}

async fn execute_state_command(
    subcommand: SubCommands,
    options: Vec<String>,
) -> (
    tokio::process::Child,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
) {
    let mut state_command = Command::new("cargo")
        .args(["run", "--", "state", &subcommand.to_string()])
        .args(options)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("The command should spawn a child process.");

    // Capture the server's log output.
    let stdout = state_command
        .stdout
        .take()
        .expect("Stdout output should be captured.");
    let stdout_lines = Arc::new(Mutex::new(BufReader::new(stdout).lines()));

    let stderr = state_command
        .stderr
        .take()
        .expect("Stderr output should be captured.");
    let stderr_lines = Arc::new(Mutex::new(BufReader::new(stderr).lines()));

    (state_command, stdout_lines, stderr_lines)
}

async fn execute_phase_command(
    subcommand: SubCommands,
    options: Vec<String>,
) -> (
    tokio::process::Child,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
) {
    let mut phase_command = Command::new("cargo")
        .args(["run", "--", "phase", &subcommand.to_string()])
        .args(options)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("The command should spawn a child process.");

    // Capture the server's log output.
    let stdout = phase_command
        .stdout
        .take()
        .expect("Stdout output should be captured.");
    let stdout_lines = Arc::new(Mutex::new(BufReader::new(stdout).lines()));

    let stderr = phase_command
        .stderr
        .take()
        .expect("Stderr output should be captured.");
    let stderr_lines = Arc::new(Mutex::new(BufReader::new(stderr).lines()));

    (phase_command, stdout_lines, stderr_lines)
}

enum SubCommands {
    Get,
    Set,
}

impl Display for SubCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubCommands::Get => write!(f, "get"),
            SubCommands::Set => write!(f, "set"),
        }
    }
}

#[tokio::test]
#[serial]
async fn test_state() {
    // Make sure the environment variable from the other test is not set, we cannot control the
    // order of tests.
    env::set_var("HEALTHMONITOR_FILECHECK_FILES", "");

    // When we run `cargo run -- state get` without a running server, we should get an error code.
    let (state_command, _, lines) = execute_state_command(SubCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["Failed to get state: Request error: error sending request for url"];
    check_log_output_regex(lines.clone(), expected_lines).await;
    assert_exit_code(state_command, 1).await;

    // Start the server. Now `cargo run -- state get` should return `healthy`.
    let mut server = TestServer::start().await;
    assert_state(true).await;

    // Manually toggle the state to unhealthy. The state should then be reported as unhealthy.
    let options = vec!["unhealthy".to_string()];
    let (update_state_command, _stdout, _stderr) =
        execute_state_command(SubCommands::Set, options).await;
    assert_exit_code(update_state_command, 0).await;
    assert_state(false).await;

    // Toggle back to healthy.
    let options = vec!["healthy".to_string()];
    let (update_state_command, _stdout, _stderr) =
        execute_state_command(SubCommands::Set, options).await;
    assert_exit_code(update_state_command, 0).await;
    assert_state(true).await;

    // Toggle to unhealthy with a custom message.
    let options = vec![
        "unhealthy".to_string(),
        "--message=Apache is not running".to_string(),
    ];
    let (update_state_command, _stdout, _stderr) =
        execute_state_command(SubCommands::Set, options).await;
    assert_exit_code(update_state_command, 0).await;

    // When we now get the state, we should see the custom message.
    let (state_command, stdout, _stderr) =
        execute_state_command(SubCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["unhealthy: Apache is not running"];
    check_log_output(stdout.clone(), expected_lines).await;
    assert_exit_code(state_command, 1).await;

    // Stop the server. We should get an error when we try to get the state.
    server.stop().await;

    let (state_command, _, lines) = execute_state_command(SubCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["Failed to get state: Request error: error sending request for url"];
    check_log_output_regex(lines.clone(), expected_lines).await;
    assert_exit_code(state_command, 1).await;
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

    // Start the server and switch the application to online mode to start the checks. It should be
    // healthy.
    let _server = TestServer::start().await;
    let (phase_command, _stdout, _stderr) =
        execute_phase_command(SubCommands::Set, ["online".to_string()].to_vec()).await;
    assert_exit_code(phase_command, 0).await;
    sleep(tokio::time::Duration::from_secs(2)).await;
    assert_state(true).await;

    // Delete the file. The server should become unhealthy.
    file.close().unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let (state_command, stdout, _stderr) =
        execute_state_command(SubCommands::Get, [].to_vec()).await;
    let expected_lines =
        vec!["unhealthy: FileCheck: Failed to access .*: No such file or directory"];
    check_log_output_regex(stdout.clone(), expected_lines).await;
    assert_exit_code(state_command, 1).await;
}

#[tokio::test]
#[serial]
async fn test_phase() {
    // Make sure the environment variable from the other test is not set, we cannot control the
    // order of tests.
    env::set_var("HEALTHMONITOR_FILECHECK_FILES", "");

    // When we run `cargo run -- phase get` without a running server, we should get an error code.
    let (phase_command, _, lines) = execute_phase_command(SubCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["Failed to get phase: Request error: error sending request for url"];
    check_log_output_regex(lines.clone(), expected_lines).await;
    assert_exit_code(phase_command, 1).await;

    // Start the server. Now `cargo run -- phase get` should return `deploying`.
    let mut server = TestServer::start().await;
    assert_phase(DeploymentPhase::Deploying).await;

    // Switch the application to online mode. The phase should then be reported as online.
    let (update_phase_command, _stdout, _stderr) =
        execute_phase_command(SubCommands::Set, ["online".to_string()].to_vec()).await;
    assert_exit_code(update_phase_command, 0).await;
    assert_phase(DeploymentPhase::Online).await;

    // Stop the server. We should get an error when we try to get the phase.
    server.stop().await;

    let (phase_command, _, lines) = execute_phase_command(SubCommands::Get, [].to_vec()).await;
    let expected_lines = vec!["Failed to get phase: Request error: error sending request for url"];
    check_log_output_regex(lines.clone(), expected_lines).await;
    assert_exit_code(phase_command, 1).await;
}
