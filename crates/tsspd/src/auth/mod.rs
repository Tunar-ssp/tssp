//! Authentication: users, roles, trusted devices, and dual-mode access.

mod context;
mod devices;
mod handlers;
mod local_network;
mod middleware;
mod service;
mod store;
mod users;

pub use context::AuthContext;
pub use devices::{DeviceStore, DeviceStoreError, TrustedDevice};
pub use handlers::{
    auth_login, auth_logout, auth_me, auth_required, auth_token, LoginRequest, TokenResponse,
};
pub use middleware::auth_middleware;
pub use service::{AuthError, AuthService, SessionInfo};
pub use store::{AuthStore, AuthStoreError};
pub use users::{UserRecord, UserStore, UserStoreError};
