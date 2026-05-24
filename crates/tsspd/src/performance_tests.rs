#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use crate::status::calculate_directory_size;
    use std::fs;
    use std::time::Instant;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_directory_size_performance() {
        let temp = tempdir().expect("tempdir failed");
        let root = temp.path();

        // Create 1000 files in 10 directories
        for i in 0..10 {
            let dir = root.join(format!("dir_{i}"));
            fs::create_dir(&dir).expect("create_dir failed");
            for j in 0..100 {
                fs::write(dir.join(format!("file_{j}")), vec![0u8; 1024]).expect("write failed");
            }
        }

        let start = Instant::now();
        let size = calculate_directory_size(root);
        let duration = start.elapsed();

        assert_eq!(size, 1000 * 1024);
        println!("Time taken for 1000 files: {duration:?}");
    }
}
