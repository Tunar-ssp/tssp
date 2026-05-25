//! `PATCH /api/v1/files/{id}/folder` handler for moving one file between logical folders.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_domain::FileId;
use tssp_app::{AuditAction, log_audit_event};

use crate::auth::AuthContext;
use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Deserialize)]
pub struct MoveFileRequest {
    pub folder_path: String,
}

#[derive(Debug, Serialize)]
pub struct MoveFileResponse {
    pub schema_version: u8,
    pub file: FileRecordResponse,
}

/// `PATCH /api/v1/files/{id}/folder`
pub async fn move_file_to_folder(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(request): Json<MoveFileRequest>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return bad_request("invalid_file_id", error.to_string()),
    };

    let folder_path = crate::folders::normalize_folder_path(&request.folder_path);
    if let Err(message) = crate::folders::validate_folder_path(&folder_path) {
        return bad_request("invalid_request", format!("invalid folder path: {message}"));
    }

    let existing = match state.stats_provider.find_file(&file_id) {
        Ok(Some(file)) => file,
        Ok(None) => return not_found(),
        Err(message) => return internal(message),
    };

    if !auth.is_admin() && existing.owner_id.as_ref() != Some(&auth.user_id) {
        return forbidden();
    }

    let repository = state.repository.clone();
    match state
        .stats_provider
        .set_file_folder_path(&file_id, &folder_path)
    {
        Ok(Some(file)) => {
            log_audit_event(
                repository.as_ref(),
                AuditAction::FileMove,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id.as_str()),
                "success",
                Some(&format!("moved to folder {folder_path}")),
            );
            (
                StatusCode::OK,
                Json(MoveFileResponse {
                    schema_version: 1,
                    file: FileRecordResponse::from_record(&file),
                }),
            )
                .into_response()
        }
        Ok(None) => not_found(),
        Err(message) => internal(message),
    }
}

fn bad_request(code: &'static str, message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody { code, message },
        }),
    )
        .into_response()
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "forbidden",
                message: "you do not have permission to move this file".to_owned(),
            },
        }),
    )
        .into_response()
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "not_found",
                message: "file not found".to_owned(),
            },
        }),
    )
        .into_response()
}

