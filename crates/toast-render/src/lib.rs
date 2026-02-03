mod decoder;
mod halfblock;
mod quantizer;
mod scaler;

pub use decoder::decode_screenshot;
pub use halfblock::HalfBlockConverter;
pub use quantizer::ColorQuantizer;
pub use scaler::scale_image;

use anyhow::Result;
use toast_core::{Screenshot, TerminalFrame};

/// Full rendering pipeline: Screenshot → Terminal Frame
pub struct RenderPipeline {
    converter: HalfBlockConverter,
}

impl RenderPipeline {
    pub fn new() -> Self {
        Self {
            converter: HalfBlockConverter::new(),
        }
    }

    /// Convert a screenshot to a terminal frame
    pub fn render(
        &self,
        screenshot: &Screenshot,
        term_width: usize,
        term_height: usize,
    ) -> Result<TerminalFrame> {
        // Decode screenshot to RGB
        let rgb_image = decode_screenshot(screenshot)?;

        // Calculate target dimensions (height * 2 because each terminal row = 2 pixels)
        let target_width = term_width as u32;
        let target_height = (term_height as u32) * 2;

        // Scale to terminal dimensions
        let scaled = scale_image(&rgb_image, target_width, target_height)?;

        // Convert to half-blocks
        let frame = self.converter.convert(&scaled, term_width, term_height);

        Ok(frame)
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RenderPipeline {
    fn clone(&self) -> Self {
        Self::new()
    }
}
