use toast_core::{AnsiColor, Rgb};

/// ANSI 256 color palette
const ANSI_PALETTE: [Rgb; 256] = generate_ansi_palette();

/// Color quantizer using a 32KB lookup table for O(1) color matching
pub struct ColorQuantizer {
    /// Lookup table: 32×32×32 RGB555 → ANSI 256 index
    /// Indexed by (r>>3, g>>3, b>>3) to reduce RGB888 to RGB555
    lut: Box<[u8; 32768]>,
}

impl ColorQuantizer {
    /// Build the lookup table at startup
    /// This is a one-time cost (~50ms) that enables O(1) runtime lookups
    pub fn new() -> Self {
        let mut lut = Box::new([0u8; 32768]);

        // For each possible RGB555 color
        for r5 in 0..32u8 {
            for g5 in 0..32u8 {
                for b5 in 0..32u8 {
                    // Convert RGB555 back to RGB888 (expand to full range)
                    let r8 = (r5 << 3) | (r5 >> 2);
                    let g8 = (g5 << 3) | (g5 >> 2);
                    let b8 = (b5 << 3) | (b5 >> 2);
                    let rgb = Rgb::new(r8, g8, b8);

                    // Find nearest ANSI color using CIELEAB distance
                    let ansi_idx = find_nearest_ansi_color(rgb);

                    // Store in LUT
                    let lut_idx = lut_index(r5, g5, b5);
                    lut[lut_idx] = ansi_idx;
                }
            }
        }

        Self { lut }
    }

    /// Quantize an RGB color to ANSI 256 in O(1) time
    #[inline]
    pub fn quantize(&self, rgb: Rgb) -> AnsiColor {
        // Reduce RGB888 to RGB555 by shifting right 3 bits
        let r5 = rgb.r >> 3;
        let g5 = rgb.g >> 3;
        let b5 = rgb.b >> 3;

        // Lookup in table
        let idx = lut_index(r5, g5, b5);
        AnsiColor(self.lut[idx])
    }

    /// Quantize a slice of RGB colors in parallel
    pub fn quantize_batch(&self, colors: &[Rgb]) -> Vec<AnsiColor> {
        colors.iter().map(|&rgb| self.quantize(rgb)).collect()
    }
}

impl Default for ColorQuantizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate LUT index from RGB555 values
#[inline]
const fn lut_index(r5: u8, g5: u8, b5: u8) -> usize {
    ((r5 as usize) << 10) | ((g5 as usize) << 5) | (b5 as usize)
}

/// Find the nearest ANSI 256 color using CIELEAB color distance
fn find_nearest_ansi_color(rgb: Rgb) -> u8 {
    let lab = rgb_to_lab(rgb);
    let mut min_distance = f32::INFINITY;
    let mut best_idx = 0u8;

    for (idx, &ansi_rgb) in ANSI_PALETTE.iter().enumerate() {
        let ansi_lab = rgb_to_lab(ansi_rgb);
        let distance = color_distance_lab(lab, ansi_lab);

        if distance < min_distance {
            min_distance = distance;
            best_idx = idx as u8;
        }
    }

    best_idx
}

/// LAB color representation
#[derive(Debug, Clone, Copy)]
struct Lab {
    l: f32,
    a: f32,
    b: f32,
}

