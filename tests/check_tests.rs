mod helpers;

use helpers::*;
use serial_test::serial;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

/// Check that the health status returns 'healthy' or 'unhealthy' as expected.
pub async fn assert_check(expected_status: bool, env_vars: &[(&str, &str)]) {
    let (mut check_command, stdout, _stderr) = execute_check_command(env_vars).await;

    // The check command should exit with an error code if the application is not healthy.
    let status = check_command
        .wait()
        .await
        .expect("The command should exit.");

    assert_eq!(
        expected_status,
        status.success(),
        "The command should exit with code {}.",
        if expected_status { "0" } else { "1" }
    );

    let expected_lines = if expected_status {
        vec!["ok"]
    } else {
        vec!["error"]
    };
    check_log_output_regex(stdout.clone(), expected_lines).await;
}

async fn execute_check_command(
    env_vars: &[(&str, &str)],
) -> (
    tokio::process::Child,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
) {
    let mut status_command = Command::new("cargo")
        .args(["run", "--", "check"])
        .envs(env_vars.to_owned())
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

#[tokio::test]
#[serial]
async fn test_check() {
    // Start with a clean slate. Unset all environment variables related to checks.
    let mut env_vars = vec![("HEALTHMONITOR_FILECHECK_FILES", "")];

    // We have no checks configured, so no checks are done and the application should be healthy.
    assert_check(true, &env_vars).await;

    // Create a test file and configure the health monitor to check it.
    let mut file = NamedTempFile::new().unwrap();
    let file_path = file.path().to_str().unwrap().to_string();
    env_vars = vec![("HEALTHMONITOR_FILECHECK_FILES", &file_path)];

    // The file is empty, so the application should be unhealthy.
    assert_check(false, &env_vars).await;

    // Write to the file. The application should now be healthy.
    writeln!(file, "Test").unwrap();
    assert_check(true, &env_vars).await;

    // Delete the file. The application should now be unhealthy.
    file.close().unwrap();
    assert_check(false, &env_vars).await;
}
