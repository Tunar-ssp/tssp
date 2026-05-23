//! `tssp admin` — storage and host management commands.

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use tssp_cli_core::CliExitCode;

use crate::backend::{api_delete, api_get, api_post, api_put, build_client, BackendAddress};
use tssp::{
    AdminAction, AdminCommand, AdminDevicesAction, AdminDevicesCommand, AdminFilesArgs,
    AdminUserCreateArgs, AdminUserResetCodeArgs, AdminUserRoleArgs, AdminUsersAction,
    AdminUsersCommand, Cli,
};

pub(crate) fn run(cli: &Cli, command: &AdminCommand) -> Result<CliExitCode, String> {
    match &command.action {
        AdminAction::Overview => get_json(cli, "/api/v1/admin/overview", "overview"),
        AdminAction::System => get_json(cli, "/api/v1/admin/system", "system"),
        AdminAction::Files(args) => admin_files(cli, args),
        AdminAction::Folders => get_json(cli, "/api/v1/admin/folders", "folders"),
        AdminAction::Delete(args) => admin_delete(cli, &args.id),
        AdminAction::Corrupt => get_json(cli, "/api/v1/admin/corrupt", "corrupt"),
        AdminAction::CleanupTemp => post_json(cli, "/api/v1/admin/cleanup/temp", "cleanup"),
        AdminAction::CleanupSessions => post_json(cli, "/api/v1/admin/cleanup/sessions", "cleanup"),
        AdminAction::Users(cmd) => admin_users(cli, cmd),
        AdminAction::Devices(cmd) => admin_devices(cli, cmd),
    }
}

fn admin_users(cli: &Cli, command: &AdminUsersCommand) -> Result<CliExitCode, String> {
    match &command.action {
        AdminUsersAction::List => admin_users_list(cli),
        AdminUsersAction::Create(args) => admin_user_create(cli, args),
        AdminUsersAction::Delete(args) => admin_user_delete(cli, &args.id),
        AdminUsersAction::SetRole(args) => admin_user_set_role(cli, args),
        AdminUsersAction::ResetCode(args) => admin_user_reset_code(cli, args),
    }
}

fn admin_devices(cli: &Cli, command: &AdminDevicesCommand) -> Result<CliExitCode, String> {
    match &command.action {
        AdminDevicesAction::List => admin_devices_list(cli),
        AdminDevicesAction::Revoke(args) => admin_device_revoke(cli, &args.token),
        AdminDevicesAction::RevokeUser(args) => admin_user_devices_revoke(cli, &args.id),
    }
}

fn admin_users_list(cli: &Cli) -> Result<CliExitCode, String> {
    let body = fetch(
        cli,
        |client, url| api_get(client, url).header(ACCEPT, "application/json"),
        "/api/v1/admin/users",
    )?;
    if cli.output.json {
        println!("{body}");
        return Ok(CliExitCode::Success);
    }
    let parsed: UserListResponse =
        serde_json::from_str(&body).map_err(|e| format!("invalid users response: {e}"))?;
    if parsed.users.is_empty() {
        if !cli.output.quiet {
            println!("No users.");
        }
        return Ok(CliExitCode::Success);
    }
    for user in &parsed.users {
        let disabled = if user.disabled { " disabled" } else { "" };
        println!("{}  {}  {}{}", user.id, user.name, user.role, disabled);
    }
    Ok(CliExitCode::Success)
}

fn admin_user_create(cli: &Cli, args: &AdminUserCreateArgs) -> Result<CliExitCode, String> {
    let payload = serde_json::json!({
        "name": args.name,
        "code": args.code,
        "role": args.role,
    });
    let body = fetch_with_body(
        cli,
        |client, url| {
            api_post(client, url)
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .body(payload.to_string())
        },
        "/api/v1/admin/users",
    )?;
    Ok(print_body(cli, &body))
}

fn admin_user_delete(cli: &Cli, id: &str) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/users/{id}");
    fetch(
        cli,
        |client, url| api_delete(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    if !cli.output.quiet && !cli.output.json {
        println!("User {id} deleted.");
    }
    Ok(CliExitCode::Success)
}

fn admin_user_set_role(cli: &Cli, args: &AdminUserRoleArgs) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/users/{}/role", args.id);
    let payload = serde_json::json!({ "role": args.role });
    fetch_with_body(
        cli,
        |client, url| {
            api_put(client, url)
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .body(payload.to_string())
        },
        &path,
    )?;
    if !cli.output.quiet && !cli.output.json {
        println!("Role for {} set to {}.", args.id, args.role);
    }
    Ok(CliExitCode::Success)
}

fn admin_user_reset_code(cli: &Cli, args: &AdminUserResetCodeArgs) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/users/{}/reset-code", args.id);
    let payload = serde_json::json!({ "code": args.code });
    fetch_with_body(
        cli,
        |client, url| {
            api_post(client, url)
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .body(payload.to_string())
        },
        &path,
    )?;
    if !cli.output.quiet && !cli.output.json {
        println!("Access code reset for {}.", args.id);
    }
    Ok(CliExitCode::Success)
}

