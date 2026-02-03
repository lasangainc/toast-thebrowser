use rayon::prelude::*;
use toast_core::{RgbImage, TerminalCell, TerminalFrame};

use crate::quantizer::ColorQuantizer;

/// Unicode upper half block character
const UPPER_HALF_BLOCK: char = '▀';

/// Full block character (used when top and bottom are same color)
const FULL_BLOCK: char = '█';

/// Convert an RGB image to a terminal frame using half-block characters
/// Each terminal cell represents 2 vertical pixels using the upper half block character
pub struct HalfBlockConverter {
    quantizer: ColorQuantizer,
}

impl HalfBlockConverter {
    pub fn new() -> Self {
        Self {
            quantizer: ColorQuantizer::new(),
        }
    }

    /// Convert an RGB image to a terminal frame
    /// The image height should be 2x the terminal height
    pub fn convert(&self, image: &RgbImage, term_width: usize, term_height: usize) -> TerminalFrame {
        let mut frame = TerminalFrame::new(term_width, term_height);

        // Process rows in parallel using rayon
        let rows: Vec<Vec<TerminalCell>> = (0..term_height)
            .into_par_iter()
            .map(|y| {
                let mut row = Vec::with_capacity(term_width);
                for x in 0..term_width {
                    let cell = self.convert_cell(image, x as u32, y as u32);
                    row.push(cell);
                }
                row
            })
            .collect();

        // Copy rows into frame
        for (y, row) in rows.into_iter().enumerate() {
            for (x, cell) in row.into_iter().enumerate() {
                frame.set(x, y, cell);
            }
        }

        frame
    }

    /// Convert a single terminal cell (2 vertical pixels)
    fn convert_cell(&self, image: &RgbImage, cell_x: u32, cell_y: u32) -> TerminalCell {
        // Each cell represents 2 vertical pixels
        let pixel_y_top = cell_y * 2;
        let pixel_y_bottom = pixel_y_top + 1;

        // Handle edge case where bottom pixel is out of bounds
        let (top_rgb, bottom_rgb) = if pixel_y_bottom < image.height {
            (
                image.get_pixel(cell_x, pixel_y_top),
                image.get_pixel(cell_x, pixel_y_bottom),
            )
        } else {
            // Only top pixel exists
            let top = image.get_pixel(cell_x, pixel_y_top);
            (top, top)
        };

        // Quantize to ANSI colors
        let top_ansi = self.quantizer.quantize(top_rgb);
        let bottom_ansi = self.quantizer.quantize(bottom_rgb);

        // Choose character and colors
        if top_ansi == bottom_ansi {
            // Same color - use full block or space
            TerminalCell {
                character: FULL_BLOCK,
                foreground: top_ansi,
                background: top_ansi,
            }
        } else {
            // Different colors - use upper half block
            // Foreground = top pixel, Background = bottom pixel
            TerminalCell {
                character: UPPER_HALF_BLOCK,
                foreground: top_ansi,
                background: bottom_ansi,
            }
        }
    }
}

impl Default for HalfBlockConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toast_core::Rgb;

    #[test]
    fn test_convert_single_cell_same_color() {
        let converter = HalfBlockConverter::new();

        // Create a 1x2 image (1 pixel wide, 2 pixels tall) - pure white
        let data = vec![255, 255, 255, 255, 255, 255];
        let image = RgbImage::new(data, 1, 2);

        let cell = converter.convert_cell(&image, 0, 0);

        // Should use full block with same fg/bg
        assert_eq!(cell.character, FULL_BLOCK);
        assert_eq!(cell.foreground, cell.background);
    }

    #[test]
    fn test_convert_single_cell_different_colors() {
        let converter = HalfBlockConverter::new();

        // Create a 1x2 image: top=white, bottom=black
        let data = vec![255, 255, 255, 0, 0, 0];
        let image = RgbImage::new(data, 1, 2);

        let cell = converter.convert_cell(&image, 0, 0);

        // Should use upper half block
        assert_eq!(cell.character, UPPER_HALF_BLOCK);
        // Foreground should be white (top), background should be black (bottom)
        assert_eq!(cell.foreground.as_u8(), 15); // White
        assert_eq!(cell.background.as_u8(), 0);  // Black
    }

    #[test]
    fn test_convert_full_frame() {
        let converter = HalfBlockConverter::new();

        // Create a 4x4 image (will be 4x2 terminal cells)
        let mut data = vec![0u8; 4 * 4 * 3];
        // Fill with white
        for pixel in data.chunks_mut(3) {
            pixel[0] = 255;
            pixel[1] = 255;
            pixel[2] = 255;
        }
        let image = RgbImage::new(data, 4, 4);

        let frame = converter.convert(&image, 4, 2);

        assert_eq!(frame.width, 4);
        assert_eq!(frame.height, 2);
        assert_eq!(frame.cells.len(), 8);

        // All cells should be white
        for cell in &frame.cells {
            assert_eq!(cell.foreground.as_u8(), 15);
            assert_eq!(cell.background.as_u8(), 15);
        }
    }
}
