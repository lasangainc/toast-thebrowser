use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::path::PathBuf;

/// Launch a headless Chrome browser
/// Automatically uses Helium if available, or falls back to CHROME_PATH environment variable
pub async fn launch_browser() -> Result<Browser> {
    let mut config_builder = BrowserConfig::builder().window_size(1920, 1080);

    // Try Helium first
    let helium_path = PathBuf::from("/Applications/Helium.app/Contents/MacOS/Helium");
    if helium_path.exists() {
        tracing::info!("Using Helium browser: {}", helium_path.display());
        config_builder = config_builder.chrome_executable(helium_path);
    }
    // Check for custom Chrome path via environment variable
    else if let Ok(chrome_path) = std::env::var("CHROME_PATH") {
        tracing::info!("Using custom Chrome executable: {}", chrome_path);
        config_builder = config_builder.chrome_executable(PathBuf::from(chrome_path));
    }

    let (browser, mut handler) = Browser::launch(
        config_builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?,
    )
    .await
    .context("Failed to launch browser")?;

    // Spawn handler to process browser events
    tokio::spawn(async move {
        while let Some(_) = handler.next().await {
            // Process events
        }
    });

    Ok(browser)
}