fn admin_devices_list(cli: &Cli) -> Result<CliExitCode, String> {
    let body = fetch(
        cli,
        |client, url| api_get(client, url).header(ACCEPT, "application/json"),
        "/api/v1/admin/devices",
    )?;
    if cli.output.json {
        println!("{body}");
        return Ok(CliExitCode::Success);
    }
    let parsed: DeviceListResponse =
        serde_json::from_str(&body).map_err(|e| format!("invalid devices response: {e}"))?;
    if parsed.devices.is_empty() {
        if !cli.output.quiet {
            println!("No trusted devices.");
        }
        return Ok(CliExitCode::Success);
    }
    for device in &parsed.devices {
        println!(
            "{}  {}  {}  expires {}",
            device.device_token, device.user_id, device.device_name, device.expires_at
        );
    }
    Ok(CliExitCode::Success)
}

fn admin_device_revoke(cli: &Cli, token: &str) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/devices/{token}");
    fetch(
        cli,
        |client, url| api_delete(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    if !cli.output.quiet && !cli.output.json {
        println!("Device revoked.");
    }
    Ok(CliExitCode::Success)
}

fn admin_user_devices_revoke(cli: &Cli, user_id: &str) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/users/{user_id}/devices");
    fetch(
        cli,
        |client, url| api_delete(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    if !cli.output.quiet && !cli.output.json {
        println!("All devices revoked for {user_id}.");
    }
    Ok(CliExitCode::Success)
}

fn get_json(cli: &Cli, path: &str, _label: &str) -> Result<CliExitCode, String> {
    let body = fetch(
        cli,
        |client, url| api_get(client, url).header(ACCEPT, "application/json"),
        path,
    )?;
    Ok(print_body(cli, &body))
}

fn post_json(cli: &Cli, path: &str, _label: &str) -> Result<CliExitCode, String> {
    let body = fetch(
        cli,
        |client, url| api_post(client, url).header(ACCEPT, "application/json"),
        path,
    )?;
    Ok(print_body(cli, &body))
}

fn admin_files(cli: &Cli, args: &AdminFilesArgs) -> Result<CliExitCode, String> {
    let mut path = format!("/api/v1/admin/files?limit={}", args.limit);
    if let Some(folder) = &args.folder {
        path.push_str("&folder=");
        path.push_str(&encode_query_component(folder));
    }
    if let Some(mime) = &args.mime_type {
        path.push_str("&type=");
        path.push_str(&encode_query_component(mime));
    }
    let body = fetch(
        cli,
        |client, url| api_get(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    if cli.output.json {
        println!("{body}");
        return Ok(CliExitCode::Success);
    }
    let parsed: AdminFilesResponse =
        serde_json::from_str(&body).map_err(|e| format!("invalid admin files response: {e}"))?;
    if parsed.files.is_empty() {
        if !cli.output.quiet {
            println!("No files.");
        }
        return Ok(CliExitCode::Success);
    }
    for file in &parsed.files {
        let folder = if file.folder_path.is_empty() {
            String::new()
        } else {
            format!("  folder: {}", file.folder_path)
        };
        let pin = if file.pinned { " pinned" } else { "" };
        println!(
            "{}  {}  {} bytes  {}{}",
            file.id, file.name, file.size_bytes, file.mime_type, pin
        );
        if !folder.is_empty() && !cli.output.quiet {
            println!("{folder}");
        }
    }
    Ok(CliExitCode::Success)
}

fn admin_delete(cli: &Cli, id: &str) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/files/{id}");
    let body = fetch(
        cli,
        |client, url| api_delete(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    Ok(print_body(cli, &body))
}

fn fetch<F>(cli: &Cli, build: F, path: &str) -> Result<String, String>
where
    F: FnOnce(&reqwest::blocking::Client, &str) -> reqwest::blocking::RequestBuilder,
{
    fetch_with_body(cli, build, path)
}

fn fetch_with_body<F>(cli: &Cli, build: F, path: &str) -> Result<String, String>
where
    F: FnOnce(&reqwest::blocking::Client, &str) -> reqwest::blocking::RequestBuilder,
{
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = build_client()?;
    let url = address.url(path);
    let response = build(&client, &url)
        .send()
        .map_err(|e| format!("could not reach daemon at {}: {e}", address.base_url()))?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().unwrap_or_default();
        if detail.is_empty() {
            return Err(format!("daemon returned {status} for {path}"));
        }
        return Err(format!("daemon returned {status} for {path}: {detail}"));
    }
    response
        .text()
        .map_err(|e| format!("unreadable response from {}: {e}", address.base_url()))
}

fn print_body(cli: &Cli, body: &str) -> CliExitCode {
    if cli.output.json {
        println!("{body}");
    } else if !cli.output.quiet {
        let value: serde_json::Value = serde_json::from_str(body)
            .unwrap_or_else(|_| serde_json::Value::String(body.to_owned()));
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| body.to_owned())
        );
    }
    CliExitCode::Success
}

#[derive(Debug, Deserialize)]
struct UserListResponse {
    users: Vec<UserRow>,
}

#[derive(Debug, Deserialize)]
struct UserRow {
    id: String,
    name: String,
    role: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Debug, Deserialize)]
struct DeviceListResponse {
    devices: Vec<DeviceRow>,
}

#[derive(Debug, Deserialize)]
struct DeviceRow {
    device_token: String,
    user_id: String,
    device_name: String,
    expires_at: i64,
}

#[derive(Debug, Deserialize)]
struct AdminFilesResponse {
    files: Vec<AdminFileRow>,
}

#[derive(Debug, Deserialize)]
struct AdminFileRow {
    id: String,
    name: String,
    size_bytes: u64,
    mime_type: String,
    #[serde(default)]
    folder_path: String,
    #[serde(default)]
    pinned: bool,
}

fn encode_query_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}
