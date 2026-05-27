//! HTTP integration tests for authentication and authorization.

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::extract::ConnectInfo;
    use axum::http::{Request, StatusCode};
    use std::net::SocketAddr;
    use tower::ServiceExt;
    use tssp_adapter_sqlite::{initialize_connection, SqliteFileRepository};
    use tssp_domain::{UserId, UserName, UserRole};

    use crate::auth::devices::DeviceStore;
    use crate::auth::service::AuthService;
    use crate::auth::store::AuthStore;
    use crate::auth::users::UserStore;
    use crate::status::RepositoryMetadataStatsProvider;
    use crate::{build_router, HttpState};
    use tssp_adapter_system::SystemClock;

    fn auth_app(global: bool) -> (tempfile::TempDir, axum::Router) {
        let temp = tempfile::tempdir().expect("tempdir");
        let db = temp.path().join("metadata.sqlite3");
        let _repo = SqliteFileRepository::open(&db).expect("repo");
        let store = Arc::new(AuthStore::open(&db).expect("auth"));
        let users = Arc::new(UserStore::open(&db).expect("users"));
        let devices = Arc::new(DeviceStore::open(&db).expect("devices"));
        let admin_id = UserId::new("user-tunar").expect("id");
        let admin_name = UserName::new("Tunar").expect("name");
        users
            .create_user(&admin_id, &admin_name, UserRole::Admin, "admin-code", 1_000)
            .expect("admin");
        let user_id = UserId::new("user-alice").expect("id");
        let user_name = UserName::new("Alice").expect("name");
        users
            .create_user(&user_id, &user_name, UserRole::User, "user-code", 1_000)
            .expect("user");
        let auth = AuthService::new(store, users, devices, false, global);
        let repo = SqliteFileRepository::open(&db).expect("repo");
        let state = HttpState::test_http_state(temp.path().join("upload-tmp"))
            .with_stats_provider(Arc::new(RepositoryMetadataStatsProvider::new(
                repo,
                SystemClock,
            )))
            .with_auth(auth);
        (temp, build_router(state))
    }

    #[test]
    fn startup_initializer_prepares_auth_tables_for_direct_store_use() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db = temp.path().join("metadata.sqlite3");
        let _repo = SqliteFileRepository::open(&db).expect("repo");

        let manager = r2d2_sqlite::SqliteConnectionManager::file(&db);
        let pool = r2d2::Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("pool");
        let connection = pool.get().expect("connection");
        initialize_connection(&connection).expect("metadata init");
        crate::auth::initialize_database(&connection).expect("auth init");
        drop(connection);

        let users = UserStore::new(pool.clone());
        let store = AuthStore::new(pool.clone());
        let devices = DeviceStore::new(pool);

        assert_eq!(users.count_users().expect("count"), 0);
        assert!(store.password_hash().expect("hash").is_none());
        assert_eq!(devices.cleanup_expired(1_000).expect("cleanup"), 0);
    }

    #[test]
    fn auth_initialization_succeeds_even_if_metadata_migrations_not_run() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db = temp.path().join("repro.sqlite3");
        let connection = rusqlite::Connection::open(&db).expect("open");

        // This should now succeed because initialize_database ensures schema_migrations exists
        crate::auth::initialize_database(&connection)
            .expect("Initialization should succeed even if schema_migrations is missing initially");

        // Verify that the table was created
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
                [],
                |row| row.get(0),
            )
            .expect("Querying sqlite_master");
        assert_eq!(count, 1, "schema_migrations table should have been created");
    }

    async fn json_request(
        app: &axum::Router,
        method: &str,
        uri: &str,
        body: Option<&str>,
        token: Option<&str>,
    ) -> (StatusCode, String) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }
        let mut request = builder
            .header("Content-Type", "application/json")
            .body(Body::from(body.unwrap_or("").to_owned()))
            .expect("request");
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 8421))));
        let response = app.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    async fn login_token(app: &axum::Router, name: &str, code: &str) -> String {
        let body = format!(r#"{{"name":"{name}","code":"{code}"}}"#);
        let (status, body) =
            json_request(app, "POST", "/api/v1/auth/token", Some(&body), None).await;
        assert_eq!(status, StatusCode::OK);
        serde_json::from_str::<serde_json::Value>(&body)
            .expect("json")
            .get("token")
            .and_then(|v| v.as_str())
            .expect("token")
            .to_owned()
    }

    #[tokio::test]
    async fn admin_route_rejects_normal_user() {
        let (_temp, app) = auth_app(true);
        let token = login_token(&app, "Alice", "user-code").await;
        let (status, _) =
            json_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_route_allows_admin_user() {
        let (_temp, app) = auth_app(true);
        let token = login_token(&app, "Tunar", "admin-code").await;
        let (status, body) =
            json_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Tunar"));
    }

    #[tokio::test]
    async fn local_mode_honors_bearer_tokens_for_context_handlers() {
        let (_temp, app) = auth_app(false);
        let token = login_token(&app, "Tunar", "admin-code").await;

        let (status, body) =
            json_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Tunar"));
    }

    #[tokio::test]
    async fn admin_can_list_and_revoke_sessions() {
        let (_temp, app) = auth_app(true);
        let admin_token = login_token(&app, "Tunar", "admin-code").await;
        let user_token = login_token(&app, "Alice", "user-code").await;

        let (status, body) = json_request(
            &app,
            "GET",
            "/api/v1/admin/sessions?limit=10",
            None,
            Some(&admin_token),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        let sessions = serde_json::from_str::<serde_json::Value>(&body).expect("sessions json");
        let items = sessions
            .get("sessions")
            .and_then(serde_json::Value::as_array)
            .expect("session list");
        assert!(items.iter().any(|item| {
            item.get("token")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|token| token == admin_token)
                && item
                    .get("current")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
        }));
        assert!(items.iter().any(|item| {
            item.get("token")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|token| token == user_token)
        }));

        let path = format!("/api/v1/admin/sessions/{user_token}");
        let (status, _) = json_request(&app, "DELETE", &path, None, Some(&admin_token)).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let (status, _) = json_request(
            &app,
            "GET",
            "/api/v1/files?limit=1",
            None,
            Some(&user_token),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn admin_can_revoke_all_sessions_for_user() {
        let (_temp, app) = auth_app(true);
        let admin_token = login_token(&app, "Tunar", "admin-code").await;
        let user_token_a = login_token(&app, "Alice", "user-code").await;
        let user_token_b = login_token(&app, "Alice", "user-code").await;

        let (status, body) = json_request(
            &app,
            "DELETE",
            "/api/v1/admin/users/user-alice/sessions",
            None,
            Some(&admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("\"removed\":2"));

        let (status, body) = json_request(
            &app,
            "GET",
            "/api/v1/admin/sessions?user_id=user-alice",
            None,
            Some(&admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("\"sessions\":[]"));

        let (status, _) = json_request(
            &app,
            "GET",
            "/api/v1/files?limit=1",
            None,
            Some(&user_token_a),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let (status, _) = json_request(
            &app,
            "GET",
            "/api/v1/files?limit=1",
            None,
            Some(&user_token_b),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn global_mode_requires_auth_for_files_list() {
        let (_temp, app) = auth_app(true);
        let (status, _) = json_request(&app, "GET", "/api/v1/files?limit=1", None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn public_files_list_is_unauthenticated() {
        let (_temp, app) = auth_app(true);
        let (status, _) = json_request(&app, "GET", "/api/v1/public/files", None, None).await;
        assert_eq!(status, StatusCode::OK);
    }
}
