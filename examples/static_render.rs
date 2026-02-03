/// Example: Render a single static screenshot
/// Usage: cargo run --example static_render https://example.com

use anyhow::Result;
use toast_browser::{launch_browser, capture_screenshot};
use toast_render::RenderPipeline;
use toast_terminal::{Renderer, Terminal};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(1).expect("Usage: cargo run --example static_render <url>");

    println!("Launching browser...");
    let browser = launch_browser().await?;

    println!("Capturing screenshot of {}...", url);
    let screenshot = capture_screenshot(&browser, url).await?;

    println!("Initializing terminal...");
    let terminal = Terminal::new()?;
    let (width, height) = terminal.size()?;

    println!("Rendering...");
    let pipeline = RenderPipeline::new();
    let frame = pipeline.render(&screenshot, width, height)?;

    let renderer = Renderer::new();
    renderer.render(frame)?;

    println!("\nPress Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
