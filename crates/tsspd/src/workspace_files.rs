//! Shelved workspace-file endpoint prototype.
//!
//! This module is intentionally empty. The active workspace API lives in
//! `workspaces.rs` and is mounted under `/api/v1/workspaces`. The previous
//! implementation returned fake success JSON without touching SQLite, which
//! made unimplemented backend behavior look real.