fn internal(message: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "internal_error",
                message,
            },
        }),
    )
        .into_response()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use axum::body::to_bytes;
    use serde_json::Value;
    use tssp_domain::{
        ContentHash, FileName, FileRecord, FileSize, MimeType, StorageHandle, UnixTimestamp,
        UserId, UserRole, Visibility,
    };

    use tssp_ports::{ListQuery, PagedFiles, RepositoryStats};

    use super::*;
    use crate::MetadataStatsProvider;

    #[derive(Clone)]
    struct FolderMoveStatsProvider {
        record: Option<FileRecord>,
        updated_folder_path: Option<String>,
    }

    impl MetadataStatsProvider for FolderMoveStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: u64::from(self.record.is_some()),
                note_count: 0,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: u64::from(self.record.is_some()),
                recent_note_count: 0,
                storage_bytes_used: 0,
            })
        }

        fn list_files(&self, _: &ListQuery) -> Result<PagedFiles, String> {
            Ok(PagedFiles {
                files: self.record.clone().into_iter().collect(),
                next_cursor: None,
            })
        }

        fn list_files_recent(&self, _: u64) -> Result<Vec<FileRecord>, String> {
            Ok(self.record.clone().into_iter().collect())
        }

        fn find_file(&self, _: &FileId) -> Result<Option<FileRecord>, String> {
            Ok(self.record.clone())
        }

        fn list_files_by_tag(
            &self,
            _: &tssp_domain::TagKey,
            _: u64,
        ) -> Result<Vec<FileRecord>, String> {
            Ok(Vec::new())
        }

        fn rename_file(&self, _: &FileId, _: &FileName) -> Result<Option<FileRecord>, String> {
            Ok(self.record.clone())
        }

        fn list_folder_counts(
            &self,
            _: Option<&tssp_domain::UserId>,
        ) -> Result<Vec<(String, u64)>, String> {
            Ok(Vec::new())
        }

        fn set_file_visibility(
            &self,
            _: &FileId,
            _: Visibility,
            _: Option<&str>,
        ) -> Result<Option<FileRecord>, String> {
            Ok(self.record.clone())
        }

        fn find_file_by_public_token(&self, _: &str) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }

        fn update_folder_path_prefix(&self, _: &str, _: &str) -> Result<u64, String> {
            Ok(0)
        }

        fn set_file_folder_path(
            &self,
            _: &FileId,
            folder_path: &str,
        ) -> Result<Option<FileRecord>, String> {
            let Some(record) = &self.record else {
                return Ok(None);
            };
            let mut moved = record.clone();
            moved.folder_path = self
                .updated_folder_path
                .clone()
                .unwrap_or_else(|| folder_path.to_owned());
            Ok(Some(moved))
        }
    }

    fn auth(user_id: &str, role: UserRole) -> AuthContext {
        AuthContext {
            user_id: UserId::new(user_id).expect("valid user"),
            role,
            session_token: Some("session".to_owned()),
            device_token: None,
        }
    }

    fn record(owner_id: Option<&str>) -> FileRecord {
        FileRecord {
            id: FileId::new("file-123").expect("id"),
            name: FileName::new("report.txt").expect("name"),
            size: FileSize::new(128),
            content_hash: ContentHash::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123",
            )
            .expect("hash"),
            mime_type: MimeType::new("text/plain").expect("mime"),
            storage_handle: StorageHandle::new("blobs/ab/cd/abcdef").expect("handle"),
            uploaded_at: UnixTimestamp::new(1_700_000_000).expect("timestamp"),
            tags: Vec::new(),
            pinned_at: None,
            folder_path: String::new(),
            owner_id: owner_id.map(|value| UserId::new(value).expect("owner id")),
            visibility: Visibility::Private,
            public_token: None,
            public_expires_at: None,
        }
    }

    async fn body_json(response: Response) -> Value {
        let body = to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read body");
        serde_json::from_slice(&body).expect("json body")
    }

    #[tokio::test]
    async fn move_file_to_folder_updates_folder_path_for_owner() {
        let state = HttpState::test_http_state(std::env::temp_dir()).with_stats_provider(Arc::new(
            FolderMoveStatsProvider {
                record: Some(record(Some("user-owner"))),
                updated_folder_path: Some("projects/tssp".to_owned()),
            },
        ));

        let response = move_file_to_folder(
            State(state),
            auth("user-owner", UserRole::User),
            Path("file-123".to_owned()),
            Json(MoveFileRequest {
                folder_path: "projects/tssp".to_owned(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["file"]["folder_path"], "projects/tssp");
    }

    #[tokio::test]
    async fn move_file_to_folder_rejects_traversal() {
        let state = HttpState::test_http_state(std::env::temp_dir()).with_stats_provider(Arc::new(
            FolderMoveStatsProvider {
                record: Some(record(Some("user-owner"))),
                updated_folder_path: None,
            },
        ));

        let response = move_file_to_folder(
            State(state),
            auth("user-owner", UserRole::User),
            Path("file-123".to_owned()),
            Json(MoveFileRequest {
                folder_path: "../secrets".to_owned(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "invalid_request");
    }

    #[tokio::test]
    async fn move_file_to_folder_rejects_non_owner() {
        let state = HttpState::test_http_state(std::env::temp_dir()).with_stats_provider(Arc::new(
            FolderMoveStatsProvider {
                record: Some(record(Some("user-owner"))),
                updated_folder_path: None,
            },
        ));

        let response = move_file_to_folder(
            State(state),
            auth("user-other", UserRole::User),
            Path("file-123".to_owned()),
            Json(MoveFileRequest {
                folder_path: "projects".to_owned(),
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "forbidden");
    }
}
