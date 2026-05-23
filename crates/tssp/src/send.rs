//! `tssp send` command implementation.

use std::path::PathBuf;

use serde_json::json;
use tssp::{Cli, SendArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use crate::sessions_helper::generate_qr_code;

pub fn run(cli: &Cli, args: &SendArgs) -> Result<CliExitCode, String> {
    validate_file_exists(&args.file)?;

    let file_path = &args.file;
    eprintln!("Creating send session for {} ...", file_path.display());

    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|e| format!("invalid backend address: {e}"))?;
    let client = build_client()?;

    let req = json!({
        "file_id": "from-cli",
        "ttl_seconds": 86_400
    });

    let response = client
        .post(address.url("/api/v1/sessions/send"))
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

    if let Some(token) = body.get("token").and_then(|v| v.as_str()) {
        let share_url = format!("{}/s/{}", address.base_url(), token);

        if cli.output.json {
            let output = json!({
                "success": true,
                "token": token,
                "share_url": share_url,
                "file": file_path.to_string_lossy(),
            });
            let json_str = serde_json::to_string_pretty(&output)
                .map_err(|e| format!("failed to serialize JSON: {e}"))?;
            println!("{}", json_str);
        } else {
            let qr = generate_qr_code(&share_url, 200)?;
            println!("{}", qr);

            println!("\nShare this URL with others:");
            println!("{}\n", share_url);
            println!("Session token: {}", token);
            println!("Expires in: ~24 hours");

            if !cli.output.quiet {
                eprintln!("Send session created successfully!");
            }
        }

        Ok(CliExitCode::Success)
    } else {
        Err("failed to extract token from session response".to_string())
    }
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
