//! `tssp copy` command implementation.

use serde_json::json;
use tssp::{Cli, CopyArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};

pub fn run(cli: &Cli, args: &CopyArgs) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|e| format!("invalid backend address: {e}"))?;

    if args.share {
        eprintln!("Generating share session URL for file {} ...", args.id);

        let client = build_client()?;
        let req = json!({
            "file_id": args.id,
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

            copy_to_clipboard(&share_url)?;
            println!("Share URL copied to clipboard: {}", share_url);

            if !cli.output.quiet {
                eprintln!("Share URL copied successfully!");
            }

            Ok(CliExitCode::Success)
        } else {
            Err("failed to extract token from session response".to_string())
        }
    } else {
        let download_url = format!(
            "{}/api/v1/files/{}/content?disposition=attachment",
            address.base_url(),
            args.id
        );

        copy_to_clipboard(&download_url)?;
        println!("Download URL copied to clipboard: {}", download_url);

        if !cli.output.quiet {
            eprintln!("Download URL copied successfully!");
        }

        Ok(CliExitCode::Success)
    }
}

fn copy_to_clipboard(content: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("failed to access clipboard: {e}"))?;

    clipboard
        .set_text(content)
        .map_err(|e| format!("failed to copy to clipboard: {e}"))
}
