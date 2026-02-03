/// Example: Test color quantization and rendering
/// Usage: cargo run --example test_colors

use toast_core::{AnsiColor, Rgb, TerminalCell, TerminalFrame};
use toast_render::ColorQuantizer;
use toast_terminal::{Renderer, Terminal};

fn main() -> anyhow::Result<()> {
    let terminal = Terminal::new()?;
    let (width, height) = terminal.size()?;

    // Create a test frame with color gradient
    let mut frame = TerminalFrame::new(width, height);
    let quantizer = ColorQuantizer::new();

    for y in 0..height {
        for x in 0..width {
            // Create RGB gradient
            let r = (255.0 * x as f32 / width as f32) as u8;
            let g = (255.0 * y as f32 / height as f32) as u8;
            let b = 128;

            let rgb = Rgb::new(r, g, b);
            let ansi = quantizer.quantize(rgb);

            frame.set(
                x,
                y,
                TerminalCell {
                    character: '█',
                    foreground: ansi,
                    background: ansi,
                },
            );
        }
    }

    let renderer = Renderer::new();
    renderer.render(frame)?;

    println!("\nColor gradient test - Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
