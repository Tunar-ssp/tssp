//! Authentication: users, roles, trusted devices, and dual-mode access.

mod context;
mod devices;
mod handlers;
#[cfg(test)]
mod integration_tests;
mod local_network;
mod middleware;
mod service;
mod store;
mod users;
#[cfg(test)]
mod users_test;

pub use context::{AuthContext, OptionalAuthContext};
pub use devices::{DeviceStore, DeviceStoreError, TrustedDevice};
pub use handlers::{
    auth_login, auth_logout, auth_me, auth_required, auth_token, LoginRequest, TokenResponse,
};
pub use middleware::auth_middleware;
pub use service::{AuthError, AuthService, SessionInfo};
pub use store::{AuthStore, AuthStoreError};
pub use users::UserRecord;
pub use users::{UserStore, UserStoreError};
