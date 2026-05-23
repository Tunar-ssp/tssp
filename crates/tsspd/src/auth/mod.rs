//! Dual-mode authentication (local open, remote password).

mod handlers;
mod local_network;
mod middleware;
mod service;
mod store;

pub use handlers::{
    auth_login, auth_logout, auth_me, auth_required, auth_token, LoginRequest, TokenResponse,
};
pub use middleware::auth_middleware;
pub use service::{AuthError, AuthService};
pub use store::{AuthStore, AuthStoreError};
