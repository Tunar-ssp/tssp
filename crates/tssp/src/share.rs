//! `tssp share` — upload (optional) and publish a durable public file link.

use std::path::Path;

use reqwest::header::ACCEPT;
use serde::Deserialize;
use serde_json::json;
use tssp::{Cli, ShareArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_get, api_patch, build_client, BackendAddress};
use crate::info::path_segment;
use crate::send::upload_file_get_id;
use crate::sessions_helper::generate_qr_code;

#[derive(Debug, Deserialize)]
struct FileMeta {
    visibility: String,
    public_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VisibilityResult {
    public_url: Option<String>,
}

/// Runs `tssp share`.
pub fn run(cli: &Cli, args: &ShareArgs) -> Result<CliExitCode, String> {
    if args.wp {
        return Err(
            "WhatsApp sharing is not implemented yet (use the public URL or QR output)".to_owned(),
        );
    }

    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = build_client()?;

    let file_id = resolve_file_id(&address, &args.target, &args.tags)?;

    let public_url = if args.public {
        ensure_public_url(&client, &address, &file_id)?
    } else {
        fetch_existing_public_url(&client, &address, &file_id)?
            .ok_or_else(|| format!("file {file_id} is not public; pass --public to publish"))?
    };

    if cli.output.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "file_id": file_id,
                "public_url": public_url,
                "target": args.target,
            }))
            .map_err(|e| format!("json encode failed: {e}"))?
        );
    } else {
        if args.qr {
            let qr = generate_qr_code(&public_url, 200)?;
            println!("{qr}");
        }
        println!("Public link:\n{public_url}");
        if !cli.output.quiet {
            eprintln!("\nAnyone with this link can download the file while it stays public.");
        }
    }

    Ok(CliExitCode::Success)
}

fn resolve_file_id(
    address: &BackendAddress,
    target: &str,
    tags: &[String],
) -> Result<String, String> {
    let path = Path::new(target);
    if path.exists() {
        if !path.is_file() {
            return Err(format!("not a file: {}", path.display()));
        }
        eprintln!("Uploading {} ...", path.display());
        return upload_file_get_id(address, path, tags);
    }
    Ok(target.to_owned())
}

fn file_url(address: &BackendAddress, id: &str) -> String {
    format!("{}/{}", address.url("/api/v1/files"), path_segment(id))
}

fn patch_visibility_url(address: &BackendAddress, id: &str) -> String {
    format!("{}/visibility", file_url(address, id))
}

fn fetch_file_meta(
    client: &reqwest::blocking::Client,
    address: &BackendAddress,
    id: &str,
) -> Result<FileMeta, String> {
    #[derive(Deserialize)]
    struct Body {
        visibility: String,
        public_token: Option<String>,
    }

    let response = api_get(client, &file_url(address, id))
        .header(ACCEPT, "application/json")
        .send()
        .map_err(|e| format!("could not reach daemon: {e}"))?;

    if response.status().as_u16() == 404 {
        return Err(format!("file not found: {id}"));
    }
    if !response.status().is_success() {
        return Err(format!(
            "daemon returned {} for file metadata",
            response.status()
        ));
    }

    let body: Body = response
        .json()
        .map_err(|e| format!("invalid file metadata response: {e}"))?;
    Ok(FileMeta {
        visibility: body.visibility,
        public_token: body.public_token,
    })
}

fn fetch_existing_public_url(
    client: &reqwest::blocking::Client,
    address: &BackendAddress,
    id: &str,
) -> Result<Option<String>, String> {
    let meta = fetch_file_meta(client, address, id)?;
    if meta.visibility != "public" {
        return Ok(None);
    }
    let token = meta
        .public_token
        .filter(|t| !t.is_empty())
        .ok_or_else(|| format!("file {id} is public but has no public token"))?;
    Ok(Some(format!("{}/p/{}", address.base_url(), token)))
}

fn ensure_public_url(
    client: &reqwest::blocking::Client,
    address: &BackendAddress,
    id: &str,
) -> Result<String, String> {
    let meta = fetch_file_meta(client, address, id)?;
    if meta.visibility == "public" {
        if let Some(url) = fetch_existing_public_url(client, address, id)? {
            return Ok(url);
        }
    }

    let response = api_patch(client, &patch_visibility_url(address, id))
        .header(ACCEPT, "application/json")
        .header("Content-Type", "application/json")
        .body(r#"{"visibility":"public"}"#.to_owned())
        .send()
        .map_err(|e| format!("could not reach daemon: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let detail = response.text().unwrap_or_default();
        return Err(if detail.is_empty() {
            format!("daemon returned {status} when setting visibility")
        } else {
            format!("daemon returned {status} when setting visibility: {detail}")
        });
    }

    let body: VisibilityResult = response
        .json()
        .map_err(|e| format!("invalid visibility response: {e}"))?;

    body.public_url.ok_or_else(|| {
        "daemon did not return a public_url; set public_url in daemon config".to_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_file_id_passes_through_id_when_missing() {
        let address = BackendAddress::from_connection_args(&tssp::ConnectionArgs {
            host: Some("127.0.0.1".to_owned()),
            port: Some(18421),
        })
        .unwrap_or_else(|e| panic!("address: {e}"));
        let id = resolve_file_id(&address, "file-abc123", &[]).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(id, "file-abc123");
    }

    #[test]
    fn patch_visibility_url_format() {
        let address = BackendAddress::from_connection_args(&tssp::ConnectionArgs {
            host: Some("127.0.0.1".to_owned()),
            port: Some(8421),
        })
        .unwrap_or_else(|e| panic!("address: {e}"));
        assert_eq!(
            patch_visibility_url(&address, "file-1"),
            "http://127.0.0.1:8421/api/v1/files/file-1/visibility"
        );
    }
}
