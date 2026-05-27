//! Admin backup endpoint.
//!
//! `GET /api/v1/admin/backup` streams a tar archive containing:
//! - A safe online copy of the `SQLite` WAL database via the `rusqlite` backup API.
//! - The entire `blobs/` directory as-is.
//!
//! Admin-only. Uses a background blocking thread; sends chunks over an mpsc channel.

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::stream::StreamExt;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// `GET /api/v1/admin/backup`
///
/// Streams a `.tar` archive with the `SQLite` database snapshot and blobs directory.
/// Requires admin role. Uses a dedicated heavy-task pool to avoid blocking metadata operations.
pub(crate) async fn admin_backup(State(state): State<HttpState>, auth: AuthContext) -> Response {
    if !auth.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(ErrorResponse {
                error: ErrorBody {
                    code: "forbidden",
                    message: "admin role required for backup".to_owned(),
                },
            }),
        )
            .into_response();
    }

    let data_dir = state.settings().data_dir.clone();

    // Channel: blocking task writes tar chunks; async side streams them to the client.
    let (tx, rx) = mpsc::channel::<Result<Vec<u8>, std::io::Error>>(32);

    if let Some(pool) = &state.heavy_task_pool {
        let pool_conn = pool.clone();
        tokio::task::spawn_blocking(move || {
            let _conn = pool_conn.get();
            let result = build_backup_tar(&data_dir, &tx);
            if let Err(e) = result {
                let _ = tx.blocking_send(Err(e));
            }
        });
    } else {
        tokio::task::spawn_blocking(move || {
            let result = build_backup_tar(&data_dir, &tx);
            if let Err(e) = result {
                let _ = tx.blocking_send(Err(e));
            }
        });
    }

    // Convert the mpsc receiver into an async stream.
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(stream.map(|item| item.map(axum::body::Bytes::from)));

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/x-tar"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"tssp-backup.tar\"",
            ),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

/// Builds a tar archive and sends chunks over `tx`.
fn build_backup_tar(
    data_dir: &std::path::Path,
    tx: &mpsc::Sender<Result<Vec<u8>, std::io::Error>>,
) -> Result<(), std::io::Error> {
    let metadata_path = data_dir.join("metadata.db");
    let blob_dir = data_dir.join("blobs");

    let writer = ChannelWriter { tx: tx.clone() };
    let mut builder = tar::Builder::new(writer);

    // 1. SQLite backup via the online backup API (WAL-safe).
    let tmp = tempfile::NamedTempFile::new()?;
    backup_sqlite(&metadata_path, tmp.path())?;
    builder.append_path_with_name(tmp.path(), "metadata.db")?;

    // 2. Walk and archive the blobs directory.
    if blob_dir.exists() {
        for entry in walkdir::WalkDir::new(&blob_dir).follow_links(false) {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let relative = path.strip_prefix(&blob_dir).unwrap_or(path);
            let archive_name = PathBuf::from("blobs").join(relative);
            builder.append_path_with_name(path, &archive_name)?;
        }
    }

    builder.finish()?;
    Ok(())
}

/// Copies the `SQLite` database safely using the `rusqlite` online backup API.
fn backup_sqlite(src: &std::path::Path, dst: &std::path::Path) -> Result<(), std::io::Error> {
    let src_conn = rusqlite::Connection::open(src).map_err(io_other)?;
    let mut dst_conn = rusqlite::Connection::open(dst).map_err(io_other)?;

    let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn).map_err(io_other)?;

    backup
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(io_other)?;

    Ok(())
}

fn io_other(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

/// An [`std::io::Write`] implementation that forwards byte chunks over an mpsc channel.
struct ChannelWriter {
    tx: mpsc::Sender<Result<Vec<u8>, std::io::Error>>,
}

impl std::io::Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tx
            .blocking_send(Ok(buf.to_vec()))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "client closed"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
