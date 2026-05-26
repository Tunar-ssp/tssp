//! Workspace filesystem operations moved to adapter and app layers.
//!
//! - Trait: `tssp_ports::WorkspaceFileStore`
//! - Implementation: `tssp_adapter_fs::FilesystemWorkspaceFileStore`
//! - Service: `tssp_app::WorkspaceFileService`
//!
//! HTTP handlers should call the app service, not this module.

// This module is kept for reference only. All functionality has been moved
// to the proper architecture layers: ports, adapters, and app.
