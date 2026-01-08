//! Process Manager for spawning and managing CLI agent processes

use dashmap::DashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, mpsc, Mutex};

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub task_id: i32,
    pub line: String,
    pub is_stderr: bool,
}

/// Output line for UI refinements sessions (using string session IDs)
#[derive(Debug, Clone)]
pub struct SessionOutputLine {
    pub session_id: String,
    pub line: String,
    pub is_stderr: bool,
}

/// Event fired when a process exits
#[derive(Debug, Clone)]
pub struct ProcessExitEvent {
    pub task_id: i32,
    pub exit_code: i32,
    pub output_log: String,
}

pub struct ProcessHandle {
    pub pid: Option<u32>,
    input_tx: mpsc::Sender<String>,
    kill_tx: mpsc::Sender<()>,
}

pub struct ProcessManager {
    processes: Arc<DashMap<i32, ProcessHandle>>,
    output_tx: broadcast::Sender<OutputLine>,
    exit_tx: broadcast::Sender<ProcessExitEvent>,
    /// Output buffers for each task (task_id -> accumulated output lines)
    output_buffers: Arc<DashMap<i32, Arc<Mutex<Vec<String>>>>>,
    // Session-based processes for UI refinements
    session_processes: Arc<DashMap<String, ProcessHandle>>,
    session_output_tx: broadcast::Sender<SessionOutputLine>,
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
            output_tx: self.output_tx.clone(),
            exit_tx: self.exit_tx.clone(),
            output_buffers: Arc::clone(&self.output_buffers),
            session_processes: Arc::clone(&self.session_processes),
            session_output_tx: self.session_output_tx.clone(),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        let (output_tx, _) = broadcast::channel(1000);
        let (exit_tx, _) = broadcast::channel(100);
        let (session_output_tx, _) = broadcast::channel(1000);
        Self {
            processes: Arc::new(DashMap::new()),
            output_tx,
            exit_tx,
            output_buffers: Arc::new(DashMap::new()),
            session_processes: Arc::new(DashMap::new()),
            session_output_tx,
        }
    }

    /// Subscribe to output from all processes
    pub fn subscribe_output(&self) -> broadcast::Receiver<OutputLine> {
        self.output_tx.subscribe()
    }

    /// Subscribe to process exit events
    pub fn subscribe_exits(&self) -> broadcast::Receiver<ProcessExitEvent> {
        self.exit_tx.subscribe()
    }

    /// Check if a process is running for a task
    pub fn is_running(&self, task_id: i32) -> bool {
        self.processes.contains_key(&task_id)
    }

    /// Get the PID of a running process
    pub fn get_pid(&self, task_id: i32) -> Option<u32> {
        self.processes.get(&task_id).and_then(|h| h.pid)
    }

    /// Spawn a CLI agent process for a task
    pub async fn spawn_agent(
        &self,
        task_id: i32,
        prompt: &str,
        worktree_path: &str,
    ) -> Result<u32, String> {
        if self.is_running(task_id) {
            return Err(format!("Task {} already has a running process", task_id));
        }

        // Verify working directory exists
        if !std::path::Path::new(worktree_path).exists() {
            return Err(format!(
                "Working directory does not exist: {}",
                worktree_path
            ));
        }

        // Check if claude command is available
        let claude_check = Command::new("which")
            .arg("claude")
            .output()
            .await;

        if claude_check.is_err() || !claude_check.unwrap().status.success() {
            return Err(
                "The 'claude' command is not found in PATH. Please install Claude Code CLI and ensure it's in your PATH.".to_string()
            );
        }

        // Use script command to provide a PTY for claude
        // This makes claude think it's running in a terminal
        // On macOS: script -q file command args...
        // The -q flag suppresses the "Script started/done" messages
        // The -p flag makes claude run in non-interactive "print" mode (closes when done)
        let mut child = Command::new("script")
            .arg("-q")              // Quiet mode
            .arg("/dev/null")       // Don't save transcript to file
            .arg("claude")
            .arg("-p")              // Non-interactive print mode (exits when done)
            .arg(prompt)
            .arg("--dangerously-skip-permissions")
            .current_dir(worktree_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn claude process: {} (working dir: {})", e, worktree_path))?;

        let pid = child.id();

        // Take ownership of streams
        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

        // Create channels for input and kill signal
        let (input_tx, mut input_rx) = mpsc::channel::<String>(100);
        let (kill_tx, mut kill_rx) = mpsc::channel::<()>(1);

        // Store process handle
        let handle = ProcessHandle {
            pid,
            input_tx,
            kill_tx,
        };
        self.processes.insert(task_id, handle);

        // Create output buffer for this task
        let output_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        self.output_buffers.insert(task_id, Arc::clone(&output_buffer));

        let output_tx = self.output_tx.clone();
        let processes = Arc::clone(&self.processes);

        // Spawn task to handle stdout
        let output_tx_stdout = output_tx.clone();
        let buffer_stdout = Arc::clone(&output_buffer);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                // Append to buffer
                buffer_stdout.lock().await.push(line.clone());
                // Broadcast to websockets
                let _ = output_tx_stdout.send(OutputLine {
                    task_id,
                    line,
                    is_stderr: false,
                });
            }
        });

        // Spawn task to handle stderr
        let output_tx_stderr = output_tx.clone();
        let buffer_stderr = Arc::clone(&output_buffer);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                // Append to buffer
                buffer_stderr.lock().await.push(line.clone());
                // Broadcast to websockets
                let _ = output_tx_stderr.send(OutputLine {
                    task_id,
                    line,
                    is_stderr: true,
                });
            }
        });

        // Spawn task to handle stdin and kill signal
        tokio::spawn(async move {
            let mut stdin = stdin;
            loop {
                tokio::select! {
                    Some(input) = input_rx.recv() => {
                        if let Err(e) = stdin.write_all(input.as_bytes()).await {
                            tracing::error!("Failed to write to stdin: {}", e);
                            break;
                        }
                        let _ = stdin.flush().await;
                    }
                    _ = kill_rx.recv() => {
                        // Kill signal received
                        break;
                    }
                }
            }
        });

        // Spawn task to wait for process completion and cleanup
        let processes_cleanup = Arc::clone(&processes);
        let output_buffers_cleanup = Arc::clone(&self.output_buffers);
        let output_tx_exit = output_tx.clone();
        let exit_tx = self.exit_tx.clone();
        tokio::spawn(async move {
            let status = child.wait().await;
            let exit_code = status
                .ok()
                .and_then(|s| s.code())
                .unwrap_or(-1);

            // Broadcast exit message to websockets
            let _ = output_tx_exit.send(OutputLine {
                task_id,
                line: format!("\n[Process exited with code {}]", exit_code),
                is_stderr: false,
            });

            // Collect buffered output and send exit event
            let output_log = if let Some((_, buffer)) = output_buffers_cleanup.remove(&task_id) {
                buffer.lock().await.join("\n")
            } else {
                String::new()
            };

            let _ = exit_tx.send(ProcessExitEvent {
                task_id,
                exit_code,
                output_log,
            });

            processes_cleanup.remove(&task_id);
        });

        Ok(pid.unwrap_or(0))
    }

    /// Send input to a process
    pub async fn send_input(&self, task_id: i32, input: &str) -> Result<(), String> {
        let handle = self
            .processes
            .get(&task_id)
            .ok_or(format!("No process for task {}", task_id))?;

        handle
            .input_tx
            .send(input.to_string())
            .await
            .map_err(|e| format!("Failed to send input: {}", e))
    }

    /// Kill a process
    pub async fn kill_process(&self, task_id: i32) -> Result<(), String> {
        let handle = self
            .processes
            .get(&task_id)
            .ok_or(format!("No process for task {}", task_id))?;

        handle
            .kill_tx
            .send(())
            .await
            .map_err(|e| format!("Failed to send kill signal: {}", e))?;

        // Also try to kill the process directly
        if let Some(pid) = handle.pid {
            // Use kill command to terminate
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output()
                .await;
        }

        Ok(())
    }

    /// Get list of running task IDs
    pub fn running_tasks(&self) -> Vec<i32> {
        self.processes.iter().map(|r| *r.key()).collect()
    }

    // ============ Session-based methods for UI Refinements ============

    /// Subscribe to output from all session processes
    pub fn subscribe_session_output(&self) -> broadcast::Receiver<SessionOutputLine> {
        self.session_output_tx.subscribe()
    }

    /// Check if a session has a running process
    pub fn is_session_running(&self, session_id: &str) -> bool {
        self.session_processes.contains_key(session_id)
    }

    /// Spawn a CLI agent for a UI refinements session
    pub async fn spawn_session_agent(
        &self,
        session_id: &str,
        prompt: &str,
        worktree_path: &str,
        agent_type: &str,
    ) -> Result<u32, String> {
        if self.is_session_running(session_id) {
            return Err(format!("Session {} already has a running process", session_id));
        }

        // Verify working directory exists
        if !std::path::Path::new(worktree_path).exists() {
            return Err(format!(
                "Working directory does not exist: {}",
                worktree_path
            ));
        }

        tracing::info!(
            "Spawning {} agent for session {} in worktree: {}",
            agent_type,
            session_id,
            worktree_path
        );
        tracing::info!("Prompt: {}", prompt);

        // Spawn the agent based on type
        let mut child = match agent_type {
            "claude" => {
                // Check if claude command is available
                let claude_check = Command::new("which").arg("claude").output().await;
                if claude_check.is_err() || !claude_check.unwrap().status.success() {
                    return Err("The 'claude' command is not found in PATH.".to_string());
                }
                Command::new("script")
                    .arg("-q")
                    .arg("/dev/null")
                    .arg("claude")
                    .arg("-p")
                    .arg(prompt)
                    .arg("--dangerously-skip-permissions")
                    .current_dir(worktree_path)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn claude process: {}", e))?
            }
            "codex" => {
                // Check if codex command is available
                let codex_check = Command::new("which").arg("codex").output().await;
                if codex_check.is_err() || !codex_check.unwrap().status.success() {
                    return Err("The 'codex' command is not found in PATH.".to_string());
                }
                Command::new("codex")
                    .arg("exec")
                    .arg(prompt)
                    .arg("--full-auto")
                    .current_dir(worktree_path)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn codex process: {}", e))?
            }
            "gemini" => {
                // Check if gemini command is available
                let gemini_check = Command::new("which").arg("gemini").output().await;
                if gemini_check.is_err() || !gemini_check.unwrap().status.success() {
                    return Err("The 'gemini' command is not found in PATH.".to_string());
                }
                Command::new("gemini")
                    .arg(prompt)
                    .arg("-y")
                    .current_dir(worktree_path)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn gemini process: {}", e))?
            }
            _ => return Err(format!("Unknown agent type: {}", agent_type)),
        };

        let pid = child.id();

        // Take ownership of streams
        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

        // Create channels for input and kill signal
        let (input_tx, mut input_rx) = mpsc::channel::<String>(100);
        let (kill_tx, mut kill_rx) = mpsc::channel::<()>(1);

        // Store process handle
        let handle = ProcessHandle {
            pid,
            input_tx,
            kill_tx,
        };
        self.session_processes.insert(session_id.to_string(), handle);

        let output_tx = self.session_output_tx.clone();
        let session_processes = Arc::clone(&self.session_processes);
        let session_id_owned = session_id.to_string();

        // Spawn task to handle stdout
        let output_tx_stdout = output_tx.clone();
        let session_id_stdout = session_id_owned.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = output_tx_stdout.send(SessionOutputLine {
                    session_id: session_id_stdout.clone(),
                    line,
                    is_stderr: false,
                });
            }
        });

        // Spawn task to handle stderr
        let output_tx_stderr = output_tx.clone();
        let session_id_stderr = session_id_owned.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = output_tx_stderr.send(SessionOutputLine {
                    session_id: session_id_stderr.clone(),
                    line,
                    is_stderr: true,
                });
            }
        });

        // Spawn task to handle stdin and kill signal
        tokio::spawn(async move {
            let mut stdin = stdin;
            loop {
                tokio::select! {
                    Some(input) = input_rx.recv() => {
                        if let Err(e) = stdin.write_all(input.as_bytes()).await {
                            tracing::error!("Failed to write to stdin: {}", e);
                            break;
                        }
                        let _ = stdin.flush().await;
                    }
                    _ = kill_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Spawn task to wait for process completion and cleanup
        let session_id_cleanup = session_id_owned.clone();
        let output_tx_exit = output_tx.clone();
        tokio::spawn(async move {
            let status = child.wait().await;
            let exit_code = status.ok().and_then(|s| s.code()).unwrap_or(-1);

            let _ = output_tx_exit.send(SessionOutputLine {
                session_id: session_id_cleanup.clone(),
                line: format!("\n[Process exited with code {}]", exit_code),
                is_stderr: false,
            });

            session_processes.remove(&session_id_cleanup);
        });

        Ok(pid.unwrap_or(0))
    }

    /// Send input to a session process
    pub async fn send_session_input(&self, session_id: &str, input: &str) -> Result<(), String> {
        let handle = self
            .session_processes
            .get(session_id)
            .ok_or(format!("No process for session {}", session_id))?;

        handle
            .input_tx
            .send(input.to_string())
            .await
            .map_err(|e| format!("Failed to send input: {}", e))
    }

    /// Kill a session process
    pub async fn kill_session_process(&self, session_id: &str) -> Result<(), String> {
        let handle = self
            .session_processes
            .get(session_id)
            .ok_or(format!("No process for session {}", session_id))?;

        handle
            .kill_tx
            .send(())
            .await
            .map_err(|e| format!("Failed to send kill signal: {}", e))?;

        if let Some(pid) = handle.pid {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output()
                .await;
        }

        Ok(())
    }
}

// Global process manager instance
lazy_static::lazy_static! {
    pub static ref PROCESS_MANAGER: ProcessManager = ProcessManager::new();
}
