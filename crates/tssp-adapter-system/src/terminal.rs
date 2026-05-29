use std::io;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tssp_domain::{SandboxStrategy, TerminalConfig};
use tssp_ports::terminal::{PtyProcess, TerminalProvider};

/// Fraction of system memory a terminal session (and its children) may use.
const MEMORY_LIMIT_FRACTION: f64 = 0.80;

/// Reads total system memory (bytes) from `/proc/meminfo`.
fn total_memory_bytes() -> Option<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.trim().trim_end_matches(" kB").trim().parse().ok()?;
            return Some(kb.saturating_mul(1024));
        }
    }
    None
}

/// Computes the per-session address-space cap (80% of RAM), if determinable.
fn memory_limit_bytes() -> Option<u64> {
    total_memory_bytes().map(|total| (total as f64 * MEMORY_LIMIT_FRACTION) as u64)
}

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
                // Wrap bwrap in `prlimit` so the shell and every child it spawns
                // inherit an address-space cap (~80% of RAM). This bounds runaway
                // memory without restricting the user's commands otherwise. The
                // limit is best-effort: if prlimit or meminfo is unavailable we
                // fall back to launching bwrap directly.
                let mem_limit = memory_limit_bytes();
                let use_prlimit = mem_limit.is_some() && which::which("prlimit").is_ok();

                let mut cmd = if use_prlimit {
                    let mut c = Command::new("prlimit");
                    c.arg(format!("--as={}", mem_limit.unwrap_or(0)));
                    c.arg("bwrap");
                    c
                } else {
                    Command::new("bwrap")
                };
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
