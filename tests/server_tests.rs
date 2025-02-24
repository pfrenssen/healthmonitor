mod helpers;

use helpers::{check_log_output, check_log_output_regex, server_status, start_server, stop_server};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_stop_server_by_sending_sigint() {
    let (mut server, _, lines, _notify) = start_server().await;

    stop_server(&mut server).await;

    let expected_lines = vec![
        ".*INFO.*Received SIGINT, shutting down.*",
        ".*INFO.*Server stopped.*",
    ];
    check_log_output_regex(lines, expected_lines).await;
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
    let (mut server, _, _, _notify) = start_server().await;

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
    stop_server(&mut server).await;

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
