//! Terminal session cleanup and timeout enforcement.

use crate::terminal::TerminalManager;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Spawn a background task to cleanup expired terminal sessions.
pub fn spawn_terminal_cleanup(terminal_manager: Arc<TerminalManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_expired_sessions(&terminal_manager).await {
                tracing::warn!("terminal cleanup error: {}", e);
            }
        }
    });
}

/// Cleanup expired terminal sessions based on idle and max lifetime timeouts.
async fn cleanup_expired_sessions(terminal_manager: &Arc<TerminalManager>) -> Result<(), String> {
    let sessions = terminal_manager.get_all_sessions().await;

    let now = SystemTime::now();
    for (session_id, session_state) in sessions {
        // Check idle timeout (30 minutes = 1800 seconds)
        let idle_timeout = Duration::from_secs(1800);
        if let Ok(elapsed) = now.duration_since(session_state.session.last_activity) {
            if elapsed > idle_timeout {
                tracing::info!("closing idle terminal session: {}", session_id.as_str());
                let _ = terminal_manager.close_session(&session_id).await;
                continue;
            }
        }

        // Check max lifetime (1 hour = 3600 seconds)
        let max_lifetime = Duration::from_secs(3600);
        if let Ok(elapsed) = now.duration_since(session_state.created_at) {
            if elapsed > max_lifetime {
                tracing::info!("closing expired terminal session: {}", session_id.as_str());
                let _ = terminal_manager.close_session(&session_id).await;
            }
        }
    }

    Ok(())
}
