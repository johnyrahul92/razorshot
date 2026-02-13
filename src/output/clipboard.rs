use cairo::ImageSurface;
use std::process::Command;

/// Copy an ImageSurface to the system clipboard.
/// Tries arboard first, falls back to wl-copy.
pub fn copy_to_clipboard(surface: &ImageSurface) -> Result<(), Box<dyn std::error::Error>> {
    // Get PNG bytes first (doesn't require mutable access)
    let mut png_data = Vec::new();
    surface.write_to_png(&mut png_data)?;

    match copy_via_arboard(surface) {
        Ok(()) => {
            log::info!("Image copied to clipboard via arboard");
            Ok(())
        }
        Err(e) => {
            log::warn!("arboard failed ({}), trying wl-copy fallback", e);
            copy_via_wl_copy_png(&png_data)
        }
    }
}

fn copy_via_arboard(surface: &ImageSurface) -> Result<(), Box<dyn std::error::Error>> {
    use arboard::{Clipboard, ImageData};

    let width = surface.width() as usize;
    let height = surface.height() as usize;
    let stride = surface.stride() as usize;

    // We need to clone the surface to get mutable access for .data()
    let mut cloned = clone_surface(surface)?;
    let data = cloned.data()?;

    // Convert ARGB (Cairo's native format, little-endian BGRA) to RGBA for arboard
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let offset = y * stride + x * 4;
            let b = data[offset];
            let g = data[offset + 1];
            let r = data[offset + 2];
            let a = data[offset + 3];
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(a);
        }
    }

    let mut clipboard = Clipboard::new()?;
    let img = ImageData {
        width,
        height,
        bytes: std::borrow::Cow::Owned(rgba),
    };
    clipboard.set_image(img)?;
    Ok(())
}

fn clone_surface(src: &ImageSurface) -> Result<ImageSurface, Box<dyn std::error::Error>> {
    let w = src.width();
    let h = src.height();
    let dest = ImageSurface::create(cairo::Format::ARgb32, w, h)?;
    let cr = cairo::Context::new(&dest)?;
    cr.set_source_surface(src, 0.0, 0.0)?;
    cr.paint()?;
    drop(cr);
    dest.flush();
    Ok(dest)
}

fn copy_via_wl_copy_png(png_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin.write_all(png_data)?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err("wl-copy failed".into());
    }

    log::info!("Image copied to clipboard via wl-copy");
    Ok(())
}
