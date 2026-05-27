use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tssp_domain::{
    TerminalConfig, TerminalError, TerminalSession, TerminalSessionId,
};
use tssp_ports::terminal::TerminalProvider;

/// Internal state of a terminal session.
#[derive(Clone)]
pub struct TerminalSessionState {
    /// Domain session model.
    pub session: TerminalSession,
    /// When the process actually started.
    pub started_at: Option<SystemTime>,
}

/// Service for managing terminal session lifecycle and limits.
pub struct TerminalService {
    provider: Arc<dyn TerminalProvider>,
    sessions: Arc<Mutex<HashMap<String, TerminalSessionState>>>,
    max_sessions_per_workspace: usize,
}

impl TerminalService {
    /// Create a new terminal service.
    pub fn new(provider: Arc<dyn TerminalProvider>) -> Self {
        Self {
            provider,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions_per_workspace: 5,
        }
    }

    /// Create a new terminal session.
    pub async fn create_session(
        &self,
        workspace_id: &str,
        user_id: &str,
        config: TerminalConfig,
    ) -> Result<TerminalSession, TerminalError> {
        if workspace_id.is_empty() {
            return Err(TerminalError::Unavailable("workspace_id required".into()));
        }

        if self.max_sessions_per_workspace > 0 {
            let sessions = self.sessions.lock().await;
            let workspace_count = sessions
                .values()
                .filter(|s| s.session.workspace_id == workspace_id)
                .count();
            if workspace_count >= self.max_sessions_per_workspace {
                return Err(TerminalError::Unavailable(format!(
                    "max concurrent terminal sessions reached (limit: {})",
                    self.max_sessions_per_workspace
                )));
            }
        }

        let strategy = self.provider.detect_sandbox_strategy();
        if !strategy.is_available() || strategy != config.sandbox {
            return Err(TerminalError::Unavailable(format!(
                "requested sandbox strategy {:?} not available (detected: {:?})",
                config.sandbox, strategy
            )));
        }

        let session = TerminalSession {
            id: TerminalSessionId::new(uuid::Uuid::new_v4().to_string()),
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(
            session.id.as_str().to_string(),
            TerminalSessionState {
                session: session.clone(),
                started_at: None,
            },
        );

        Ok(session)
    }

    /// Get an existing session.
    pub async fn get_session(&self, id: &TerminalSessionId) -> Result<TerminalSession, TerminalError> {
        let sessions = self.sessions.lock().await;
        sessions
            .get(id.as_str())
            .map(|s| s.session.clone())
            .ok_or(TerminalError::SessionNotFound)
    }

    /// Close a session.
    pub async fn close_session(&self, id: &TerminalSessionId) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.lock().await;
        if sessions.remove(id.as_str()).is_some() {
            Ok(())
        } else {
            Err(TerminalError::SessionNotFound)
        }
    }

    /// Update session activity.
    pub async fn update_activity(&self, id: &TerminalSessionId) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(state) = sessions.get_mut(id.as_str()) {
            state.session.last_activity = SystemTime::now();
            Ok(())
        } else {
            Err(TerminalError::SessionNotFound)
        }
    }

    /// Mark session as started.
    pub async fn mark_started(&self, id: &TerminalSessionId) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(state) = sessions.get_mut(id.as_str()) {
            state.started_at = Some(SystemTime::now());
            Ok(())
        } else {
            Err(TerminalError::SessionNotFound)
        }
    }

    /// Get all sessions.
    pub async fn get_all_sessions(&self) -> Vec<TerminalSessionState> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    /// Cleanup expired sessions.
    pub async fn cleanup_expired_sessions(&self) -> Result<(), String> {
        let sessions = self.get_all_sessions().await;
        let now = SystemTime::now();
        let idle_timeout = Duration::from_secs(1800);
        let max_lifetime = Duration::from_secs(3600);

        for state in sessions {
            let mut should_close = false;

            if let Ok(elapsed) = now.duration_since(state.session.last_activity) {
                if elapsed > idle_timeout {
                    should_close = true;
                }
            }

            if let Ok(elapsed) = now.duration_since(state.session.created_at) {
                if elapsed > max_lifetime {
                    should_close = true;
                }
            }

            if should_close {
                let _ = self.close_session(&state.session.id).await;
            }
        }

        Ok(())
    }

    /// Return the provider.
    pub fn provider(&self) -> &dyn TerminalProvider {
        self.provider.as_ref()
    }
}
