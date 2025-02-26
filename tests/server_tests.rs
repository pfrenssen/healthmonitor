mod helpers;

use helpers::*;
use serial_test::serial;
use tokio::time::sleep;

#[tokio::test]
#[serial]
async fn test_stop_server_by_sending_sigint() {
    let mut server = TestServer::start().await;

    sleep(tokio::time::Duration::from_secs(1)).await;

    // Our stop() command works by sending a SIGINT signal to the server process.
    server.stop().await;

    let expected_lines = vec![
        ".*INFO.*Received SIGINT, shutting down.*",
        ".*INFO.*Server stopped.*",
    ];
    check_log_output_regex(server.stderr.clone(), expected_lines).await;
}

#[tokio::test]
#[serial]
async fn test_server_status() {
    // Initially `cargo run -- server status` should return `not running`.
    let (mut status_command, lines) = server_status().await;
    let expected_lines = vec!["not running"];
    check_log_output(lines.clone(), expected_lines).await;

    // The status command should exit with an error code.
    let status = status_command
        .wait()
        .await
        .expect("The status command should exit.");
    assert!(
        !status.success(),
        "The status command should exit with an error code if the server is not running."
    );

    // Start the server.
    let mut server = TestServer::start().await;

    // Now the status command should return `running`.
    let (mut status_command, lines) = server_status().await;
    let expected_lines = vec!["running"];
    check_log_output(lines.clone(), expected_lines).await;

    // The status command should exit with a success code.
    let status = status_command
        .wait()
        .await
        .expect("The status command should exit.");
    assert!(
        status.success(),
        "The status command should exit with a success code if the server is running."
    );

    // Stop the server. The status command should return `not running` again.
    server.stop().await;

    let (mut server_status, lines) = server_status().await;
    let expected_lines = vec!["not running"];
    check_log_output(lines.clone(), expected_lines).await;
    let status = server_status
        .wait()
        .await
        .expect("The status command should exit.");
    assert!(
        !status.success(),
        "The status command should exit with an error code if the server is not running."
    );
}
