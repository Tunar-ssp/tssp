//! Daemon startup initialization and cleanup routines.

use tssp_app::SessionService;
use tssp_ports::{Clock, SessionRepository};

/// Startup service for daemon initialization tasks.
#[allow(dead_code)]
pub struct StartupService<R: SessionRepository, C: Clock> {
    session_service: SessionService<R>,
    clock: C,
}

impl<R: SessionRepository, C: Clock> StartupService<R, C> {
    /// Creates a new startup service.
    #[allow(dead_code)]
    pub fn new(session_service: SessionService<R>, clock: C) -> Self {
        Self {
            session_service,
            clock,
        }
    }

    /// Runs startup cleanup tasks.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup cannot be completed.
    #[allow(dead_code)]
    pub fn run_cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.clock.now();
        let deleted = self.session_service.cleanup_expired_sessions(now)?;

        if deleted > 0 {
            eprintln!("cleanup: removed {} expired sessions", deleted);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tssp_domain::SessionKind;
    use tssp_ports::RepositoryError;

    struct MockSessionRepository {
        sessions: Arc<std::sync::Mutex<Vec<tssp_domain::TransferSession>>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }
    }

    impl SessionRepository for MockSessionRepository {
        fn create_session(
            &self,
            session: tssp_domain::TransferSession,
        ) -> Result<(), RepositoryError> {
            let mut sessions = self.sessions.lock().expect("lock poisoned");
            sessions.push(session);
            Ok(())
        }

        fn find_session(
            &self,
            _token: &tssp_domain::SessionToken,
        ) -> Result<Option<tssp_domain::TransferSession>, RepositoryError> {
            Ok(None)
        }

        fn mark_session_used(
            &self,
            _token: &tssp_domain::SessionToken,
            _used_at: tssp_domain::UnixTimestamp,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn cleanup_expired_sessions(
            &self,
            _before: tssp_domain::UnixTimestamp,
        ) -> Result<u64, RepositoryError> {
            Ok(0)
        }

        fn list_sessions_by_kind(
            &self,
            _kind: SessionKind,
        ) -> Result<Vec<tssp_domain::TransferSession>, RepositoryError> {
            Ok(Vec::new())
        }
    }

    struct MockClock;

    impl Clock for MockClock {
        fn now(&self) -> tssp_domain::UnixTimestamp {
            tssp_domain::UnixTimestamp::new(1000).expect("invalid timestamp")
        }
    }

    #[test]
    fn startup_service_can_be_created() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);
        let clock = MockClock;
        let startup = StartupService::new(service, clock);

        // Just verify it was created successfully
        let _ = startup;
    }

    #[test]
    fn run_cleanup_succeeds() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);
        let clock = MockClock;
        let startup = StartupService::new(service, clock);

        let result = startup.run_cleanup();
        assert!(result.is_ok());
    }
}
