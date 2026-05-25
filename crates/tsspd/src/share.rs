//! Public link and QR helpers for shared files.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tssp_domain::{FileId, Visibility};

use crate::auth::AuthContext;
use crate::qr::terminal_qr;
use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Serialize)]
pub struct FileShareResponse {
    pub schema_version: u8,
    pub public_url: String,
    pub qr_terminal: String,
}

fn can_view_share(auth: &AuthContext, owner_id: Option<&tssp_domain::UserId>) -> bool {
    auth.is_admin() || owner_id == Some(&auth.user_id)
}

/// `GET /api/v1/files/{id}/share` — public URL and terminal QR for a public file.
#[allow(clippy::too_many_lines)]
pub async fn get_file_share(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_file_id",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };

    let file = match state.stats_provider.find_file(&file_id) {
        Ok(Some(record)) => record,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "not_found",
                        message: "file not found".to_owned(),
                    },
                }),
            )
                .into_response();
        }
        Err(message) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "internal_error",
                        message,
                    },
                }),
            )
                .into_response();
        }
    };

    if !can_view_share(&auth, file.owner_id.as_ref()) {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "forbidden",
                    message: "you do not have permission to share this file".to_owned(),
                },
            }),
        )
            .into_response();
    }

    if file.visibility != Visibility::Public {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "not_public",
                    message: "file is not public; set visibility to public first".to_owned(),
                },
            }),
        )
            .into_response();
    }

    let Some(token) = file.public_token.as_deref() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "not_public",
                    message: "file has no public link token".to_owned(),
                },
            }),
        )
            .into_response();
    };

    let public_url = state.public_urls().public_file_url(token);
    let qr_terminal = match terminal_qr(&public_url) {
        Ok(qr) => qr,
        Err(message) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "internal_error",
                        message,
                    },
                }),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(FileShareResponse {
            schema_version: 1,
            public_url,
            qr_terminal,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::util::ServiceExt;
    use tssp_domain::{FileId, FileRecord, Visibility};
    use tssp_ports::PagedFiles;

    use crate::router::build_router;
    use crate::state::HttpState;
    use crate::status::MetadataStatsProvider;

    struct PublicFileProvider;

    impl MetadataStatsProvider for PublicFileProvider {
        fn stats(&self) -> Result<tssp_ports::RepositoryStats, String> {
            Ok(tssp_ports::RepositoryStats {
                file_count: 1,
                note_count: 0,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: 0,
                recent_note_count: 0,
                storage_bytes_used: 0,
            })
        }

        fn list_files(&self, _: &tssp_ports::ListQuery) -> Result<PagedFiles, String> {
            Ok(PagedFiles {
                files: Vec::new(),
                next_cursor: None,
            })
        }

        fn list_files_recent(&self, _: u64) -> Result<Vec<FileRecord>, String> {
            Ok(Vec::new())
        }

        fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, String> {
            if id.as_str() != "file-public" {
                return Ok(None);
            }
            Ok(Some(FileRecord {
                id: FileId::new("file-public").expect("id"),
                name: tssp_domain::FileName::new("share-me.txt").expect("name"),
                size: tssp_domain::FileSize::new(1),
                content_hash: tssp_domain::ContentHash::new(
                    "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123",
                )
                .expect("hash"),
                mime_type: tssp_domain::MimeType::new("text/plain").expect("mime"),
                storage_handle: tssp_domain::StorageHandle::new("blobs/ab/cd/abcdef")
                    .expect("handle"),
                uploaded_at: tssp_domain::UnixTimestamp::new(1).expect("ts"),
                tags: Vec::new(),
                pinned_at: None,
                folder_path: String::new(),
                owner_id: None,
                visibility: Visibility::Public,
                public_token: Some("tok123".to_owned()),
                public_expires_at: None,
            }))
        }

        fn list_files_by_tag(
            &self,
            _: &tssp_domain::TagKey,
            _: u64,
        ) -> Result<Vec<FileRecord>, String> {
            Ok(Vec::new())
        }

        fn rename_file(
            &self,
            _: &FileId,
            _: &tssp_domain::FileName,
        ) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }

        fn list_folder_counts(
            &self,
            _owner_id: Option<&tssp_domain::UserId>,
        ) -> Result<Vec<(String, u64)>, String> {
            Ok(Vec::new())
        }

        fn set_file_visibility(
            &self,
            _: &FileId,
            _: Visibility,
            _: Option<&str>,
        ) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }

        fn find_file_by_public_token(&self, _: &str) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }

        fn update_folder_path_prefix(&self, _: &str, _: &str) -> Result<u64, String> {
            Ok(0)
        }

        fn set_file_folder_path(&self, _: &FileId, _: &str) -> Result<Option<FileRecord>, String> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn share_endpoint_returns_qr_for_public_file() {
        let app = build_router(
            HttpState::test_http_state(std::env::temp_dir())
                .with_stats_provider(Arc::new(PublicFileProvider)),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/files/file-public/share")
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("request: {e}")),
            )
            .await
            .unwrap_or_else(|e| panic!("response: {e}"));
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 16_384)
            .await
            .unwrap_or_else(|e| panic!("body: {e}"));
        let parsed: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json: {e}"));
        assert!(parsed["public_url"]
            .as_str()
            .unwrap_or("")
            .contains("/p/tok123"));
        assert!(parsed["qr_terminal"].as_str().unwrap_or("").len() > 10);
    }
}
