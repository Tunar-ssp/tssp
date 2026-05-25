//! Audit event logging for compliance and forensics.

use tssp_domain::UserId;
use tssp_ports::FileRepository;
use uuid::Uuid;

/// Audit event action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    /// User login event.
    Login,
    /// User logout event.
    Logout,
    /// Device revocation event.
    DeviceRevoke,
    /// Session revocation event.
    SessionRevoke,
    /// File upload event.
    FileUpload,
    /// File deletion event.
    FileDelete,
    /// File restoration event.
    FileRestore,
    /// File share event.
    FileShare,
    /// File visibility change event.
    FileVisibilityChange,
    /// File move event.
    FileMove,
    /// File rename event.
    FileRename,
    /// File tag event.
    FileTag,
    /// File pinned event.
    FilePinned,
    /// File unpinned event.
    FileUnpinned,
    /// Note creation event.
    NoteCreate,
    /// Note update event.
    NoteUpdate,
    /// Note deletion event.
    NoteDelete,
    /// Note tag event.
    NoteTag,
    /// Admin user creation event.
    AdminUserCreate,
    /// Admin user deletion event.
    AdminUserDelete,
    /// Admin code reset event.
    AdminCodeReset,
    /// Admin console command event.
    AdminConsoleCommand,
}

impl AuditAction {
    /// Get the string representation of this audit action.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Logout => "logout",
            Self::DeviceRevoke => "device_revoke",
            Self::SessionRevoke => "session_revoke",
            Self::FileUpload => "file_upload",
            Self::FileDelete => "file_delete",
            Self::FileRestore => "file_restore",
            Self::FileShare => "file_share",
            Self::FileVisibilityChange => "file_visibility_change",
            Self::FileMove => "file_move",
            Self::FileRename => "file_rename",
            Self::FileTag => "file_tag",
            Self::FilePinned => "file_pinned",
            Self::FileUnpinned => "file_unpinned",
            Self::NoteCreate => "note_create",
            Self::NoteUpdate => "note_update",
            Self::NoteDelete => "note_delete",
            Self::NoteTag => "note_tag",
            Self::AdminUserCreate => "admin_user_create",
            Self::AdminUserDelete => "admin_user_delete",
            Self::AdminCodeReset => "admin_code_reset",
            Self::AdminConsoleCommand => "admin_console_command",
        }
    }
}

/// Log an audit event for compliance and forensics.
///
/// Failures to insert the audit event are logged but do not halt operations.
/// This ensures that audit logging failures never break core functionality.
pub fn log_audit_event(
    repository: &dyn FileRepository,
    action: AuditAction,
    user_id: Option<&UserId>,
    resource: Option<&str>,
    resource_id: Option<&str>,
    status: &str,
    details: Option<&str>,
) {
    let event_id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    if let Err(e) = repository.insert_audit_event(
        &event_id,
        now,
        user_id.map(|id| id.as_str()),
        action.as_str(),
        resource,
        resource_id,
        status,
        details,
    ) {
        tracing::warn!("failed to log audit event: {}", e);
    }
}
