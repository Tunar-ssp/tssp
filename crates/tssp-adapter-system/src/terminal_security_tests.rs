#[cfg(test)]
mod tests {
    use tssp_domain::{SandboxStrategy, TerminalConfig};
    use tssp_ports::terminal::TerminalProvider;
    use crate::terminal::LinuxTerminalProvider;

    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let provider = LinuxTerminalProvider::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_root = temp_dir.path().canonicalize().unwrap();

        // Config with path outside workspace
        let config = TerminalConfig {
            workspace_dir: "/etc".into(),
            sandbox: SandboxStrategy::Bubblewrap,
            env: std::collections::HashMap::new(),
            idle_timeout: 60,
            max_lifetime: 60,
        };

        let result = provider.spawn_pty(&workspace_root, &config).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), std::io::ErrorKind::PermissionDenied);
    }
}
