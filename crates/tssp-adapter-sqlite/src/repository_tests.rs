//! Integration tests for [`SqliteFileRepository`].

#![allow(clippy::unwrap_used, clippy::expect_used)]

use tempfile::tempdir;
use tssp_domain::{
    ContentHash, Cursor, FileId, FileName, FileSize, MimeType, StorageHandle, Tag, TagKey,
    UnixTimestamp,
};
use tssp_ports::{
    FileRepository, ListQuery, ListSort, NewFileRecord, NoteRepository, RepositoryError, SearchHit,
    SessionRepository,
};

use crate::{initialize_connection, SqliteFileRepository, SqliteSessionRepository};

#[test]
fn open_file_database_runs_migrations() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let path = temp.path().join("metadata.sqlite3");

    let repository = SqliteFileRepository::open(path);

    assert!(repository.is_ok());
}

#[test]
fn initialize_connection_prepares_empty_database_for_session_cleanup() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let path = temp.path().join("metadata.sqlite3");
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&path);
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .unwrap_or_else(|error| panic!("pool build failed: {error}"));

    let connection = pool
        .get()
        .unwrap_or_else(|error| panic!("pool get failed: {error}"));
    initialize_connection(&connection)
        .unwrap_or_else(|error| panic!("initialization failed: {error}"));
    drop(connection);

    let repository = SqliteSessionRepository::new(pool);
    let deleted = repository
        .cleanup_expired_sessions(timestamp(1_700_000_000))
        .unwrap_or_else(|error| panic!("cleanup failed: {error}"));

    assert_eq!(deleted, 0);
}

#[test]
fn insert_and_find_file_roundtrips_metadata_and_tags() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    let file = new_file("file-1", &["Docs", "family photos"], 1_700_000_000);

    let inserted = repository
        .insert_file(file)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));
    let found = repository
        .find_file(&inserted.id)
        .unwrap_or_else(|error| panic!("find failed: {error}"));

    assert!(matches!(
        found,
        Some(record) if record.id.as_str() == "file-1"
            && record.name.original() == "report.pdf"
            && record.tags.len() == 2
            && record.pinned_at == Some(2)
    ));
}

#[test]
fn duplicate_file_id_returns_conflict() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &[], 1_700_000_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let duplicate = repository.insert_file(new_file("file-1", &[], 1_700_000_000));

    assert!(matches!(duplicate, Err(RepositoryError::Conflict { .. })));
}

#[test]
fn missing_file_returns_none() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    let missing = repository
        .find_file(&file_id("missing"))
        .unwrap_or_else(|error| panic!("find failed: {error}"));

    assert!(missing.is_none());
}

#[test]
fn find_file_by_content_hash_returns_oldest_matching_record() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-2", &["new"], 2_000))
        .unwrap_or_else(|error| panic!("new insert failed: {error}"));
    repository
        .insert_file(new_file("file-1", &["old"], 1_000))
        .unwrap_or_else(|error| panic!("old insert failed: {error}"));

    let found = repository
        .find_file_by_content_hash(&hash())
        .unwrap_or_else(|error| panic!("hash lookup failed: {error}"));

    assert!(matches!(
        found,
        Some(record) if record.id.as_str() == "file-1"
            && record.tags == vec![tag_value("old")]
    ));
}

#[test]
fn stats_since_counts_files_tags_pins_and_recent_uploads() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("old", &["archive"], 1_000))
        .unwrap_or_else(|error| panic!("old insert failed: {error}"));
    repository
        .insert_file(new_file("new", &["archive", "fresh"], 2_000))
        .unwrap_or_else(|error| panic!("new insert failed: {error}"));

    let stats = repository
        .stats_since(timestamp(1_500))
        .unwrap_or_else(|error| panic!("stats failed: {error}"));

    assert_eq!(stats.file_count, 2);
    assert_eq!(stats.note_count, 0);
    assert_eq!(stats.tag_count, 2);
    assert_eq!(stats.pinned_count, 2);
    assert_eq!(stats.recent_upload_count, 1);
    assert_eq!(stats.recent_note_count, 0);
}

