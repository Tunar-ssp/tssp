//! Admin API for storage, system, and maintenance operations.

mod handlers;
mod sessions;
mod system;
mod users;

pub use handlers::{
    admin_cleanup_sessions, admin_cleanup_temp, admin_corrupt_files, admin_delete_file,
    admin_folders, admin_list_files, admin_overview, admin_system,
};
pub use sessions::{admin_list_devices, admin_revoke_device, admin_revoke_user_devices};
pub use users::{
    admin_create_user, admin_delete_user, admin_list_users, admin_reset_code, admin_set_role,
};
