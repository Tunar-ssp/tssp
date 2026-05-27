use std::io;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tssp_domain::{SandboxStrategy, TerminalConfig};
use tssp_ports::terminal::{PtyProcess, TerminalProvider};

/// Linux implementation of terminal provider using bubblewrap or systemd-nspawn.
pub struct LinuxTerminalProvider;

impl LinuxTerminalProvider {
    /// Create a new linux terminal provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxTerminalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TerminalProvider for LinuxTerminalProvider {
    async fn spawn_pty(
        &self,
        workspace_root: &Path,
        config: &TerminalConfig,
    ) -> io::Result<PtyProcess> {
        // Canonicalize workspace root and verify it exists
        let abs_workspace = workspace_root.canonicalize()?;
        if !abs_workspace.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "workspace root is not a directory",
            ));
        }

        // Verify the requested directory is within the workspace root
        let requested_dir = config.workspace_dir.canonicalize()?;
        if !requested_dir.starts_with(&abs_workspace) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "requested directory is outside of the workspace root",
            ));
        }

        match config.sandbox {
            SandboxStrategy::Bubblewrap => {
                let mut cmd = Command::new("bwrap");
                // Bind workspace as /workspace (read-write)
                cmd.arg("--bind").arg(&abs_workspace).arg("/workspace");
                // Temporary filesystem for /tmp
                cmd.arg("--tmpfs").arg("/tmp");
                // Minimal /dev
                cmd.arg("--dev").arg("/dev");
                // Read-only /proc
                cmd.arg("--ro-bind").arg("/proc").arg("/proc");

                // Change to requested subdirectory relative to /workspace
                let rel_requested = requested_dir
                    .strip_prefix(&abs_workspace)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid path"))?;
                let target_dir = Path::new("/workspace").join(rel_requested);

                cmd.arg("--chdir").arg(target_dir);
                cmd.arg("--setenv").arg("HOME").arg("/workspace");
                cmd.arg("--clearenv");
                cmd.arg("--setenv")
                    .arg("PATH")
                    .arg("/usr/local/bin:/usr/bin:/bin");
                cmd.arg("--setenv").arg("TERM").arg("xterm-256color");
                cmd.arg("--setenv").arg("SHELL").arg("/bin/bash");

                for (key, value) in &config.env {
                    cmd.arg("--setenv").arg(key).arg(value);
                }

                cmd.arg("--");
                cmd.arg("/bin/bash").arg("-i");

                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                let child = cmd.spawn()?;
                Ok(PtyProcess {
                    child: Box::new(child),
                })
            }
            SandboxStrategy::Systemd => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "systemd-nspawn terminal not yet implemented",
            )),
            SandboxStrategy::None => Err(io::Error::other(
                "no sandbox configured, terminal unavailable",
            )),
        }
    }

    fn detect_sandbox_strategy(&self) -> SandboxStrategy {
        if which::which("bwrap").is_ok() {
            return SandboxStrategy::Bubblewrap;
        }

        if which::which("systemd-nspawn").is_ok() {
            return SandboxStrategy::Systemd;
        }

        SandboxStrategy::None
    }
}
