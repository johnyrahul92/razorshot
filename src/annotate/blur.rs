use cairo::ImageSurface;

/// Apply pixelation blur to a rectangular region of an ImageSurface.
/// This modifies the surface in-place by averaging NxN blocks of pixels.
pub fn pixelate_region(
    surface: &mut ImageSurface,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    block_size: u32,
) {
    surface.flush();

    let img_width = surface.width();
    let img_height = surface.height();
    let stride = surface.stride() as usize;
    let block = block_size.max(2) as i32;

    // Clamp region to image bounds
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + width).min(img_width);
    let y1 = (y + height).min(img_height);

    if x0 >= x1 || y0 >= y1 {
        return;
    }

    let mut data = match surface.data() {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to get surface data for blur: {}", e);
            return;
        }
    };

    let mut by = y0;
    while by < y1 {
        let mut bx = x0;
        while bx < x1 {
            let bw = block.min(x1 - bx);
            let bh = block.min(y1 - by);

            // Compute average color for this block
            let mut sum_r: u64 = 0;
            let mut sum_g: u64 = 0;
            let mut sum_b: u64 = 0;
            let mut sum_a: u64 = 0;
            let count = (bw * bh) as u64;

            if count == 0 {
                bx += block;
                continue;
            }

            for py in by..by + bh {
                for px in bx..bx + bw {
                    let offset = py as usize * stride + px as usize * 4;
                    sum_b += data[offset] as u64;
                    sum_g += data[offset + 1] as u64;
                    sum_r += data[offset + 2] as u64;
                    sum_a += data[offset + 3] as u64;
                }
            }

            let avg_b = (sum_b / count) as u8;
            let avg_g = (sum_g / count) as u8;
            let avg_r = (sum_r / count) as u8;
            let avg_a = (sum_a / count) as u8;

            // Fill block with average color
            for py in by..by + bh {
                for px in bx..bx + bw {
                    let offset = py as usize * stride + px as usize * 4;
                    data[offset] = avg_b;
                    data[offset + 1] = avg_g;
                    data[offset + 2] = avg_r;
                    data[offset + 3] = avg_a;
                }
            }

            bx += block;
        }
        by += block;
    }

    drop(data);
    surface.mark_dirty();
}

/// Create a pixelated copy of a region from a source surface.
/// Returns a new ImageSurface with the pixelated content.
pub fn pixelate_region_copy(
    source: &ImageSurface,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    block_size: u32,
) -> Option<ImageSurface> {
    if width <= 0 || height <= 0 {
        return None;
    }

    // Create a copy of just this region
    let region = ImageSurface::create(cairo::Format::ARgb32, width, height).ok()?;
    let cr = cairo::Context::new(&region).ok()?;
    cr.set_source_surface(source, -x as f64, -y as f64).ok()?;
    cr.paint().ok()?;
    drop(cr);

    let mut region = region;
    pixelate_region(&mut region, 0, 0, width, height, block_size);
    Some(region)
}