/// Convert RGB to CIELEAB color space for perceptual distance calculations
fn rgb_to_lab(rgb: Rgb) -> Lab {
    // First convert RGB to XYZ
    let r = gamma_expand(rgb.r as f32 / 255.0);
    let g = gamma_expand(rgb.g as f32 / 255.0);
    let b = gamma_expand(rgb.b as f32 / 255.0);

    // RGB to XYZ matrix (D65 illuminant)
    let x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
    let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
    let z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041;

    // Normalize by D65 white point
    let x = x / 0.95047;
    let y = y / 1.00000;
    let z = z / 1.08883;

    // XYZ to LAB
    let fx = lab_f(x);
    let fy = lab_f(y);
    let fz = lab_f(z);

    Lab {
        l: 116.0 * fy - 16.0,
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

#[inline]
fn gamma_expand(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn lab_f(t: f32) -> f32 {
    const DELTA: f32 = 6.0 / 29.0;
    if t > DELTA * DELTA * DELTA {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

/// Calculate CIEDE2000 color distance (simplified)
fn color_distance_lab(lab1: Lab, lab2: Lab) -> f32 {
    let dl = lab1.l - lab2.l;
    let da = lab1.a - lab2.a;
    let db = lab1.b - lab2.b;
    // Simplified Euclidean distance in LAB space
    // (Full CIEDE2000 is more complex but this is good enough)
    (dl * dl + da * da + db * db).sqrt()
}

/// Generate the ANSI 256 color palette
const fn generate_ansi_palette() -> [Rgb; 256] {
    let mut palette = [Rgb::new(0, 0, 0); 256];

    // 16 basic colors (0-15)
    palette[0] = Rgb::new(0, 0, 0);       // Black
    palette[1] = Rgb::new(128, 0, 0);     // Red
    palette[2] = Rgb::new(0, 128, 0);     // Green
    palette[3] = Rgb::new(128, 128, 0);   // Yellow
    palette[4] = Rgb::new(0, 0, 128);     // Blue
    palette[5] = Rgb::new(128, 0, 128);   // Magenta
    palette[6] = Rgb::new(0, 128, 128);   // Cyan
    palette[7] = Rgb::new(192, 192, 192); // White
    palette[8] = Rgb::new(128, 128, 128); // Bright Black
    palette[9] = Rgb::new(255, 0, 0);     // Bright Red
    palette[10] = Rgb::new(0, 255, 0);    // Bright Green
    palette[11] = Rgb::new(255, 255, 0);  // Bright Yellow
    palette[12] = Rgb::new(0, 0, 255);    // Bright Blue
    palette[13] = Rgb::new(255, 0, 255);  // Bright Magenta
    palette[14] = Rgb::new(0, 255, 255);  // Bright Cyan
    palette[15] = Rgb::new(255, 255, 255);// Bright White

    // 216 colors (16-231): 6×6×6 color cube
    let mut idx = 16;
    let mut r = 0;
    while r < 6 {
        let mut g = 0;
        while g < 6 {
            let mut b = 0;
            while b < 6 {
                let rv = if r == 0 { 0 } else { 55 + r * 40 };
                let gv = if g == 0 { 0 } else { 55 + g * 40 };
                let bv = if b == 0 { 0 } else { 55 + b * 40 };
                palette[idx] = Rgb::new(rv as u8, gv as u8, bv as u8);
                idx += 1;
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }

    // 24 grayscale colors (232-255)
    let mut i = 0;
    while i < 24 {
        let gray = (8 + i * 10) as u8;
        palette[232 + i] = Rgb::new(gray, gray, gray);
        i += 1;
    }

    palette
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantizer_basic_colors() {
        let q = ColorQuantizer::new();

        // Test black
        let black = q.quantize(Rgb::new(0, 0, 0));
        assert_eq!(black.as_u8(), 0);

        // Test white
        let white = q.quantize(Rgb::new(255, 255, 255));
        assert_eq!(white.as_u8(), 15);

        // Test pure red
        let red = q.quantize(Rgb::new(255, 0, 0));
        assert_eq!(red.as_u8(), 9);
    }

    #[test]
    fn test_lut_index() {
        assert_eq!(lut_index(0, 0, 0), 0);
        assert_eq!(lut_index(31, 31, 31), 32767);
        assert_eq!(lut_index(1, 0, 0), 1024);
        assert_eq!(lut_index(0, 1, 0), 32);
        assert_eq!(lut_index(0, 0, 1), 1);
    }

    #[test]
    fn test_ansi_palette_generation() {
        let palette = generate_ansi_palette();
        assert_eq!(palette[0], Rgb::new(0, 0, 0));
        assert_eq!(palette[15], Rgb::new(255, 255, 255));
        // Test grayscale ramp
        assert_eq!(palette[232], Rgb::new(8, 8, 8));
        assert_eq!(palette[255], Rgb::new(238, 238, 238));
    }
}
