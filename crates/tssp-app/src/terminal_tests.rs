#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::path::Path;
    use crate::terminal::TerminalService;
    use tssp_ports::terminal::{PtyProcess, TerminalProvider};
    use tssp_domain::{SandboxStrategy, TerminalConfig, TerminalError};

    struct MockTerminalProvider;

    #[async_trait::async_trait]
    impl TerminalProvider for MockTerminalProvider {
        async fn spawn_pty(&self, _root: &Path, _config: &TerminalConfig) -> std::io::Result<PtyProcess> {
            Ok(PtyProcess { child: Box::new(()) })
        }
        fn detect_sandbox_strategy(&self) -> SandboxStrategy {
            SandboxStrategy::Bubblewrap
        }
    }

    #[tokio::test]
    async fn test_session_limit_enforcement() {
        let provider = Arc::new(MockTerminalProvider);
        let service = TerminalService::new(provider);

        let config = TerminalConfig {
            workspace_dir: ".".into(),
            sandbox: SandboxStrategy::Bubblewrap,
            env: std::collections::HashMap::new(),
            idle_timeout: 60,
            max_lifetime: 60,
        };

        // Create 5 sessions (default limit)
        for _ in 0..5 {
            let _ = service.create_session("ws1", "user1", config.clone()).await.unwrap();
        }

        // 6th session should fail
        let result: Result<_, TerminalError> = service.create_session("ws1", "user1", config.clone()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max concurrent terminal sessions reached"));
    }
}
