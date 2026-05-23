//! Integration tests for user authentication.

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use tssp_domain::{UserId, UserName, UserRole};

    use crate::auth::devices::DeviceStore;
    use crate::auth::service::AuthService;
    use crate::auth::store::AuthStore;
    use crate::auth::users::UserStore;

    fn service_with_tunar() -> (tempfile::TempDir, AuthService) {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("db.sqlite3");
        rusqlite::Connection::open(&path)
            .expect("open")
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
            )
            .expect("schema");
        let store = Arc::new(AuthStore::open(&path).expect("auth"));
        let users = Arc::new(UserStore::open(&path).expect("users"));
        let devices = Arc::new(DeviceStore::open(&path).expect("devices"));
        let id = UserId::new("user-tunar").expect("id");
        let name = UserName::new("Tunar").expect("name");
        users
            .create_user(&id, &name, UserRole::Admin, "secret-code", 1_000)
            .expect("user");
        (temp, AuthService::new(store, users, devices, false, true))
    }

    #[test]
    fn non_admin_cannot_be_assumed_from_user_login() {
        let (_temp, auth) = service_with_tunar();
        let session = auth
            .authenticate_user("Tunar", "secret-code", "api", false, "", 1_000, None, None)
            .expect("login");
        assert!(session.role == UserRole::Admin);
    }

    #[test]
    fn last_admin_cannot_be_demoted() {
        let (_temp, auth) = service_with_tunar();
        let users = auth.users().expect("users");
        let id = UserId::new("user-tunar").expect("id");

        let result = users.set_role(&id, UserRole::User);

        assert!(result
            .expect_err("last admin demotion must fail")
            .to_string()
            .contains("last admin"));
    }
}