#[test]
fn list_files_recent_returns_newest_first() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("old", &[], 1_000))
        .unwrap_or_else(|error| panic!("old insert failed: {error}"));
    repository
        .insert_file(new_file("middle", &[], 2_000))
        .unwrap_or_else(|error| panic!("middle insert failed: {error}"));
    repository
        .insert_file(new_file("new", &[], 3_000))
        .unwrap_or_else(|error| panic!("new insert failed: {error}"));

    let list = repository
        .list_files_recent(10)
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert_eq!(list.len(), 3);
    assert_eq!(list[0].id.as_str(), "new");
    assert_eq!(list[1].id.as_str(), "middle");
    assert_eq!(list[2].id.as_str(), "old");
}

#[test]
fn list_files_recent_respects_limit() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("1", &[], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));
    repository
        .insert_file(new_file("2", &[], 2_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));
    repository
        .insert_file(new_file("3", &[], 3_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let list = repository
        .list_files_recent(2)
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id.as_str(), "3");
    assert_eq!(list[1].id.as_str(), "2");
}

#[test]
fn list_files_applies_filters_and_cursor_pagination() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));

    let mut earliest = new_file("file-1", &["Docs", "Family"], 1_000);
    earliest.name = filename("report-alpha.png");
    earliest.mime_type = mime_type("image/png");
    earliest.pinned_at = Some(1);
    repository
        .insert_file(earliest)
        .unwrap_or_else(|error| panic!("first insert failed: {error}"));

    let mut second = new_file("file-2", &["Docs", "Family"], 2_000);
    second.name = filename("report-beta.png");
    second.mime_type = mime_type("image/png");
    second.pinned_at = Some(2);
    repository
        .insert_file(second)
        .unwrap_or_else(|error| panic!("second insert failed: {error}"));

    let mut wrong_tags = new_file("file-3", &["Docs"], 1_500);
    wrong_tags.name = filename("report-missing-tag.png");
    wrong_tags.mime_type = mime_type("image/png");
    wrong_tags.pinned_at = Some(3);
    repository
        .insert_file(wrong_tags)
        .unwrap_or_else(|error| panic!("third insert failed: {error}"));

    let mut wrong_mime = new_file("file-4", &["Docs", "Family"], 1_600);
    wrong_mime.name = filename("report-text.txt");
    wrong_mime.mime_type = mime_type("text/plain");
    wrong_mime.pinned_at = Some(4);
    repository
        .insert_file(wrong_mime)
        .unwrap_or_else(|error| panic!("fourth insert failed: {error}"));

    let mut unpinned = new_file("file-5", &["Docs", "Family"], 1_700);
    unpinned.name = filename("report-unpinned.png");
    unpinned.mime_type = mime_type("image/png");
    unpinned.pinned_at = None;
    repository
        .insert_file(unpinned)
        .unwrap_or_else(|error| panic!("fifth insert failed: {error}"));

    let query = ListQuery {
        limit: 1,
        tags: vec![tag_key("Docs"), tag_key("Family")],
        mime_prefix: Some("image".to_owned()),
        name_substring: Some("report".to_owned()),
        since: Some(timestamp(900)),
        until: Some(timestamp(2_100)),
        pinned_only: true,
        sort: ListSort::UploadedAsc,
        ..ListQuery::default()
    };

    let first_page = repository
        .list_files(&query)
        .unwrap_or_else(|error| panic!("first list failed: {error}"));
    assert_eq!(first_page.files.len(), 1);
    assert_eq!(first_page.files[0].id.as_str(), "file-1");
    assert!(first_page.next_cursor.is_some());

    let second_page = repository
        .list_files(&ListQuery {
            after_cursor: first_page.next_cursor,
            ..query
        })
        .unwrap_or_else(|error| panic!("second list failed: {error}"));
    assert_eq!(second_page.files.len(), 1);
    assert_eq!(second_page.files[0].id.as_str(), "file-2");
    assert!(second_page.next_cursor.is_none());
}

