use ashpd::desktop::screenshot::Screenshot;
use cairo::ImageSurface;
use std::fs::File;

/// Capture a screenshot via xdg-desktop-portal.
/// Runs the async portal call inside a tokio runtime on a background thread,
/// then returns the path as a String so the caller can load the image on the main thread.
pub fn capture_screenshot_path(interactive: bool) -> Result<String, String> {
    log::debug!("Requesting screenshot from portal (interactive={})", interactive);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

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

/// Capture screenshot and return an ImageSurface.
/// Spawns a background thread for the portal D-Bus call, sends result back via channel.
#[allow(dead_code)]
pub fn capture_screenshot_threaded(
    interactive: bool,
    callback: Box<dyn FnOnce(Result<ImageSurface, String>) + Send + 'static>,
) {
    std::thread::spawn(move || {
        // Small delay to let windows hide
        std::thread::sleep(std::time::Duration::from_millis(200));

        let result = capture_screenshot_path(interactive).and_then(|path| {
            log::debug!("Loading screenshot from: {}", path);
            load_image_surface(&path)
                .map_err(|e| e.to_string())
        });

        callback(result);
    });
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
