use std::process::Stdio;
use std::sync::{Arc};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

#[tokio::test]
async fn test_stop_server_by_sending_sigint() {
    // Find a free port in the range 8000-9000.
    let port = port_check::free_local_port_in_range(8000..9000)
        .expect("A free port should be found in the range 8000-9000.");
    std::env::set_var("HEALTHCHECKER_SERVER_ADDRESS", "127.0.0.1");
    std::env::set_var("HEALTHCHECKER_SERVER_PORT", port.to_string());

    // Run `cargo run -- server start` as a child process.
    let mut server = Command::new("cargo")
        .args(&["run", "--", "server", "start"])
        .stderr(Stdio::piped())
        .spawn()
        .expect("The command to start the server should spawn a child process.");

    // Capture the server's log output on stderr.
    let stderr = server.stderr.take().expect("Stderr output should be captured.");
    let reader = BufReader::new(stderr);
    let lines = Arc::new(Mutex::new(reader.lines()));

    // Wait for the server to start by checking for the log message "Server started." in an
    // asynchronous task.
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let lines_clone = Arc::clone(&lines);
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines_clone.lock().await.next_line().await {
            if line.contains("Server started.") {
                notify_clone.notify_one();
                break;
            }
        }
    });
    notify.notified().await;

    // Server is up. Terminate the server by sending SIGINT signal.
    let pid = Pid::from_raw(server.id().expect("The server process should be running and have a process ID.") as i32);
    kill(pid, Signal::SIGINT).expect("The SIGINT signal should be sent.");

    // Wait for the server to shut down.
    let status = server.wait().await.expect("The server process should exit.");
    assert!(status.success(), "Server did not shut down gracefully");

    // Check that the log output contains the expected log messages related to the server shutdown.
    let regex_expected_lines = vec![
        ".*INFO.*Received SIGINT, shutting down.*",
        ".*INFO.*Server stopped.*",
    ];

    let mut captured_lines = Vec::new();
    while let Ok(Some(line)) = lines.lock().await.next_line().await {
        captured_lines.push(line);
    }

    for expected_line in regex_expected_lines {
        let re = regex::Regex::new(expected_line).expect("Failed to compile regex");
        let found = captured_lines.iter().any(|line| re.is_match(line.as_ref()));
        assert!(found, "The output contains the line '{}'.", expected_line);
    }

}
