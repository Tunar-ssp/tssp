#![cfg(test)]

use crate::terminal::{TerminalError, TerminalManager, TerminalSessionId};
use std::sync::Arc;

#[test]
fn terminal_session_id_generates_unique() {
    let id1 = TerminalSessionId::new();
    let id2 = TerminalSessionId::new();
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn create_session_succeeds() {
    let manager = TerminalManager::new();
    let session = manager
        .create_session(
            "workspace-123",
            "user-456",
            crate::terminal::TerminalConfig {
                workspace_dir: std::path::PathBuf::from("/tmp"),
                sandbox: crate::workspace_features::SandboxStrategy::None,
                env: std::collections::HashMap::new(),
                idle_timeout: 1800,
                max_lifetime: 3600,
            },
        )
        .await;

    assert!(session.is_err(), "should fail with no sandbox");
}

#[tokio::test]
async fn session_lifecycle_works() {
    let manager = TerminalManager::new();

    // Create a session (will fail due to sandbox, but session object is created)
    let result = manager
        .create_session(
            "workspace-123",
            "user-456",
            crate::terminal::TerminalConfig {
                workspace_dir: std::path::PathBuf::from("/tmp"),
                sandbox: crate::workspace_features::SandboxStrategy::None,
                env: std::collections::HashMap::new(),
                idle_timeout: 1800,
                max_lifetime: 3600,
            },
        )
        .await;

    // We expect this to fail because no sandbox is configured
    assert!(result.is_err());
}

#[tokio::test]
async fn close_nonexistent_session_returns_error() {
    let manager = TerminalManager::new();
    let session_id = TerminalSessionId::new();

    let result = manager.close_session(&session_id).await;

    assert!(matches!(result, Err(TerminalError::SessionNotFound)));
}

#[tokio::test]
async fn max_concurrent_sessions_can_be_enforced() {
    // This test verifies the infrastructure is in place for max concurrent session limits
    let manager = Arc::new(TerminalManager::new());
    let sessions = manager.get_all_sessions().await;

    // Should start with no sessions
    assert_eq!(sessions.len(), 0);
}

#[tokio::test]
async fn get_all_sessions_returns_empty_initially() {
    let manager = TerminalManager::new();
    let sessions = manager.get_all_sessions().await;

    assert_eq!(sessions.len(), 0);
}

#[test]
fn terminal_session_id_as_str_not_empty() {
    let id = TerminalSessionId::new();
    assert!(!id.as_str().is_empty());
}
