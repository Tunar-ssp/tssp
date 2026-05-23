//! `tssp paste` command implementation.

use std::io::Write;
use tempfile::NamedTempFile;
use image::{ImageBuffer, RgbaImage};

use tssp::{Cli, PasteArgs, UploadArgs};
use tssp_cli_core::CliExitCode;

pub fn run(cli: &Cli, args: &PasteArgs) -> Result<CliExitCode, String> {
    eprintln!(
        "Reading clipboard contents with {} tag(s)...",
        args.tags.len()
    );

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("failed to access clipboard: {e}"))?;

    // Try image first
    if let Ok(image_data) = clipboard.get_image() {
        eprintln!("Got image from clipboard: {}x{}", image_data.width, image_data.height);
        
        let mut temp_file = NamedTempFile::new().map_err(|e| format!("could not create temp file: {e}"))?;
        let filename = args.filename.clone().unwrap_or_else(|| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("clipboard-{timestamp}.png")
        });

        let width = u32::try_from(image_data.width).map_err(|_| "image width too large")?;
        let height = u32::try_from(image_data.height).map_err(|_| "image height too large")?;

        let rgba_image: RgbaImage = ImageBuffer::from_raw(
            width,
            height,
            image_data.bytes.into_owned(),
        ).ok_or("Failed to parse image data")?;
        
        let mut cursor = std::io::Cursor::new(Vec::new());
        rgba_image.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("failed to encode image: {e}"))?;
        temp_file.write_all(&cursor.into_inner())
            .map_err(|e| format!("failed to write image to temp file: {e}"))?;
        
        let path = temp_file.path().to_path_buf();
        return run_upload(cli, args, path, Some(filename), temp_file);
    }

    // Try text second
    if let Ok(text) = clipboard.get_text() {
        let mut paths = Vec::new();
        let lines: Vec<&str> = text.lines().map(str::trim).filter(|s| !s.is_empty()).collect();
        
        let mut is_file_list = !lines.is_empty();
        for line in &lines {
            let mut p_str = *line;
            if p_str.starts_with("file://") {
                p_str = p_str.trim_start_matches("file://");
            }
            
            let mut p = std::path::PathBuf::from(p_str);
            if !p.exists() {
                // Try simple URL decoding for spaces
                let decoded = p_str.replace("%20", " ");
                p = std::path::PathBuf::from(decoded);
            }
            
            if p.exists() && p.is_file() {
                paths.push(p);
            } else {
                is_file_list = false;
                break;
            }
        }

        if is_file_list && !paths.is_empty() {
            eprintln!("Clipboard contains file paths. Uploading...");
            let mut upload_cli = cli.clone();
            upload_cli.upload = UploadArgs {
                tags: args.tags.clone(),
                pin: false,
                rename: args.filename.clone(),
                parallel: None,
                recursive: None,
                all: false,
                files: paths,
            };
            return crate::upload::run(&upload_cli);
        }

        eprintln!("Got text from clipboard: {} bytes", text.len());

        let filename = args.filename.clone().unwrap_or_else(|| "clipboard.txt".to_string());
        let mut temp_file = NamedTempFile::new().map_err(|e| format!("could not create temp file: {e}"))?;
        temp_file.write_all(text.as_bytes()).map_err(|e| format!("could not write to temp file: {e}"))?;

        let path = temp_file.path().to_path_buf();
        return run_upload(cli, args, path, Some(filename), temp_file);
    }

    Ok(CliExitCode::Usage)
}

// We pass `_temp_file` to ensure it isn't dropped until the upload is fully complete.
fn run_upload(cli: &Cli, args: &PasteArgs, path: std::path::PathBuf, rename: Option<String>, _temp_file: NamedTempFile) -> Result<CliExitCode, String> {
    let mut upload_cli = cli.clone();
    upload_cli.upload = UploadArgs {
        tags: args.tags.clone(),
        pin: false,
        rename,
        parallel: None,
        recursive: None,
        all: false,
        files: vec![path],
    };
    
    crate::upload::run(&upload_cli)
}
