mod helpers;

use helpers::*;
use serial_test::serial;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use tokio::sync::Mutex;

/// Check that the health status returns 'healthy' or 'unhealthy' as expected.
pub async fn assert_status(expected_status: bool) {
    let mut status_command = Command::new("cargo")
        .args(&["run", "--", "status", "get"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("The command to check the status should spawn a child process.");

    // Capture the server's log output on stdout.
    let stdout = status_command
        .stdout
        .take()
        .expect("Stdout output should be captured.");
    let reader = BufReader::new(stdout);
    let lines = Arc::new(Mutex::new(reader.lines()));

    let expected_lines = if expected_status {
        vec!["healthy"]
    } else {
        vec!["unhealthy"]
    };
    check_log_output(lines.clone(), expected_lines).await;

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
}

#[tokio::test]
#[serial]
async fn test_status() {
    // Initially `cargo run -- status` should return `healthy`.
    assert_status(true).await;
}
