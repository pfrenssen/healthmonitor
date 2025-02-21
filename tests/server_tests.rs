use std::process::Stdio;
use std::sync::{Arc};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serial_test::serial;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::Notify;

async fn start_server() -> (tokio::process::Child, Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>, Arc<Notify>) {
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

    (server, lines, notify)
}

async fn server_status() -> (tokio::process::Child, Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>) {
    // Run `cargo run -- server status` as a child process.
    let mut status_command = Command::new("cargo")
        .args(&["run", "--", "server", "status"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("The command to check the server status should spawn a child process.");

    // Capture the server's log output on stderr.
    let stderr = status_command.stdout.take().expect("Stdout output should be captured.");
    let reader = BufReader::new(stderr);
    let lines = Arc::new(Mutex::new(reader.lines()));

    (status_command, lines)
}

async fn stop_server(server: &mut tokio::process::Child) {
    let pid = Pid::from_raw(server.id().expect("The server process should be running and have a process ID.") as i32);
    kill(pid, Signal::SIGINT).expect("The SIGINT signal should be sent.");

    // Wait for the server to shut down.
    let status = server.wait().await.expect("The server process should exit.");
    assert!(status.success(), "Server did not shut down gracefully");
}

async fn check_log_output_regex(lines: Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>, regex_expected_lines: Vec<&str>) {
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

async fn check_log_output<T>(lines: Arc<Mutex<tokio::io::Lines<BufReader<T>>>>, expected_lines: Vec<&str>)
where
    T: tokio::io::AsyncRead + Unpin,
{
    let mut captured_lines = Vec::new();
    while let Ok(Some(line)) = lines.lock().await.next_line().await {
        captured_lines.push(line);
    }

    for expected_line in expected_lines {
        let found = captured_lines.iter().any(|line| line == expected_line);
        assert!(found, "The output contains the line '{}'.", expected_line);
    }
}

#[tokio::test]
#[serial]
async fn test_stop_server_by_sending_sigint() {
    let (mut server, lines, _notify) = start_server().await;

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
    let status = status_command.wait().await.expect("The status command should exit.");
    assert!(!status.success(), "The status command should exit with an error code if the server is not running.");

    // Start the server.
    let (mut server, _, _notify) = start_server().await;

    // Now the status command should return `running`.
    let (mut status_command, lines) = server_status().await;
    let expected_lines = vec!["running"];
    check_log_output(lines.clone(), expected_lines).await;

    // The status command should exit with a success code.
    let status = status_command.wait().await.expect("The status command should exit.");
    assert!(status.success(), "The status command should exit with a success code if the server is running.");

    // Stop the server. The status command should return `not running` again.
    stop_server(&mut server).await;

    let (mut server_status, lines) = server_status().await;
    let expected_lines = vec!["not running"];
    check_log_output(lines.clone(), expected_lines).await;
    let status = server_status.wait().await.expect("The status command should exit.");
    assert!(!status.success(), "The status command should exit with an error code if the server is not running.");
}
