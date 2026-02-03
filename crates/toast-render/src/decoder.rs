use anyhow::{Context, Result};
use toast_core::{ImageFormat, RgbImage, Screenshot};

/// Decode a screenshot to RGB format
pub fn decode_screenshot(screenshot: &Screenshot) -> Result<RgbImage> {
    let img = match screenshot.format {
        ImageFormat::Jpeg => {
            image::load_from_memory_with_format(&screenshot.data, image::ImageFormat::Jpeg)
        }
        ImageFormat::Png => {
            image::load_from_memory_with_format(&screenshot.data, image::ImageFormat::Png)
        }
    }
    .context("Failed to decode image")?;

    // Convert to RGB8
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    Ok(RgbImage::new(rgb.into_raw(), width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_decode_simple_image() {
        // Create a simple 2x2 white PNG
        let mut img_buffer = image::RgbImage::new(2, 2);
        for pixel in img_buffer.pixels_mut() {
            *pixel = image::Rgb([255, 255, 255]);
        }

        let mut png_data = Vec::new();
        img_buffer
            .write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
            .unwrap();

        let screenshot = Screenshot {
            data: Bytes::from(png_data),
            format: ImageFormat::Png,
        };

        let decoded = decode_screenshot(&screenshot).unwrap();
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.data.len(), 2 * 2 * 3);
    }
}
