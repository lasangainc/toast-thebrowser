use anyhow::Result;
use fast_image_resize as fr;
use std::num::NonZeroU32;
use toast_core::RgbImage;

/// Scale an RGB image to target dimensions using high-quality resampling
pub fn scale_image(image: &RgbImage, target_width: u32, target_height: u32) -> Result<RgbImage> {
    // If already correct size, return clone
    if image.width == target_width && image.height == target_height {
        return Ok(image.clone());
    }

    // Create source image view
    let src_image = fr::Image::from_vec_u8(
        NonZeroU32::new(image.width).unwrap(),
        NonZeroU32::new(image.height).unwrap(),
        image.data.clone(),
        fr::PixelType::U8x3,
    )?;

    // Create destination image
    let mut dst_image = fr::Image::new(
        NonZeroU32::new(target_width).unwrap(),
        NonZeroU32::new(target_height).unwrap(),
        fr::PixelType::U8x3,
    );

    // Create resizer with Lanczos3 algorithm (good quality/speed tradeoff)
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

    // Perform resize
    resizer.resize(&src_image.view(), &mut dst_image.view_mut())?;

    Ok(RgbImage::new(
        dst_image.into_vec(),
        target_width,
        target_height,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use toast_core::Rgb;

    #[test]
    fn test_scale_image_upscale() {
        // Create a 2x2 white image
        let data = vec![255u8; 2 * 2 * 3];
        let image = RgbImage::new(data, 2, 2);

        // Scale up to 4x4
        let scaled = scale_image(&image, 4, 4).unwrap();

        assert_eq!(scaled.width, 4);
        assert_eq!(scaled.height, 4);
        assert_eq!(scaled.data.len(), 4 * 4 * 3);
    }

    #[test]
    fn test_scale_image_downscale() {
        // Create a 4x4 white image
        let data = vec![255u8; 4 * 4 * 3];
        let image = RgbImage::new(data, 4, 4);

        // Scale down to 2x2
        let scaled = scale_image(&image, 2, 2).unwrap();

        assert_eq!(scaled.width, 2);
        assert_eq!(scaled.height, 2);
        assert_eq!(scaled.data.len(), 2 * 2 * 3);
    }

    #[test]
    fn test_scale_image_no_change() {
        // Create a 2x2 white image
        let data = vec![255u8; 2 * 2 * 3];
        let image = RgbImage::new(data, 2, 2);

        // "Scale" to same size
        let scaled = scale_image(&image, 2, 2).unwrap();

        assert_eq!(scaled.width, 2);
        assert_eq!(scaled.height, 2);
    }
}
