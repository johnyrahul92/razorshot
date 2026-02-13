use cairo::ImageSurface;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

/// Save an ImageSurface as PNG to the configured save directory.
/// Returns the path of the saved file.
pub fn save_screenshot(
    surface: &ImageSurface,
    config: &Config,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let save_dir = config.resolve_save_dir();
    fs::create_dir_all(&save_dir)?;

    let filename = Local::now().format(&config.filename_template).to_string();
    let path = save_dir.join(&filename);

    let mut file = fs::File::create(&path)?;
    surface.write_to_png(&mut file)?;

    log::info!("Screenshot saved to: {}", path.display());
    Ok(path)
}
