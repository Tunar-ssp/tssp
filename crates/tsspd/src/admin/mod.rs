//! Admin API for storage, system, and maintenance operations.

mod handlers;
mod system;

pub use handlers::{
    admin_cleanup_sessions, admin_cleanup_temp, admin_corrupt_files, admin_delete_file,
    admin_folders, admin_list_files, admin_overview, admin_system,
};
