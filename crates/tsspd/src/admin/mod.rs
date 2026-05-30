//! Admin API for storage, system, and maintenance operations.

mod backup;
mod console;
mod editor;
mod handlers;
mod sessions;
mod system;
mod users;

pub(crate) use backup::admin_backup;
pub use console::{list_commands, run_command};
pub(crate) use editor::{
    admin_editor_check, admin_editor_create_document, admin_editor_delete_document,
    admin_editor_get_document, admin_editor_get_workspace, admin_editor_list_documents,
    admin_editor_list_workspaces, admin_editor_update_document,
};
pub use handlers::{
    admin_activity, admin_cleanup_sessions, admin_cleanup_temp, admin_corrupt_files,
    admin_delete_file, admin_folders, admin_list_files, admin_maintenance_prune_logs,
    admin_maintenance_vacuum, admin_overview, admin_status, admin_system,
};
pub use sessions::{
    admin_list_devices, admin_list_sessions, admin_revoke_device, admin_revoke_session,
    admin_revoke_user_devices, admin_revoke_user_sessions,
};
pub use users::{
    admin_create_user, admin_delete_user, admin_list_users, admin_reset_code, admin_set_role,
};