#[test]
fn list_files_supports_name_and_size_sorts() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));

    let mut alpha = new_file("alpha", &[], 1_000);
    alpha.name = filename("alpha.txt");
    alpha.size = FileSize::new(20);
    alpha.pinned_at = None;
    repository
        .insert_file(alpha)
        .unwrap_or_else(|error| panic!("alpha insert failed: {error}"));

    let mut gamma = new_file("gamma", &[], 1_100);
    gamma.name = filename("gamma.txt");
    gamma.size = FileSize::new(30);
    gamma.pinned_at = None;
    repository
        .insert_file(gamma)
        .unwrap_or_else(|error| panic!("gamma insert failed: {error}"));

    let mut beta = new_file("beta", &[], 1_200);
    beta.name = filename("beta.txt");
    beta.size = FileSize::new(10);
    beta.pinned_at = None;
    repository
        .insert_file(beta)
        .unwrap_or_else(|error| panic!("beta insert failed: {error}"));

    let by_name = repository
        .list_files(&ListQuery {
            limit: 10,
            sort: ListSort::NameAsc,
            ..ListQuery::default()
        })
        .unwrap_or_else(|error| panic!("name list failed: {error}"));
    assert_eq!(by_name.files.len(), 3);
    assert_eq!(by_name.files[0].name.original(), "alpha.txt");
    assert_eq!(by_name.files[1].name.original(), "beta.txt");
    assert_eq!(by_name.files[2].name.original(), "gamma.txt");

    let by_size = repository
        .list_files(&ListQuery {
            limit: 10,
            sort: ListSort::SizeDesc,
            ..ListQuery::default()
        })
        .unwrap_or_else(|error| panic!("size list failed: {error}"));
    assert_eq!(by_size.files.len(), 3);
    assert_eq!(by_size.files[0].size.bytes(), 30);
    assert_eq!(by_size.files[1].size.bytes(), 20);
    assert_eq!(by_size.files[2].size.bytes(), 10);
}

#[test]
fn list_files_rejects_invalid_cursor() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &[], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let result = repository.list_files(&ListQuery {
        limit: 10,
        sort: ListSort::UploadedAsc,
        after_cursor: Some(
            Cursor::new("ua.bad-value.file-1")
                .unwrap_or_else(|cursor_error| panic!("cursor parse failed: {cursor_error}")),
        ),
        ..ListQuery::default()
    });
    let error = match result {
        Ok(page) => panic!(
            "expected invalid cursor error, got {} files",
            page.files.len()
        ),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        RepositoryError::OperationFailed { message } if message.starts_with("invalid cursor:")
    ));
}

#[test]
fn delete_file_removes_metadata_tags_and_reports_last_reference() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &["docs"], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let deleted = repository
        .delete_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("delete failed: {error}"));
    let stats = repository
        .stats_since(timestamp(0))
        .unwrap_or_else(|error| panic!("stats failed: {error}"));

    assert!(matches!(
        deleted,
        Some(record) if record.record.id.as_str() == "file-1"
            && record.record.tags == vec![tag_value("docs")]
            && record.remaining_content_references == 0
    ));
    assert_eq!(stats.file_count, 0);
    assert_eq!(stats.tag_count, 0);
    assert!(repository
        .find_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("find failed: {error}"))
        .is_none());
}

#[test]
fn delete_file_keeps_shared_tags_and_reports_remaining_references() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &["shared"], 1_000))
        .unwrap_or_else(|error| panic!("first insert failed: {error}"));
    repository
        .insert_file(new_file("file-2", &["shared"], 2_000))
        .unwrap_or_else(|error| panic!("second insert failed: {error}"));

    let deleted = repository
        .delete_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("delete failed: {error}"));
    let stats = repository
        .stats_since(timestamp(0))
        .unwrap_or_else(|error| panic!("stats failed: {error}"));

    assert!(matches!(
        deleted,
        Some(record) if record.remaining_content_references == 1
    ));
    assert_eq!(stats.file_count, 1);
    assert_eq!(stats.tag_count, 1);
}

#[test]
fn delete_missing_file_is_idempotent() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));

    let deleted = repository
        .delete_file(&file_id("missing"))
        .unwrap_or_else(|error| panic!("delete failed: {error}"));

    assert!(deleted.is_none());
}

