use std::io;
use std::path::Path;
use tssp_domain::{SandboxStrategy, TerminalConfig};

/// PTY process wrapper to avoid direct dependency on tokio in ports.
pub struct PtyProcess {
    /// Opaque handle to the child process.
    pub child: Box<dyn std::any::Any + Send>,
}

/// Port for managing low-level terminal PTY processes.
#[async_trait::async_trait]
pub trait TerminalProvider: Send + Sync {
    /// Spawns a sandboxed shell process in the given workspace.
    /// Returns an opaque handle to the child process.
    async fn spawn_pty(
        &self,
        workspace_root: &Path,
        config: &TerminalConfig,
    ) -> io::Result<PtyProcess>;

    /// Detects the best available sandbox strategy on the host system.
    fn detect_sandbox_strategy(&self) -> SandboxStrategy;
}
