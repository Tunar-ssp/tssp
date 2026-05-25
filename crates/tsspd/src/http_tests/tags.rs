//! `tags` route integration tests.

//! HTTP integration tests.

use super::common::*;
use super::imports::*;

#[tokio::test]
async fn tag_endpoints_list_add_and_remove_tags() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let repository = Arc::new(
        SqliteFileRepository::open(temp.path().join("metadata.sqlite3"))
            .unwrap_or_else(|error| panic!("repository open failed: {error}")),
    );
    let storage = Arc::new(
        FilesystemBlobStore::new(temp.path().join("storage"))
            .unwrap_or_else(|error| panic!("blob store open failed: {error}")),
    );
    let stats_provider = RepositoryMetadataStatsProvider::new(repository.clone(), SystemClock);
    let upload_service = UploadService::new(
        storage.clone(),
        repository.clone(),
        UuidV7FileIdGenerator,
        SystemClock,
    );
    let delete_service = DeleteFileService::new(storage.clone(), repository.clone());
    let tag_service = TagService::new(repository);
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_stats_provider(Arc::new(stats_provider))
            .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
            .with_delete_provider(Arc::new(ApplicationFileDeleteProvider::new(delete_service)))
            .with_blob_reader(storage)
            .with_tag_provider(Arc::new(ApplicationFileTagProvider::new(tag_service))),
    );
    let upload = app
        .clone()
        .oneshot(multipart_request(REAL_UPLOAD_BODY))
        .await
        .unwrap_or_else(|error| panic!("upload request failed: {error}"));
    let body = response_json(upload).await;
    let id = body["id"]
        .as_str()
        .unwrap_or_else(|| panic!("uploaded id is missing"));

    let added = app
        .clone()
        .oneshot(add_tags_request(id, r#"["Docs","Family"]"#))
        .await
        .unwrap_or_else(|error| panic!("add tags request failed: {error}"));
    let added_body = response_json(added).await;
    assert_eq!(added_body["changed_count"], 1);

    let listed = app
        .clone()
        .oneshot(tags_request())
        .await
        .unwrap_or_else(|error| panic!("tags request failed: {error}"));
    let listed_body = response_json(listed).await;
    assert_eq!(listed_body["tags"].as_array().map(Vec::len), Some(2));
    assert_eq!(listed_body["tags"][0]["name"], "Docs");
    assert_eq!(listed_body["tags"][1]["name"], "Family");

    let removed = app
        .oneshot(remove_tag_request(id, "Family"))
        .await
        .unwrap_or_else(|error| panic!("remove tag request failed: {error}"));
    let removed_body = response_json(removed).await;
    assert_eq!(removed_body["changed_count"], 1);
}
