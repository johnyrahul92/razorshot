use cairo::ImageSurface;
use chrono::Local;
use image::ImageEncoder;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

/// Save an ImageSurface to the configured save directory.
/// Supports PNG and JPEG formats based on config.export_format.
/// Returns the path of the saved file.
pub fn save_screenshot(
    surface: &ImageSurface,
    config: &Config,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let save_dir = config.resolve_save_dir();
    fs::create_dir_all(&save_dir)?;

    let ext = match config.export_format.to_lowercase().as_str() {
        "jpg" | "jpeg" => "jpg",
        _ => "png",
    };

    let basename = Local::now().format(&config.filename_template).to_string();
    let filename = format!("{basename}.{ext}");
    let path = save_dir.join(&filename);

    match ext {
        "jpg" => save_as_jpeg(surface, &path, config.annotation.jpeg_quality)?,
        _ => {
            let mut file = fs::File::create(&path)?;
            surface.write_to_png(&mut file)?;
        }
    }

    log::info!("Screenshot saved to: {}", path.display());
    Ok(path)
}

fn save_as_jpeg(
    surface: &ImageSurface,
    path: &PathBuf,
    quality: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = surface.width() as u32;
    let height = surface.height() as u32;
    let stride = surface.stride() as usize;

    // Clone the surface to get mutable access to pixel data
    let mut cloned = {
        let w = surface.width();
        let h = surface.height();
        let dest = ImageSurface::create(cairo::Format::ARgb32, w, h)?;
        let cr = cairo::Context::new(&dest)?;
        cr.set_source_surface(surface, 0.0, 0.0)?;
        cr.paint()?;
        drop(cr);
        dest.flush();
        dest
    };

    let data = cloned.data()?;

    // Convert Cairo BGRA to RGB (JPEG doesn't support alpha)
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height as usize {
        for x in 0..width as usize {
            let offset = y * stride + x * 4;
            let b = data[offset];
            let g = data[offset + 1];
            let r = data[offset + 2];
            rgb.push(r);
            rgb.push(g);
            rgb.push(b);
        }
    }

    let file = fs::File::create(path)?;
    let mut buf_writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf_writer, quality);
    encoder.write_image(&rgb, width, height, image::ExtendedColorType::Rgb8)?;

    Ok(())
}
