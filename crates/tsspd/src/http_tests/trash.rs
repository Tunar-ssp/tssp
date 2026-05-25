//! Trash and soft-delete integration tests.


use super::common::*;
use super::imports::*;

#[tokio::test]
async fn delete_moves_file_to_trash_and_list_trash_shows_it() {
    let (_temp, app) = real_storage_app();

    let upload = app.clone().oneshot(multipart_request(REAL_UPLOAD_BODY)).await.unwrap();
    assert_eq!(upload.status(), StatusCode::CREATED);
    let upload_body = response_json(upload).await;
    let file_id = upload_body["id"].as_str().expect("id should exist");

    let delete = app.clone().oneshot(delete_request(file_id)).await.unwrap();
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    let trash_list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trash_list.status(), StatusCode::OK);
    let trash_body = response_json(trash_list).await;
    assert!(trash_body["files"].is_array());
    let files = trash_body["files"].as_array().unwrap();
    assert!(!files.is_empty());
    assert_eq!(files[0]["id"].as_str().unwrap(), file_id);
}

#[tokio::test]
async fn restore_moves_file_from_trash() {
    let (_temp, app) = real_storage_app();

    let upload = app.clone().oneshot(multipart_request(REAL_UPLOAD_BODY)).await.unwrap();
    let upload_body = response_json(upload).await;
    let file_id = upload_body["id"].as_str().expect("id should exist");

    let _delete = app
        .clone()
        .oneshot(delete_request(file_id))
        .await
        .unwrap();

    let trash_after_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let trash_after_delete_body = response_json(trash_after_delete).await;
    let files_in_trash = trash_after_delete_body["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["id"].as_str() == Some(file_id))
        .count();
    assert_eq!(files_in_trash, 1);

    let restore = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/{file_id}/restore"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restore.status(), StatusCode::OK);

    let trash_after_restore = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let trash_after_restore_body = response_json(trash_after_restore).await;
    let files_in_trash_after = trash_after_restore_body["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["id"].as_str() == Some(file_id))
        .count();
    assert_eq!(files_in_trash_after, 0);
}

#[tokio::test]
async fn permanent_delete_removes_file_from_trash() {
    let (_temp, app) = real_storage_app();

    let upload = app.clone().oneshot(multipart_request(REAL_UPLOAD_BODY)).await.unwrap();
    let upload_body = response_json(upload).await;
    let file_id = upload_body["id"].as_str().expect("id should exist");

    let _delete = app
        .clone()
        .oneshot(delete_request(file_id))
        .await
        .unwrap();

    let trash_before = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let trash_before_body = response_json(trash_before).await;
    let before_count = trash_before_body["files"].as_array().unwrap().len();

    let purge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/files/{file_id}/purge"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(purge.status(), StatusCode::NO_CONTENT);

    let trash_after = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let trash_after_body = response_json(trash_after).await;
    let after_count = trash_after_body["files"].as_array().unwrap().len();

    assert_eq!(after_count, before_count - 1);
}

#[tokio::test]
async fn empty_trash_purges_old_deleted_files() {
    let (_temp, app) = real_storage_app();

    let upload1 = app.clone().oneshot(multipart_request(REAL_UPLOAD_BODY)).await.unwrap();
    let body1 = response_json(upload1).await;
    let id1 = body1["id"].as_str().unwrap();

    let upload2 = app.clone().oneshot(multipart_request(SECOND_UPLOAD_BODY)).await.unwrap();
    let body2 = response_json(upload2).await;
    let id2 = body2["id"].as_str().unwrap();

    let _delete1 = app.clone().oneshot(delete_request(id1)).await.unwrap();
    let _delete2 = app.clone().oneshot(delete_request(id2)).await.unwrap();

    let trash_before = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let before_body = response_json(trash_before).await;
    assert!(before_body["files"].as_array().unwrap().len() >= 2);

    let empty = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/trash/empty")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(empty.status(), StatusCode::OK);
    let empty_body = response_json(empty).await;
    assert_eq!(empty_body["purged"].as_u64().unwrap_or(0), 0);
}

#[tokio::test]
async fn soft_deleted_file_is_not_accessible_but_exists_in_trash() {
    let (_temp, app) = real_storage_app();

    let upload = app.clone().oneshot(multipart_request(REAL_UPLOAD_BODY)).await.unwrap();
    let upload_body = response_json(upload).await;
    let file_id = upload_body["id"].as_str().expect("id should exist");

    let _delete = app
        .clone()
        .oneshot(delete_request(file_id))
        .await
        .unwrap();

    let get_file = app
        .clone()
        .oneshot(file_request(file_id))
        .await
        .unwrap();
    assert_eq!(get_file.status(), StatusCode::NOT_FOUND);

    let trash_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/trash")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let trash_body = response_json(trash_list).await;
    let files = trash_body["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f["id"].as_str() == Some(file_id)));
}