#[test]
fn list_tags_returns_counts_in_key_order() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &["Beta", "alpha"], 1_000))
        .unwrap_or_else(|error| panic!("first insert failed: {error}"));
    repository
        .insert_file(new_file("file-2", &["beta"], 2_000))
        .unwrap_or_else(|error| panic!("second insert failed: {error}"));

    let tags = repository
        .list_tags()
        .unwrap_or_else(|error| panic!("list tags failed: {error}"));

    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].tag.display(), "alpha");
    assert_eq!(tags[0].file_count, 1);
    assert_eq!(tags[1].tag.display(), "Beta");
    assert_eq!(tags[1].file_count, 2);
}

#[test]
fn add_tags_to_file_is_idempotent_and_normalizes_duplicates() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &["Docs"], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));
    let tags = vec![tag_value("docs"), tag_value("Family")];

    let outcome = repository
        .add_tags_to_file(&file_id("file-1"), &tags)
        .unwrap_or_else(|error| panic!("add tags failed: {error}"));
    let found = repository
        .find_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("find failed: {error}"));

    assert_eq!(outcome.changed_count, 1);
    assert!(matches!(
        found,
        Some(record) if record.tags == vec![tag_value("Docs"), tag_value("Family")]
    ));
}

#[test]
fn tag_mutations_report_missing_file() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    let tags = vec![tag_value("Docs")];

    let add = repository.add_tags_to_file(&file_id("missing"), &tags);
    let remove = repository.remove_tag_from_file(&file_id("missing"), tag_value("Docs").key());

    assert!(matches!(add, Err(RepositoryError::NotFound)));
    assert!(matches!(remove, Err(RepositoryError::NotFound)));
}

#[test]
fn remove_tag_from_file_is_idempotent_and_cleans_orphaned_tag() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &["Docs"], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let first = repository
        .remove_tag_from_file(&file_id("file-1"), tag_value("Docs").key())
        .unwrap_or_else(|error| panic!("remove failed: {error}"));
    let second = repository
        .remove_tag_from_file(&file_id("file-1"), tag_value("Docs").key())
        .unwrap_or_else(|error| panic!("second remove failed: {error}"));
    let tags = repository
        .list_tags()
        .unwrap_or_else(|error| panic!("list tags failed: {error}"));

    assert_eq!(first.changed_count, 1);
    assert_eq!(second.changed_count, 0);
    assert!(tags.is_empty());
}

#[test]
fn update_folder_path_prefix_moves_and_flattens_folders() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));

    for (id, folder) in [("file-a", "photos"), ("file-b", "photos/2024")] {
        let mut record = new_file(id, &[], 1_000);
        record.folder_path = folder.to_owned();
        repository
            .insert_file(record)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
    }

    let renamed = repository
        .update_folder_path_prefix("photos", "archive/photos")
        .unwrap_or_else(|error| panic!("rename failed: {error}"));
    assert_eq!(renamed, 2);

    let file_a = repository
        .find_file(&file_id("file-a"))
        .unwrap_or_else(|error| panic!("find failed: {error}"))
        .expect("file-a");
    let file_b = repository
        .find_file(&file_id("file-b"))
        .unwrap_or_else(|error| panic!("find failed: {error}"))
        .expect("file-b");
    assert_eq!(file_a.folder_path, "archive/photos");
    assert_eq!(file_b.folder_path, "archive/photos/2024");

    let flattened = repository
        .update_folder_path_prefix("archive/photos", "")
        .unwrap_or_else(|error| panic!("flatten failed: {error}"));
    assert_eq!(flattened, 2);
    let file_a = repository
        .find_file(&file_id("file-a"))
        .unwrap_or_else(|error| panic!("find failed: {error}"))
        .expect("file-a");
    let file_b = repository
        .find_file(&file_id("file-b"))
        .unwrap_or_else(|error| panic!("find failed: {error}"))
        .expect("file-b");
    assert_eq!(file_a.folder_path, "");
    assert_eq!(file_b.folder_path, "2024");
}

