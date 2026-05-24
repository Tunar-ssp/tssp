//! Terminal QR rendering for share links.

use qrcode::render::unicode::Dense1x2;
use qrcode::QrCode;

/// Renders `data` as a Unicode block QR code for terminal or `<pre>` display.
///
/// # Errors
///
/// Returns a message when the payload cannot be encoded as a QR symbol.
pub fn terminal_qr(data: &str) -> Result<String, String> {
    let qr = QrCode::new(data.as_bytes()).map_err(|error| format!("qr encode failed: {error}"))?;
    Ok(qr.render::<Dense1x2>().build())
}

#[cfg(test)]
mod tests {
    use super::terminal_qr;

    #[test]
    #[allow(clippy::expect_used)]
    fn terminal_qr_encodes_url() {
        let qr = terminal_qr("https://example.com/p/abc").expect("qr");
        assert!(qr.contains('█') || qr.contains('▀') || qr.lines().count() > 3);
    }
}
