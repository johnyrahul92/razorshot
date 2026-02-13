use ashpd::desktop::screenshot::Screenshot;
use cairo::ImageSurface;
use std::fs::File;
use std::sync::OnceLock;

/// Shared tokio runtime — reused across captures to avoid D-Bus connection conflicts.
fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
    })
}

/// Capture a screenshot via xdg-desktop-portal.
/// Runs the async portal call on the shared tokio runtime from a background thread,
/// then returns the file path as a String.
pub fn capture_screenshot_path(interactive: bool) -> Result<String, String> {
    log::debug!("Requesting screenshot from portal (interactive={})", interactive);

    let rt = runtime();

    let path = rt.block_on(async {
        let response = Screenshot::request()
            .interactive(interactive)
            .modal(false)
            .send()
            .await
            .map_err(|e| format!("Portal request failed: {}", e))?
            .response()
            .map_err(|e| format!("Portal response failed: {}", e))?;

        let uri = response.uri().to_string();
        log::info!("Portal returned screenshot URI: {}", uri);

        let path = if let Some(stripped) = uri.strip_prefix("file://") {
            stripped.to_string()
        } else {
            uri
        };

        Ok::<String, String>(percent_decode(&path))
    })?;

    Ok(path)
}

/// Load a PNG file into a cairo::ImageSurface
pub fn load_image_surface(path: &str) -> Result<ImageSurface, Box<dyn std::error::Error>> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open screenshot file '{}': {}", path, e))?;
    let surface = ImageSurface::create_from_png(&mut file)
        .map_err(|e| format!("Failed to decode PNG from '{}': {}", path, e))?;
    log::debug!("Loaded image: {}x{}", surface.width(), surface.height());
    Ok(surface)
}

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().and_then(|c| char::from(c).to_digit(16));
            let lo = chars.next().and_then(|c| char::from(c).to_digit(16));
            if let (Some(h), Some(l)) = (hi, lo) {
                result.push(char::from((h * 16 + l) as u8));
            }
        } else {
            result.push(char::from(b));
        }
    }
    result
}
