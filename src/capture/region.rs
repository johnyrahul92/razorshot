use cairo::ImageSurface;

/// Crop an ImageSurface to the given rectangle.
/// Returns a new ImageSurface with just the selected region.
pub fn crop_surface(
    source: &ImageSurface,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<ImageSurface, Box<dyn std::error::Error>> {
    let cropped = ImageSurface::create(cairo::Format::ARgb32, width, height)?;
    let cr = cairo::Context::new(&cropped)?;
    cr.set_source_surface(source, -x as f64, -y as f64)?;
    cr.paint()?;
    drop(cr);
    cropped.flush();
    Ok(cropped)
}

/// Crop an ImageSurface for a specific monitor region.
#[allow(dead_code)]
pub fn crop_for_monitor(
    source: &ImageSurface,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
) -> Result<ImageSurface, Box<dyn std::error::Error>> {
    crop_surface(source, monitor_x, monitor_y, monitor_width, monitor_height)
}
