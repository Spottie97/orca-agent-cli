use crate::utils::redact::redact_secrets;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct SubprocessResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SubprocessError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("Timed out after {0}s")]
    Timeout(u64),
    #[error("Non-zero exit code: {0}")]
    NonZeroExit(i32),
    #[error("Invalid UTF-8 in output")]
    InvalidUtf8,
    #[error("IO error: {0}")]
    Io(String),
    #[error("Empty output")]
    EmptyOutput,
}

pub async fn run_subprocess(
    command: &str,
    args: &[String],
    stdin_input: Option<&str>,
    working_dir: Option<&std::path::Path>,
    env_vars: &HashMap<String, String>,
    timeout_seconds: u64,
    max_output_bytes: usize,
) -> Result<SubprocessResult, SubprocessError> {
    let start = Instant::now();
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.envs(env_vars);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SubprocessError::CommandNotFound(command.to_string())
            } else {
                SubprocessError::Io(e.to_string())
            }
        })?;

    if let Some(input) = stdin_input {
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| SubprocessError::Io(e.to_string()))?;
        }
    }

    let result = timeout(Duration::from_secs(timeout_seconds), child.wait()).await;

    match result {
        Ok(Ok(status)) => {
            let exit_code = status.code().unwrap_or(-1);
            let stdout = read_pipe(child.stdout, max_output_bytes).await;
            let stderr = read_pipe(child.stderr, max_output_bytes).await;

            let stdout = redact_secrets(&stdout);
            let stderr = redact_secrets(&stderr);

            if stdout.is_empty() && stderr.is_empty() {
                return Err(SubprocessError::EmptyOutput);
            }

            Ok(SubprocessResult {
                stdout,
                stderr,
                exit_code,
                timed_out: false,
                duration_ms: start.elapsed().as_millis() as u64,
            })
        }
        Ok(Err(e)) => Err(SubprocessError::Io(e.to_string())),
        Err(_) => {
            let _ = child.kill().await;
            Err(SubprocessError::Timeout(timeout_seconds))
        }
    }
}

async fn read_pipe<T>(pipe: Option<T>, max_bytes: usize) -> String
where
    T: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    if let Some(mut pipe) = pipe {
        let mut buf = Vec::new();
        if pipe.read_to_end(&mut buf).await.is_err() {
            return String::new();
        }
        if buf.len() > max_bytes {
            buf.truncate(max_bytes);
            let mut s = String::from_utf8_lossy(&buf).to_string();
            s.push_str("\n[truncated]");
            return s;
        }
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subprocess_echo_stdout() {
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "echo hello".to_string()]
        } else {
            vec!["-c".to_string(), "echo hello".to_string()]
        };
        let result = run_subprocess(cmd, &args, None, None, &HashMap::new(), 30, 100_000)
            .await
            .unwrap();
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.exit_code, 0);
        assert!(!result.timed_out);
    }

    #[tokio::test]
    async fn test_subprocess_non_zero_exit() {
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "echo fail && exit 1".to_string()]
        } else {
            vec!["-c".to_string(), "echo fail; exit 1".to_string()]
        };
        let result = run_subprocess(cmd, &args, None, None, &HashMap::new(), 30, 100_000)
            .await
            .unwrap();
        assert!(result.stdout.contains("fail"));
        assert_eq!(result.exit_code, 1);
    }

    #[tokio::test]
    async fn test_subprocess_command_not_found() {
        let result = run_subprocess(
            "definitely_not_a_real_command_12345",
            &[],
            None,
            None,
            &HashMap::new(),
            30,
            100_000,
        )
        .await;
        assert!(matches!(result, Err(SubprocessError::CommandNotFound(_))));
    }

    #[tokio::test]
    async fn test_subprocess_redacts_secrets() {
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec![
                "/C".to_string(),
                "echo secret is sk-abc123xyz456".to_string(),
            ]
        } else {
            vec![
                "-c".to_string(),
                "echo secret is sk-abc123xyz456".to_string(),
            ]
        };
        let result = run_subprocess(cmd, &args, None, None, &HashMap::new(), 30, 100_000)
            .await
            .unwrap();
        assert!(!result.stdout.contains("sk-abc123xyz456"));
        assert!(result.stdout.contains("[REDACTED]"));
    }

    #[tokio::test]
    async fn test_subprocess_stdin_input() {
        if cfg!(windows) {
            return;
        }
        let result = run_subprocess(
            "cat",
            &[],
            Some("hello from stdin"),
            None,
            &HashMap::new(),
            30,
            100_000,
        )
        .await
        .unwrap();
        assert!(result.stdout.contains("hello from stdin"));
        assert_eq!(result.exit_code, 0);
    }
}
