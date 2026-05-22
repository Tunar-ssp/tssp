//! Stable process exit codes defined by the specification.

/// Stable CLI exit codes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CliExitCode {
    /// Success.
    Success = 0,
    /// Generic uncategorized error.
    Generic = 1,
    /// Usage error such as bad flags or missing arguments.
    Usage = 2,
    /// Configuration error.
    Configuration = 3,
    /// Network error while reaching the daemon.
    Network = 4,
    /// Daemon returned a 5xx response.
    Server = 5,
    /// Requested resource does not exist.
    NotFound = 6,
    /// Duplicate or version conflict.
    Conflict = 7,
    /// Local filesystem denied access.
    PermissionDenied = 8,
    /// User cancelled the operation.
    Cancelled = 9,
    /// Batch operation partially succeeded.
    PartialSuccess = 10,
}

impl CliExitCode {
    /// Converts the stable code into a process exit code.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::CliExitCode;

    #[test]
    fn exit_codes_match_specification() {
        assert_eq!(CliExitCode::Success.as_u8(), 0);
        assert_eq!(CliExitCode::Usage.as_u8(), 2);
        assert_eq!(CliExitCode::PartialSuccess.as_u8(), 10);
    }
}