#[test]
fn pin_file_sets_position_and_returns_changed() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(NewFileRecord {
            id: file_id("file-1"),
            name: filename("report.pdf"),
            size: FileSize::new(42),
            content_hash: hash(),
            mime_type: mime_type("application/pdf"),
            storage_handle: storage_handle(),
            uploaded_at: timestamp(1_000),
            tags: vec![],
            pinned_at: None,
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
            public_expires_at: None,
        })
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let first = repository
        .pin_file(&file_id("file-1"), Some(5))
        .unwrap_or_else(|error| panic!("pin failed: {error}"));
    let second = repository
        .pin_file(&file_id("file-1"), Some(5))
        .unwrap_or_else(|error| panic!("second pin failed: {error}"));

    let list = repository
        .list_pinned_files()
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert!(first.existed);
    assert!(first.changed);
    assert!(second.existed);
    assert!(!second.changed);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id.as_str(), "file-1");
    assert_eq!(list[0].pinned_at, Some(5));
}

#[test]
fn pin_file_inserts_before_existing_positions() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    let mut first = new_file("file-1", &[], 1_000);
    first.pinned_at = None;
    repository
        .insert_file(first)
        .unwrap_or_else(|error| panic!("first insert failed: {error}"));
    let mut second = new_file("file-2", &[], 2_000);
    second.pinned_at = None;
    repository
        .insert_file(second)
        .unwrap_or_else(|error| panic!("second insert failed: {error}"));

    repository
        .pin_file(&file_id("file-1"), None)
        .unwrap_or_else(|error| panic!("first pin failed: {error}"));
    repository
        .pin_file(&file_id("file-2"), Some(1))
        .unwrap_or_else(|error| panic!("second pin failed: {error}"));

    let list = repository
        .list_pinned_files()
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id.as_str(), "file-2");
    assert_eq!(list[0].pinned_at, Some(1));
    assert_eq!(list[1].id.as_str(), "file-1");
    assert_eq!(list[1].pinned_at, Some(2));
}

#[test]
fn unpin_file_clears_position_and_returns_changed() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &[], 1_000)) // new_file pins by default
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let first = repository
        .unpin_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("unpin failed: {error}"));
    let second = repository
        .unpin_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("second unpin failed: {error}"));

    let list = repository
        .list_pinned_files()
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert!(first.existed);
    assert!(first.changed);
    assert!(second.existed);
    assert!(!second.changed);
    assert!(list.is_empty());
}

#[test]
fn unpin_file_compacts_remaining_positions() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    let mut first = new_file("file-1", &[], 1_000);
    first.pinned_at = None;
    repository
        .insert_file(first)
        .unwrap_or_else(|error| panic!("first insert failed: {error}"));
    let mut second = new_file("file-2", &[], 2_000);
    second.pinned_at = None;
    repository
        .insert_file(second)
        .unwrap_or_else(|error| panic!("second insert failed: {error}"));

    repository
        .pin_file(&file_id("file-1"), None)
        .unwrap_or_else(|error| panic!("first pin failed: {error}"));
    repository
        .pin_file(&file_id("file-2"), None)
        .unwrap_or_else(|error| panic!("second pin failed: {error}"));

    repository
        .unpin_file(&file_id("file-1"))
        .unwrap_or_else(|error| panic!("unpin failed: {error}"));

    let list = repository
        .list_pinned_files()
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id.as_str(), "file-2");
    assert_eq!(list[0].pinned_at, Some(1));
}

#[test]
fn reorder_pins_updates_positions() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));
    repository
        .insert_file(new_file("file-1", &[], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));
    repository
        .insert_file(new_file("file-2", &[], 1_000))
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    repository
        .reorder_pins(&[file_id("file-2"), file_id("file-1")])
        .unwrap_or_else(|error| panic!("reorder failed: {error}"));

    let list = repository
        .list_pinned_files()
        .unwrap_or_else(|error| panic!("list failed: {error}"));

    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id.as_str(), "file-2");
    assert_eq!(list[0].pinned_at, Some(1));
    assert_eq!(list[1].id.as_str(), "file-1");
    assert_eq!(list[1].pinned_at, Some(2));
}

