//! Clock port trait for injectable time.

use tssp_domain::UnixTimestamp;

/// Supplies server-side UTC timestamps.
pub trait Clock {
    /// Returns the current UTC timestamp.
    fn now(&self) -> UnixTimestamp;
}
