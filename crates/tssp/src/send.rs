//! `tssp send` command implementation.
//! Uploads a file to the daemon, then creates a send session so the file
//! can be downloaded by a phone via QR code.

use std::path::Path;

use serde_json::json;
use tssp::{Cli, SendArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_post, build_client, BackendAddress};
use crate::sessions_helper::generate_qr_code;

pub fn run(cli: &Cli, args: &SendArgs) -> Result<CliExitCode, String> {
    validate_file_exists(&args.file)?;

    let file_path = &args.file;
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|e| format!("invalid backend address: {e}"))?;

    eprintln!("Uploading {} ...", file_path.display());
    let file_id = upload_file_get_id(&address, file_path, &args.tags)?;

    eprintln!("Creating send session ...");
    let client = build_client()?;
    let req = json!({ "file_id": file_id, "ttl_seconds": 86_400 });

    let response = api_post(&client, &address.url("/api/v1/sessions/send"))
        .json(&req)
        .send()
        .map_err(|e| format!("failed to create send session: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "failed to create send session: {}",
            response.status()
        ));
    }

    let body = response
        .json::<serde_json::Value>()
        .map_err(|e| format!("failed to parse session response: {e}"))?;

    let token = body
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or("failed to extract token from session response")?;

    let share_url = format!("{}/s/{}", address.base_url(), token);

    if cli.output.json {
        let output = json!({
            "success": true,
            "token": token,
            "share_url": share_url,
            "file": file_path.to_string_lossy(),
            "file_id": file_id,
        });
        let json_str = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("failed to serialize JSON: {e}"))?;
        println!("{json_str}");
    } else {
        let qr = generate_qr_code(&share_url, 200)?;
        println!("{qr}");
        println!("\nShare this URL with others:");
        println!("{share_url}\n");
        println!("Session token: {token}");
        println!("Expires in: ~24 hours");
        if !cli.output.quiet {
            eprintln!("Send session created successfully!");
        }
    }

    Ok(CliExitCode::Success)
}

/// Upload a file via multipart to the daemon and return the assigned file ID.
fn upload_file_get_id(
    address: &BackendAddress,
    path: &Path,
    tags: &[String],
) -> Result<String, String> {
    let client = build_client()?;
    let mut form = reqwest::blocking::multipart::Form::new()
        .file("file", path)
        .map_err(|e| format!("could not prepare file for upload: {e}"))?;

    for tag in tags {
        form = form.text("tag", tag.clone());
    }

    let response = api_post(&client, &address.url("/api/v1/files"))
        .multipart(form)
        .send()
        .map_err(|e| format!("upload failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("upload failed: {}", response.status()));
    }

    let body = response
        .json::<serde_json::Value>()
        .map_err(|e| format!("failed to parse upload response: {e}"))?;

    body.get("id")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "upload response missing 'id' field".to_string())
}

fn validate_file_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("not a file: {}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
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
        let result = validate_file_exists(temp.path());
        assert!(matches!(result, Err(e) if e.contains("not a file")));
    }

    #[test]
    fn validate_file_exists_accepts_regular_file() {
        let temp = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, b"test content").unwrap_or_else(|e| panic!("write failed: {e}"));
        assert!(validate_file_exists(&file_path).is_ok());
    }
}
