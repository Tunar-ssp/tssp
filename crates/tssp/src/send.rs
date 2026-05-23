//! `tssp send` command implementation.

use std::path::PathBuf;

use tssp::{Cli, SendArgs};
use tssp_cli_core::CliExitCode;

/// Runs the send command.
pub fn run(_cli: &Cli, args: &SendArgs) -> Result<CliExitCode, String> {
    validate_file_exists(&args.file)?;

    eprintln!(
        "creating send session for {} with {} tag(s)",
        args.file.display(),
        args.tags.len()
    );

    eprintln!(
        "send session would be created; implementation pending\ntarget: {}",
        args.file.display()
    );

    Ok(CliExitCode::Success)
}

fn validate_file_exists(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("not a file: {}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_file_exists;
    use std::path::PathBuf;

    #[test]
    fn validate_file_exists_rejects_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = validate_file_exists(&path);
        assert!(matches!(result, Err(e) if e.contains("not found")));
    }

    #[test]
    fn validate_file_exists_rejects_directory() {
        let temp = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
        let result = validate_file_exists(&temp.path().to_path_buf());
        assert!(matches!(result, Err(e) if e.contains("not a file")));
    }

    #[test]
    fn validate_file_exists_accepts_regular_file() {
        let temp = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, b"test content").unwrap_or_else(|e| panic!("write failed: {e}"));

        let result = validate_file_exists(&file_path);
        assert!(result.is_ok());
    }
}
