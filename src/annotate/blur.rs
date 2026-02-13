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
    let img_width = surface.width();
    let img_height = surface.height();
    let stride = surface.stride() as usize;
    let block = block_size.max(1) as i32;

    // Clamp region to image bounds
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + width).min(img_width);
    let y1 = (y + height).min(img_height);

    let mut data = surface.data().expect("Failed to get surface data");

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
}
