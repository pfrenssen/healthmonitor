use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::Notify;

#[allow(dead_code)] // Not dead code, used in tests.
pub struct TestServer {
    server: tokio::process::Child,
    pub stdout: Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    pub stderr: Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(server_id) = self.server.id() {
            tokio::task::spawn_blocking(move || {
                let pid = Pid::from_raw(server_id as i32);
                kill(pid, Signal::SIGINT).expect("The SIGINT signal should be sent.");
            });
        }
    }
}

#[allow(dead_code)] // Not dead code, used in tests.
impl TestServer {
    pub async fn start() -> Self {
        let (server, stdout, stderr, _notify) = start_server().await;
        TestServer {
            server,
            stdout,
            stderr,
        }
    }

    pub async fn stop(&mut self) {
        stop_server(&mut self.server).await;
    }
}

#[allow(dead_code)] // Not dead code, used in tests.
async fn start_server() -> (
    tokio::process::Child,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>>,
    Arc<Notify>,
) {
    // Run `cargo run -- server start` as a child process.
    let mut server = Command::new("cargo")
        .args(["run", "--", "server", "start"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("The command to start the server should spawn a child process.");

    // Capture the server's log output on stdout.
    let stdout = server
        .stdout
        .take()
        .expect("Stdout output should be captured.");
    let stdout_lines = Arc::new(Mutex::new(BufReader::new(stdout).lines()));

    // Capture the server's log output on stderr.
    let stderr = server
        .stderr
        .take()
        .expect("Stderr output should be captured.");
    let stderr_lines = Arc::new(Mutex::new(BufReader::new(stderr).lines()));

    // Wait for the server to start by checking for the log message "Server started." in an
    // asynchronous task.
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let lines_clone = Arc::clone(&stderr_lines);
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines_clone.lock().await.next_line().await {
            if line.contains("Server started.") {
                notify_clone.notify_one();
                break;
            }
        }
    });
    notify.notified().await;

    (server, stdout_lines, stderr_lines, notify)
}

#[allow(dead_code)] // Not dead code, used in tests.
async fn stop_server(server: &mut tokio::process::Child) {
    let pid = Pid::from_raw(
        server
            .id()
            .expect("The server process should be running and have a process ID.") as i32,
    );
    kill(pid, Signal::SIGINT).expect("The SIGINT signal should be sent.");

    // Wait for the server to shut down.
    let status = server
        .wait()
        .await
        .expect("The server process should exit.");
    assert!(status.success(), "Server did not shut down gracefully");
}

#[allow(dead_code)] // Not dead code, used in tests.
pub async fn server_status() -> (
    tokio::process::Child,
    Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
) {
    // Run `cargo run -- server status` as a child process.
    let mut status_command = Command::new("cargo")
        .args(["run", "--", "server", "status"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("The command to check the server status should spawn a child process.");

    // Capture the server's log output on stdout.
    let stdout = status_command
        .stdout
        .take()
        .expect("Stdout output should be captured.");
    let reader = BufReader::new(stdout);
    let lines = Arc::new(Mutex::new(reader.lines()));

    (status_command, lines)
}

#[allow(dead_code)] // Not dead code, used in tests.
pub async fn check_log_output_regex<T>(
    lines: Arc<Mutex<tokio::io::Lines<BufReader<T>>>>,
    regex_expected_lines: Vec<&str>,
) where
    T: tokio::io::AsyncRead + Unpin,
{
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

#[allow(dead_code)] // Not dead code, used in tests.
pub async fn check_log_output<T>(
    lines: Arc<Mutex<tokio::io::Lines<BufReader<T>>>>,
    expected_lines: Vec<&str>,
) where
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

/// Asserts that the given command exits with the expected exit code.
#[allow(dead_code)] // Not dead code, used in tests.
pub async fn assert_exit_code(mut command: tokio::process::Child, expected_exit_code: i32) {
    let status = command.wait().await.expect("The command should exit.");
    assert_eq!(
        status.code().unwrap_or(-1),
        expected_exit_code,
        "The command should exit with code {}.",
        expected_exit_code
    );
}
