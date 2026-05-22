//! Application services for backend use cases.
//!
//! Services in this crate orchestrate domain values and ports. They own use-case
//! ordering and cleanup rules but do not know whether persistence is `SQLite`,
//! files are local, or delivery is HTTP.

mod upload;

pub use upload::{UploadError, UploadRequest, UploadResult, UploadService};
