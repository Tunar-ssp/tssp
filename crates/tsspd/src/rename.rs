//! PATCH /api/v1/files/{id} handler for renaming files.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tssp_domain::FileName;

use crate::HttpState;

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RenameResponse {
    pub schema_version: u8,
    pub id: String,
    pub file: Value,
}

pub async fn rename_file(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Json(request): Json<RenameRequest>,
) -> Result<(StatusCode, Json<RenameResponse>), (StatusCode, Json<Value>)> {
    let file_id = match tssp_domain::FileId::new(&id) {
        Ok(fid) => fid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema_version": 1,
                    "error": {
                        "code": "invalid_request",
                        "message": "invalid file id"
                    }
                })),
            ))
        }
    };

    let new_name = match FileName::new(&request.name) {
        Ok(fname) => fname,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema_version": 1,
                    "error": {
                        "code": "invalid_request",
                        "message": "invalid filename"
                    }
                })),
            ))
        }
    };

    match state.stats_provider.rename_file(&file_id, &new_name) {
        Ok(Some(record)) => {
            let file_json = json!({
                "id": record.id.as_str(),
                "name": record.name.original(),
                "size": record.size.bytes(),
                "mime": record.mime_type.as_str(),
                "tags": record.tags.iter().map(|t| t.display()).collect::<Vec<_>>(),
                "uploaded": record.uploaded_at.seconds(),
                "pinned": record.pinned_at.is_some(),
            });

            Ok((
                StatusCode::OK,
                Json(RenameResponse {
                    schema_version: 1,
                    id: id.clone(),
                    file: file_json,
                }),
            ))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "schema_version": 1,
                "error": {
                    "code": "not_found",
                    "message": "file not found"
                }
            })),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "schema_version": 1,
                "error": {
                    "code": "internal_error",
                    "message": "rename failed"
                }
            })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use axum::http::StatusCode;
    use axum::Json;
    use axum::extract::{Path, State};
    use tssp_domain::{ContentHash, FileId, FileName, FileRecord, MimeType, StorageHandle, UnixTimestamp};
    use tssp_ports::{ListQuery, PagedFiles, RepositoryStats};

    use super::*;
    use crate::{HttpState, MetadataStatsProvider};

    struct SuccessfulStatsProvider {
        record: FileRecord,
    }

    impl MetadataStatsProvider for SuccessfulStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: 1,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: 1,
            })
        }

        fn list_files(&self, _query: &ListQuery) -> Result<PagedFiles, String> {
            Ok(PagedFiles {
                files: vec![self.record.clone()],
                next_cursor: None,
            })
        }

        fn list_files_recent(&self, _limit: u64) -> Result<Vec<FileRecord>, String> {
            Ok(vec![self.record.clone()])
        }

        fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, String> {
            Ok(Some(self.record.clone()))
        }

        fn list_files_by_tag(
            &self,
            _tag: &tssp_domain::TagKey,
            _limit: u64,
        ) -> Result<Vec<FileRecord>, String> {
            Ok(vec![])
        }

        fn rename_file(
            &self,
            _id: &FileId,
            _new_name: &FileName,
        ) -> Result<Option<FileRecord>, String> {
            Ok(Some(self.record.clone()))
        }
    }

    struct NotFoundStatsProvider;

    impl MetadataStatsProvider for NotFoundStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: 0,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: 0,
            })
        }

        fn list_files(&self, _query: &ListQuery) -> Result<PagedFiles, String> {
            Ok(PagedFiles {
                files: vec![],
                next_cursor: None,
            })
        }

        fn list_files_recent(&self, _limit: u64) -> Result<Vec<FileRecord>, String> {
            Ok(vec![])
        }

        fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }

        fn list_files_by_tag(
            &self,
            _tag: &tssp_domain::TagKey,
            _limit: u64,
        ) -> Result<Vec<FileRecord>, String> {
            Ok(vec![])
        }

        fn rename_file(
            &self,
            _id: &FileId,
            _new_name: &FileName,
        ) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }
    }

    struct ErrorStatsProvider;

    impl MetadataStatsProvider for ErrorStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: 0,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: 0,
            })
        }

        fn list_files(&self, _query: &ListQuery) -> Result<PagedFiles, String> {
            Ok(PagedFiles {
                files: vec![],
                next_cursor: None,
            })
        }

        fn list_files_recent(&self, _limit: u64) -> Result<Vec<FileRecord>, String> {
            Ok(vec![])
        }

        fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }

        fn list_files_by_tag(
            &self,
            _tag: &tssp_domain::TagKey,
            _limit: u64,
        ) -> Result<Vec<FileRecord>, String> {
            Ok(vec![])
        }

        fn rename_file(
            &self,
            _id: &FileId,
            _new_name: &FileName,
        ) -> Result<Option<FileRecord>, String> {
            Err("database locked".to_owned())
        }
    }

    fn test_record() -> FileRecord {
        FileRecord {
            id: FileId::new("test-file-id-00000000").unwrap(),
            name: FileName::new("newname.txt").unwrap(),
            size: tssp_domain::FileSize::new(1024),
            content_hash: ContentHash::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123"
                    .to_owned(),
            )
            .unwrap(),
            mime_type: MimeType::new("text/plain").unwrap(),
            tags: vec![],
            uploaded_at: UnixTimestamp::new(1000000000).unwrap(),
            pinned_at: None,
            storage_handle: StorageHandle::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123"
                    .to_owned(),
            )
            .unwrap(),
        }
    }

    #[tokio::test]
    async fn rename_file_returns_ok_with_renamed_record() {
        let state = HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"))
            .with_stats_provider(Arc::new(SuccessfulStatsProvider {
                record: test_record(),
            }));

        let response = rename_file(
            State(state),
            Path("test-file-id-00000000".to_string()),
            Json(RenameRequest {
                name: "newname.txt".to_string(),
            }),
        )
        .await;

        assert!(response.is_ok());
        let (status, body) = response.unwrap();
        assert_eq!(status, StatusCode::OK);
        let response = body.0;
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.id, "test-file-id-00000000");
        assert_eq!(response.file["name"], "newname.txt");
    }

    #[tokio::test]
    async fn rename_file_returns_bad_request_for_invalid_id() {
        let state =
            HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"));

        let response = rename_file(
            State(state),
            Path("invalid@file#id".to_string()),
            Json(RenameRequest {
                name: "newname.txt".to_string(),
            }),
        )
        .await;

        assert!(response.is_err());
        let (status, body) = response.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let response = body.0;
        assert_eq!(response["error"]["code"], "invalid_request");
        assert_eq!(response["error"]["message"], "invalid file id");
    }

    #[tokio::test]
    async fn rename_file_returns_bad_request_for_invalid_filename() {
        let state =
            HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"));

        let response = rename_file(
            State(state),
            Path("test-file-id-00000000".to_string()),
            Json(RenameRequest {
                name: "".to_string(),
            }),
        )
        .await;

        assert!(response.is_err());
        let (status, body) = response.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let response = body.0;
        assert_eq!(response["error"]["code"], "invalid_request");
        assert_eq!(response["error"]["message"], "invalid filename");
    }

    #[tokio::test]
    async fn rename_file_returns_not_found_when_file_missing() {
        let state = HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"))
            .with_stats_provider(Arc::new(NotFoundStatsProvider));

        let response = rename_file(
            State(state),
            Path("test-file-id-00000000".to_string()),
            Json(RenameRequest {
                name: "newname.txt".to_string(),
            }),
        )
        .await;

        assert!(response.is_err());
        let (status, body) = response.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        let response = body.0;
        assert_eq!(response["error"]["code"], "not_found");
        assert_eq!(response["error"]["message"], "file not found");
    }

    #[tokio::test]
    async fn rename_file_returns_internal_error_on_stats_provider_error() {
        let state = HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"))
            .with_stats_provider(Arc::new(ErrorStatsProvider));

        let response = rename_file(
            State(state),
            Path("test-file-id-00000000".to_string()),
            Json(RenameRequest {
                name: "newname.txt".to_string(),
            }),
        )
        .await;

        assert!(response.is_err());
        let (status, body) = response.unwrap_err();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let response = body.0;
        assert_eq!(response["error"]["code"], "internal_error");
        assert_eq!(response["error"]["message"], "rename failed");
    }
}
