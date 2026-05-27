#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::path::Path;
    use crate::git::GitService;
    use tssp_ports::GitProvider;
    use tssp_domain::GitStatus;
    use tokio::time::Duration;

    struct SlowGitProvider;

    #[async_trait::async_trait]
    impl GitProvider for SlowGitProvider {
        async fn get_status(&self, _root: &Path) -> Result<GitStatus, String> {
            tokio::time::sleep(Duration::from_secs(10)).await;
            Ok(GitStatus {
                is_repo: true,
                branch: Some("main".into()),
                changed_count: 0,
                staged_count: 0,
                untracked_count: 0,
            })
        }
    }

    #[tokio::test]
    async fn test_git_status_timeout() {
        // Create a temporary directory that exists
        let temp_dir = std::env::temp_dir();

        let provider = Arc::new(SlowGitProvider);
        let service = GitService::new(provider);

        let result: Result<_, String> = service.get_status(&temp_dir).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "git status detection timed out");
    }
}