#[test]
fn search_files_returns_matching_records() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));

    let mut file1 = new_file("file-1", &["Docs", "Work"], 1_000);
    file1.name = filename("annual_report_2023.pdf");
    repository
        .insert_file(file1)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let mut file2 = new_file("file-2", &["Images"], 1_000);
    file2.name = filename("vacation_photo.jpg");
    repository
        .insert_file(file2)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let mut file3 = new_file("file-3", &["Docs", "Personal"], 1_000);
    file3.name = filename("personal_budget_2023.xlsx");
    repository
        .insert_file(file3)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    // Search by name
    let results = repository
        .search_files("report")
        .unwrap_or_else(|error| panic!("search failed: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id.as_str(), "file-1");

    // Search by tag
    let results = repository
        .search_files("Docs")
        .unwrap_or_else(|error| panic!("search failed: {error}"));
    assert_eq!(results.len(), 2);

    // Search matching across different files
    let results = repository
        .search_files("2023")
        .unwrap_or_else(|error| panic!("search failed: {error}"));
    assert_eq!(results.len(), 2);

    // Search with no matches
    let results = repository
        .search_files("nonexistent")
        .unwrap_or_else(|error| panic!("search failed: {error}"));
    assert!(results.is_empty());
}

#[test]
fn unified_search_ranks_exact_prefix_and_fuzzy_file_hits() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("repository open failed: {error}"));

    let mut exact = new_file("file-exact", &["Finance"], 3_000);
    exact.name = filename("annual_report.pdf");
    repository
        .insert_file(exact)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let mut prefix = new_file("file-prefix", &["Finance"], 2_000);
    prefix.name = filename("reporting-notes.txt");
    repository
        .insert_file(prefix)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let mut fuzzy = new_file("file-fuzzy", &["Finance"], 1_000);
    fuzzy.name = filename("reprot-draft.txt");
    repository
        .insert_file(fuzzy)
        .unwrap_or_else(|error| panic!("insert failed: {error}"));

    let hits = repository
        .search_all("report")
        .unwrap_or_else(|error| panic!("search failed: {error}"));
    let ids = hits
        .iter()
        .filter_map(|hit| match hit {
            SearchHit::File(file) => Some(file.id.as_str().to_owned()),
            SearchHit::Note(_) => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(ids.first().map(String::as_str), Some("file-exact"));
    assert!(ids.iter().any(|id| id == "file-fuzzy"));
}

fn new_file(id: &str, tags: &[&str], uploaded_at: i64) -> NewFileRecord {
    NewFileRecord {
        id: file_id(id),
        name: filename("report.pdf"),
        size: FileSize::new(42),
        content_hash: hash(),
        mime_type: mime_type("application/pdf"),
        storage_handle: storage_handle(),
        uploaded_at: timestamp(uploaded_at),
        tags: tags.iter().map(|tag| tag_value(tag)).collect(),
        pinned_at: Some(2),
        folder_path: String::new(),
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        public_expires_at: None,
    }
}

fn file_id(value: &str) -> FileId {
    FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
}

fn filename(value: &str) -> FileName {
    FileName::new(value).unwrap_or_else(|error| panic!("invalid filename: {error}"))
}

fn hash() -> ContentHash {
    ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
        .unwrap_or_else(|error| panic!("invalid content hash: {error}"))
}

fn mime_type(value: &str) -> MimeType {
    MimeType::new(value).unwrap_or_else(|error| panic!("invalid mime type: {error}"))
}

fn storage_handle() -> StorageHandle {
    StorageHandle::new("blobs/ab/cd/abcdef")
        .unwrap_or_else(|error| panic!("invalid storage handle: {error}"))
}

fn timestamp(seconds: i64) -> UnixTimestamp {
    UnixTimestamp::new(seconds).unwrap_or_else(|error| panic!("invalid timestamp: {error}"))
}

fn tag_value(value: &str) -> Tag {
    Tag::new(value).unwrap_or_else(|error| panic!("invalid tag: {error}"))
}

fn tag_key(value: &str) -> TagKey {
    TagKey::new(value).unwrap_or_else(|error| panic!("invalid tag key: {error}"))
}
