//! Helper functions for session management and display.

pub fn generate_qr_code(data: &str, _size: usize) -> Result<String, String> {
    use qrcode::render::unicode::Dense1x2;

    let qr = qrcode::QrCode::new(data).map_err(|e| format!("failed to generate QR code: {e}"))?;
    let img = qr.render::<Dense1x2>().build();
    let mut output = String::new();
    for line in img.lines() {
        output.push_str(line);
        output.push('\n');
    }
    Ok(output)
}
