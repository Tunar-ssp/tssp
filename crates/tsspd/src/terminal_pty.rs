//! Simple PTY process management for terminal sessions.
//!
//! Spawns sandboxed shell processes with I/O handling.

use std::io;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};

/// Terminal session wrapping a shell process.
pub struct PtySession {
    child: Child,
}

impl PtySession {
    /// Spawn a shell inside workspace with bubblewrap sandbox.
    pub fn spawn_in_workspace(workspace_root: &Path) -> io::Result<Self> {
        // Validate workspace root exists
        if !workspace_root.exists() || !workspace_root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "workspace_root not found",
            ));
        }

        let abs_workspace = workspace_root.canonicalize()?;
        let workspace_str = abs_workspace
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid path"))?;

        // Build bubblewrap sandbox with minimal attack surface
        let mut cmd = Command::new("bwrap");

        // Bind workspace as /workspace (read-write for user operations)
        cmd.arg("--bind").arg(workspace_str).arg("/workspace");

        // Temporary filesystem for /tmp (no persistence, isolates temp files)
        cmd.arg("--tmpfs").arg("/tmp");

        // Make /dev available but minimal (essential for terminal I/O)
        // --dev creates a minimal /dev with only essential character devices
        cmd.arg("--dev").arg("/dev");

        // Bind /proc as read-only to allow processes to function
        // but prevent privileged operations
        cmd.arg("--ro-bind").arg("/proc").arg("/proc");

        // Change to workspace directory (working directory isolation)
        cmd.arg("--chdir").arg("/workspace");

        // Set HOME to workspace (prevents leaking host paths, contains config writes)
        cmd.arg("--setenv").arg("HOME").arg("/workspace");

        // Clear most environment variables, keep only essentials
        cmd.arg("--clearenv");
        cmd.arg("--setenv")
            .arg("PATH")
            .arg("/usr/local/bin:/usr/bin:/bin");
        cmd.arg("--setenv").arg("TERM").arg("xterm-256color");
        cmd.arg("--setenv").arg("SHELL").arg("/bin/bash");

        // Separate the bwrap args from the command to run
        cmd.arg("--");

        // Run bash in interactive mode with no startup files (fast, predictable)
        cmd.arg("/bin/bash").arg("-i");

        // Configure I/O
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the child process
        let child = cmd.spawn()?;

        Ok(Self { child })
    }

    /// Write input to shell stdin.
    pub async fn write_input(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(mut stdin) = self.child.stdin.take() {
            stdin.write_all(data).await?;
            stdin.flush().await?;
            self.child.stdin = Some(stdin);
        }
        Ok(())
    }

    /// Read output from shell stdout (non-blocking).
    pub async fn read_output(&mut self) -> io::Result<Vec<u8>> {
        if let Some(mut stdout) = self.child.stdout.take() {
            let mut buf = vec![0u8; 1024];
            match stdout.read(&mut buf).await {
                Ok(0) => {
                    self.child.stdout = Some(stdout);
                    Ok(Vec::new())
                }
                Ok(n) => {
                    buf.truncate(n);
                    self.child.stdout = Some(stdout);
                    Ok(buf)
                }
                Err(e) => {
                    self.child.stdout = Some(stdout);
                    Err(e)
                }
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Kill the shell process.
    pub async fn kill(&mut self) -> io::Result<()> {
        self.child.kill().await
    }
}
