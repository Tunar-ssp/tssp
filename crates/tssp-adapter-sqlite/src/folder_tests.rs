#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use crate::SqliteFileRepository;
    use tssp_domain::{
        ContentHash, FileId, FileName, FileSize, MimeType, StorageHandle, UnixTimestamp,
    };
    use tssp_ports::{FileRepository, NewFileRecord};

    fn test_repo() -> SqliteFileRepository {
        SqliteFileRepository::open_in_memory().expect("open in memory")
    }

    fn new_file(id: &str, folder: &str) -> NewFileRecord {
        NewFileRecord {
            id: FileId::new(id).expect("valid id"),
            name: FileName::new("test.txt").expect("valid name"),
            size: FileSize::new(10),
            content_hash: ContentHash::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123",
            )
            .expect("valid hash"),
            mime_type: MimeType::new("text/plain").expect("valid mime"),
            storage_handle: StorageHandle::new("blobs/test").expect("valid handle"),
            uploaded_at: UnixTimestamp::new(1_700_000_000).expect("valid timestamp"),
            tags: vec![],
            pinned_at: None,
            folder_path: folder.to_owned(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
            public_expires_at: None,
        }
    }

    #[test]
    fn test_update_folder_path_prefix_rename() {
        let repo = test_repo();
        repo.insert_file(new_file("f1", "photos/2024"))
            .expect("insert failed");
        repo.insert_file(new_file("f2", "photos/2024/summer"))
            .expect("insert failed");
        repo.insert_file(new_file("f3", "documents"))
            .expect("insert failed");

        let count = repo
            .update_folder_path_prefix("photos/2024", "archive/photos")
            .expect("update failed");
        assert_eq!(count, 2);

        assert_eq!(
            repo.find_file(&FileId::new("f1").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "archive/photos"
        );
        assert_eq!(
            repo.find_file(&FileId::new("f2").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "archive/photos/summer"
        );
        assert_eq!(
            repo.find_file(&FileId::new("f3").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "documents"
        );
    }

    #[test]
    fn test_update_folder_path_prefix_move_to_root() {
        let repo = test_repo();
        repo.insert_file(new_file("f1", "photos/2024"))
            .expect("insert failed");
        repo.insert_file(new_file("f2", "photos/2024/summer"))
            .expect("insert failed");

        let count = repo
            .update_folder_path_prefix("photos/2024", "")
            .expect("update failed");
        assert_eq!(count, 2);

        assert_eq!(
            repo.find_file(&FileId::new("f1").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            ""
        );
        assert_eq!(
            repo.find_file(&FileId::new("f2").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "summer"
        );
    }

    #[test]
    fn test_update_folder_path_prefix_from_root() {
        let repo = test_repo();
        repo.insert_file(new_file("f1", "")).expect("insert failed");
        repo.insert_file(new_file("f2", "sub"))
            .expect("insert failed");

        let count = repo
            .update_folder_path_prefix("", "bucket")
            .expect("update failed");
        assert_eq!(count, 1);

        assert_eq!(
            repo.find_file(&FileId::new("f1").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "bucket"
        );
        assert_eq!(
            repo.find_file(&FileId::new("f2").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "sub"
        );
    }

    #[test]
    fn test_update_folder_path_prefix_trailing_slash_handling() {
        let repo = test_repo();
        repo.insert_file(new_file("f1", "photos/2024"))
            .expect("insert failed");

        let count = repo
            .update_folder_path_prefix("photos/2024/", "archive/")
            .expect("update failed");
        assert_eq!(count, 1);
        assert_eq!(
            repo.find_file(&FileId::new("f1").expect("valid id"))
                .expect("find failed")
                .expect("missing")
                .folder_path,
            "archive"
        );
    }
}
