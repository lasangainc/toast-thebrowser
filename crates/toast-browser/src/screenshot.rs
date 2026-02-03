use anyhow::{Context, Result};
use bytes::Bytes;
use chromiumoxide::browser::Browser;
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use chromiumoxide::page::Page;
use toast_core::{ImageFormat, Screenshot};

/// Capture a screenshot from a browser page
pub async fn capture_screenshot(browser: &Browser, url: &str) -> Result<Screenshot> {
    // Create new page
    let page = browser
        .new_page(url)
        .await
        .context("Failed to create new page")?;

    // Wait for page to load
    page.wait_for_navigation()
        .await
        .context("Failed to wait for navigation")?;

    // Give page a moment to render
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Capture screenshot as JPEG (faster than PNG)
    let params = CaptureScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Jpeg)
        .quality(85) // Good quality/speed tradeoff
        .build();

    let screenshot_data = page
        .screenshot(params)
        .await
        .context("Failed to capture screenshot")?;

    Ok(Screenshot {
        data: Bytes::from(screenshot_data),
        format: ImageFormat::Jpeg,
    })
}

/// Screenshot stream at a target frame rate
pub struct ScreenshotStreamer {
    browser: Browser,
    url: String,
    page: Option<Page>,
}

impl ScreenshotStreamer {
    pub fn new(browser: Browser, url: String) -> Self {
        Self { browser, url, page: None }
    }

    /// Initialize the page (call this once before capturing)
    pub async fn initialize(&mut self) -> Result<()> {
        let page = self.browser
            .new_page(&self.url)
            .await
            .context("Failed to create new page")?;

        page.wait_for_navigation()
            .await
            .context("Failed to wait for navigation")?;

        // Wait for page to be fully loaded and interactive
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        tracing::info!("Page initialized and ready for interaction");

        self.page = Some(page);
        Ok(())
    }

    /// Capture a single screenshot
    pub async fn capture(&self) -> Result<Screenshot> {
        if let Some(page) = &self.page {
            let params = CaptureScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Jpeg)
                .quality(85)
                .build();

            let screenshot_data = page
                .screenshot(params)
                .await
                .context("Failed to capture screenshot")?;

            Ok(Screenshot {
                data: Bytes::from(screenshot_data),
                format: ImageFormat::Jpeg,
            })
        } else {
            Err(anyhow::anyhow!("Page not initialized. Call initialize() first."))
        }
    }

    /// Send a mouse click at the specified coordinates
    pub async fn click(&self, x: f64, y: f64) -> Result<()> {
        if let Some(page) = &self.page {
            tracing::info!("Starting click at ({}, {})", x, y);

            // Use CDP mouse events (simpler and more reliable)
            // First, move the mouse to the position
            let mouse_move = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseMoved)
                .x(x)
                .y(y)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build mouse move event: {}", e))?;

            tracing::info!("Executing mouse move");
            page.execute(mouse_move).await
                .context("Failed to execute mouse move")?;

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Mouse down
            let mouse_down = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MousePressed)
                .x(x)
                .y(y)
                .button(MouseButton::Left)
                .click_count(1)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build mouse down event: {}", e))?;

            tracing::info!("Executing mouse down");
            page.execute(mouse_down).await
                .context("Failed to execute mouse down")?;

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Mouse up
            let mouse_up = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseReleased)
                .x(x)
                .y(y)
                .button(MouseButton::Left)
                .click_count(1)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build mouse up event: {}", e))?;

            tracing::info!("Executing mouse up");
            page.execute(mouse_up).await
                .context("Failed to execute mouse up")?;

            tracing::info!("Click completed at ({}, {})", x, y);

            Ok(())
        } else {
            Err(anyhow::anyhow!("Page not initialized. Call initialize() first."))
        }
    }

    /// Scroll the page by a given pixel amount
    pub async fn scroll(&self, delta_y: i32) -> Result<()> {
        if let Some(page) = &self.page {
            // Use JavaScript to scroll - this is the most reliable method
            let script = format!("window.scrollBy(0, {})", delta_y);
            page.evaluate_expression(script)
                .await
                .context("Failed to execute scroll command")?;

            tracing::info!("Scrolled by {} pixels", delta_y);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Page not initialized. Call initialize() first."))
        }
    }
}
