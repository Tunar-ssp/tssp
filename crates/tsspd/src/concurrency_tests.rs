#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::io::Read;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use tssp_app::DeleteFileService;
    use tssp_domain::{
        ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle,
        UnixTimestamp, Visibility,
    };
    use tssp_ports::{
        BlobStore, BlobStoreError, BlobWriteOutcome, DeletedFileRecord, FileRepository, ListQuery,
        PagedFiles, PinOutcome, RepositoryError, RepositoryStats, TagMutationOutcome, TagSummary,
    };

    struct MockBlobStore {
        cleanup_count: Arc<Mutex<usize>>,
    }

    impl BlobStore for MockBlobStore {
        fn put_stream(&self, _: &mut dyn Read) -> Result<BlobWriteOutcome, BlobStoreError> {
            unimplemented!()
        }
        fn put_staged(
            &self,
            _: &Path,
            _: &ContentHash,
            _: FileSize,
        ) -> Result<BlobWriteOutcome, BlobStoreError> {
            unimplemented!()
        }
        fn cleanup_unreferenced(&self, _: &StorageHandle) -> Result<(), BlobStoreError> {
            let mut lock = self.cleanup_count.lock().expect("lock poisoned");
            *lock += 1;
            Ok(())
        }
    }

    struct MockRepo {
        deleted_count: Arc<Mutex<usize>>,
        record: FileRecord,
    }

    impl FileRepository for MockRepo {
        fn insert_file(&self, _: tssp_ports::NewFileRecord) -> Result<FileRecord, RepositoryError> {
            unimplemented!()
        }
        fn find_file(&self, _: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn find_file_by_content_hash(
            &self,
            _: &ContentHash,
        ) -> Result<Option<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn delete_file(&self, _: &FileId) -> Result<Option<DeletedFileRecord>, RepositoryError> {
            // Simulate a race where multiple threads might enter if not atomic
            let mut lock = self.deleted_count.lock().expect("lock poisoned");
            if *lock == 0 {
                // Simulate work inside transaction
                std::thread::sleep(Duration::from_millis(50));
                *lock += 1;
                Ok(Some(DeletedFileRecord {
                    record: self.record.clone(),
                    remaining_content_references: 0,
                }))
            } else {
                Ok(None)
            }
        }
        fn list_files(&self, _: &ListQuery) -> Result<PagedFiles, RepositoryError> {
            unimplemented!()
        }
        fn list_files_recent(&self, _: u64) -> Result<Vec<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn list_tags(&self) -> Result<Vec<TagSummary>, RepositoryError> {
            unimplemented!()
        }
        fn add_tags_to_file(
            &self,
            _: &FileId,
            _: &[tssp_domain::Tag],
        ) -> Result<TagMutationOutcome, RepositoryError> {
            unimplemented!()
        }
        fn remove_tag_from_file(
            &self,
            _: &FileId,
            _: &tssp_domain::TagKey,
        ) -> Result<TagMutationOutcome, RepositoryError> {
            unimplemented!()
        }
        fn stats_since(&self, _: UnixTimestamp) -> Result<RepositoryStats, RepositoryError> {
            unimplemented!()
        }
        fn pin_file(&self, _: &FileId, _: Option<u32>) -> Result<PinOutcome, RepositoryError> {
            unimplemented!()
        }
        fn unpin_file(&self, _: &FileId) -> Result<PinOutcome, RepositoryError> {
            unimplemented!()
        }
        fn list_pinned_files(&self) -> Result<Vec<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn reorder_pins(&self, _: &[FileId]) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        fn search_files(&self, _: &str) -> Result<Vec<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn list_files_by_tag(
            &self,
            _: &tssp_domain::TagKey,
            _: u64,
        ) -> Result<Vec<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn rename_file(
            &self,
            _: &FileId,
            _: &FileName,
        ) -> Result<Option<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn list_folder_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
            unimplemented!()
        }
        fn set_file_visibility(
            &self,
            _: &FileId,
            _: Visibility,
            _: Option<&str>,
        ) -> Result<Option<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn find_file_by_public_token(
            &self,
            _: &str,
        ) -> Result<Option<FileRecord>, RepositoryError> {
            unimplemented!()
        }
        fn update_folder_path_prefix(&self, _: &str, _: &str) -> Result<u64, RepositoryError> {
            unimplemented!()
        }
        fn set_file_folder_path(
            &self,
            _: &FileId,
            _: &str,
        ) -> Result<Option<FileRecord>, RepositoryError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_concurrent_delete_blob_cleanup() {
        let cleanup_count = Arc::new(Mutex::new(0));
        let blob_store = MockBlobStore {
            cleanup_count: cleanup_count.clone(),
        };

        let record = FileRecord {
            id: FileId::new("file-1").expect("valid id"),
            name: FileName::new("test.txt").expect("valid name"),
            size: FileSize::new(10),
            content_hash: ContentHash::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123",
            )
            .expect("valid hash"),
            mime_type: MimeType::new("text/plain").expect("valid mime"),
            storage_handle: StorageHandle::new("blobs/ab/cd/test").expect("valid handle"),
            uploaded_at: UnixTimestamp::new(1_700_000_000).expect("valid timestamp"),
            tags: vec![],
            pinned_at: None,
            folder_path: String::new(),
            owner_id: None,
            visibility: Visibility::Private,
            public_token: None,
        };

        let repo = MockRepo {
            deleted_count: Arc::new(Mutex::new(0)),
            record: record.clone(),
        };

        let service = Arc::new(DeleteFileService::new(blob_store, repo));

        let mut handles = vec![];
        for _ in 0..10 {
            let s = service.clone();
            handles.push(std::thread::spawn(move || {
                let _ = s.delete(&FileId::new("file-1").expect("valid id"));
            }));
        }

        for h in handles {
            let _ = h.join();
        }

        let final_count = *cleanup_count.lock().expect("lock poisoned");
        println!("Final cleanup count: {final_count}");
        assert_eq!(final_count, 1, "Only one cleanup should be scheduled");
    }
}
