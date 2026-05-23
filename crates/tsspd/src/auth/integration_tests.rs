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
    use tssp_adapter_sqlite::SqliteFileRepository;
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

    #[tokio::test]
    async fn admin_route_rejects_normal_user() {
        let (_temp, app) = auth_app(true);
        let (status, body) = json_request(
            &app,
            "POST",
            "/api/v1/auth/token",
            Some(r#"{"name":"Alice","code":"user-code"}"#),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let token = serde_json::from_str::<serde_json::Value>(&body)
            .expect("json")
            .get("token")
            .and_then(|v| v.as_str())
            .expect("token")
            .to_owned();
        let (status, _) =
            json_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_route_allows_admin_user() {
        let (_temp, app) = auth_app(true);
        let (status, body) = json_request(
            &app,
            "POST",
            "/api/v1/auth/token",
            Some(r#"{"name":"Tunar","code":"admin-code"}"#),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let token = serde_json::from_str::<serde_json::Value>(&body)
            .expect("json")
            .get("token")
            .and_then(|v| v.as_str())
            .expect("token")
            .to_owned();
        let (status, body) =
            json_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Tunar"));
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
