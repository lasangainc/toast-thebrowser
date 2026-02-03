use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use toast_browser::{launch_browser, ScreenshotStreamer};
use toast_core::{AnsiColor, CursorPosition, TerminalCell};
use toast_render::RenderPipeline;
use toast_terminal::{Renderer, Terminal};
use tracing::{error, info};

const TARGET_FPS: u32 = 15;
const FRAME_INTERVAL_MS: u64 = 1000 / TARGET_FPS as u64; // ~66ms

/// Main application orchestrator
pub struct App {
    url: String,
}

impl App {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Run the application
    pub async fn run(self) -> Result<()> {
        info!("Launching browser...");
        let browser = launch_browser().await?;

        info!("Initializing terminal...");
        let terminal = Terminal::new()?;
        let (width, height) = terminal.size()?;
        info!("Terminal size: {}x{}", width, height);

        let renderer = Renderer::new();
        let pipeline = RenderPipeline::new();
        let mut streamer = ScreenshotStreamer::new(browser, self.url.clone());

        // Initialize the page
        streamer.initialize().await?;

        // Wrap streamer in Arc for sharing between tasks
        let streamer = Arc::new(streamer);

        // Cursor position (shared between keyboard and display tasks)
        let cursor_pos = Arc::new(Mutex::new(CursorPosition {
            x: width / 2,
            y: height / 2,
        }));

        // Channels for async pipeline
        let (screenshot_tx, mut screenshot_rx) = mpsc::channel(2);
        let (frame_tx, mut frame_rx) = mpsc::channel(1);

        // Screenshot capture task - runs at 15fps interval
        let screenshot_task = {
            let streamer = Arc::clone(&streamer);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(FRAME_INTERVAL_MS));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                loop {
                    interval.tick().await;

                    match streamer.capture().await {
                        Ok(screenshot) => {
                            // Use try_send for backpressure - drop frame if channel full
                            if screenshot_tx.try_send(screenshot).is_err() {
                                info!("Dropped screenshot frame (channel full)");
                            }
                        }
                        Err(e) => {
                            error!("Failed to capture screenshot: {}", e);
                        }
                    }
                }
            })
        };

        // Render task - CPU-intensive processing
        let render_task = {
            tokio::spawn(async move {
                while let Some(screenshot) = screenshot_rx.recv().await {
                    // Render in blocking thread pool
                    let pipeline_clone = pipeline.clone();
                    match tokio::task::spawn_blocking(move || {
                        pipeline_clone.render(&screenshot, width, height)
                    })
                    .await
                    {
                        Ok(Ok(frame)) => {
                            // Send to display task
                            if frame_tx.send(frame).await.is_err() {
                                error!("Display task disconnected");
                                break;
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Failed to render frame: {}", e);
                        }
                        Err(e) => {
                            error!("Render task panicked: {}", e);
                        }
                    }
                }
            })
        };

        // Display task - write to terminal with cursor overlay
        let display_task = {
            let cursor_pos: Arc<Mutex<CursorPosition>> = Arc::clone(&cursor_pos);
            tokio::spawn(async move {
                while let Some(mut frame) = frame_rx.recv().await {
                    // Overlay cursor on the frame - draw classic arrow pointer using half blocks
                    if let Ok(pos) = cursor_pos.lock() {
                        // Classic arrow cursor using half blocks (2 pixel rows per char row):
                        // █▄           (row 0: pixels 0-1)
                        // ███▄         (row 1: pixels 2-3)
                        // █████▄       (row 2: pixels 4-5)
                        // ▀ ██         (row 3: pixels 6-7)
                        //    ▀         (row 4: pixel 8)

                        let black = AnsiColor(16);

                        // Helper to set cursor cell with specific character (all black)
                        let mut set_cell = |x: usize, y: usize, ch: char| {
                            if x < frame.width && y < frame.height {
                                frame.set(x, y, TerminalCell {
                                    character: ch,
                                    foreground: black,
                                    background: black,
                                });
                            }
                        };

                        // Row 0: █▄ (pixels 0,1 at x=0; pixel 1 at x=1)
                        set_cell(pos.x, pos.y, '█');
                        set_cell(pos.x + 1, pos.y, '▄');

                        // Row 1: ███▄ (pixels 2,3 full at x=0,1,2; pixel 3 at x=3)
                        set_cell(pos.x, pos.y + 1, '█');
                        set_cell(pos.x + 1, pos.y + 1, '█');
                        set_cell(pos.x + 2, pos.y + 1, '█');
                        set_cell(pos.x + 3, pos.y + 1, '▄');

                        // Row 2: █████▄ (pixels 4,5 full at x=0-4; pixel 5 at x=5)
                        set_cell(pos.x, pos.y + 2, '█');
                        set_cell(pos.x + 1, pos.y + 2, '█');
                        set_cell(pos.x + 2, pos.y + 2, '█');
                        set_cell(pos.x + 3, pos.y + 2, '█');
                        set_cell(pos.x + 4, pos.y + 2, '█');
                        set_cell(pos.x + 5, pos.y + 2, '▄');

                        // Row 3: ▀ ██ (pixel 6 at x=0; pixels 6,7 at x=3,4)
                        set_cell(pos.x, pos.y + 3, '▀');
                        set_cell(pos.x + 3, pos.y + 3, '█');
                        set_cell(pos.x + 4, pos.y + 3, '█');

                        // Row 4:    ▀ (pixel 8 at x=4)
                        set_cell(pos.x + 4, pos.y + 4, '▀');
                    }

                    if let Err(e) = renderer.render(frame) {
                        error!("Failed to render to terminal: {}", e);
                    }
                }
            })
        };

        info!("Rendering started. Use arrow keys to move cursor, Enter to click, Ctrl+C to exit.");

        // Channel for shutdown signal and click events
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        let (click_tx, mut click_rx) = mpsc::channel(10);

        // Keyboard input task - handle arrow keys, Enter, and Ctrl+C
        let keyboard_task = {
            let shutdown_tx = shutdown_tx.clone();
            let cursor_pos: Arc<Mutex<CursorPosition>> = Arc::clone(&cursor_pos);
            tokio::spawn(async move {
                loop {
                    // Poll for events with timeout
                    if let Ok(true) = event::poll(Duration::from_millis(100)) {
                        if let Ok(Event::Key(key_event)) = event::read() {
                        // Only handle key press events, not release or repeat
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                    info!("Ctrl+C detected from keyboard");
                                    let _ = shutdown_tx.send(()).await;
                                    break;
                                }
                                KeyCode::Up => {
                                    if let Ok(mut pos) = cursor_pos.lock() {
                                        if pos.y > 0 {
                                            pos.y -= 1;
                                        }
                                    }
                                }
                                KeyCode::Down => {
                                    if let Ok(mut pos) = cursor_pos.lock() {
                                        if pos.y < height - 1 {
                                            pos.y += 1;
                                        }
                                    }
                                }
                                KeyCode::Left => {
                                    if let Ok(mut pos) = cursor_pos.lock() {
                                        if pos.x > 0 {
                                            pos.x -= 1;
                                        }
                                    }
                                }
                                KeyCode::Right => {
                                    if let Ok(mut pos) = cursor_pos.lock() {
                                        if pos.x < width - 1 {
                                            pos.x += 1;
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    // Copy values out of the mutex before await
                                    let coords = cursor_pos.lock().map(|pos| (pos.x, pos.y)).ok();
                                    if let Some((x, y)) = coords {
                                        info!("Enter pressed - sending click at terminal ({}, {})", x, y);
                                        let _ = click_tx.send((x, y)).await;
                                    }
                                }
                                _ => {}
                            }
                        }
                        }
                    }
                }
            })
        };

        // Click handler task - sends clicks to the browser
        let click_task = {
            let streamer = Arc::clone(&streamer);
            tokio::spawn(async move {
                while let Some((x, y)) = click_rx.recv().await {
                    // Convert terminal coordinates to browser viewport coordinates
                    // Terminal char represents 2 vertical pixels (half-block)
                    // Browser viewport is 1920x1080
                    let browser_x = (x as f64 / width as f64) * 1920.0;
                    let browser_y = (y as f64 / height as f64) * 1080.0;

                    info!("Clicking at terminal ({}, {}) -> browser ({:.0}, {:.0})", x, y, browser_x, browser_y);

                    if let Err(e) = streamer.click(browser_x, browser_y).await {
                        error!("Failed to send click: {}", e);
                    }
                }
            })
        };

        // Wait for shutdown signal (from keyboard or Ctrl+C signal)
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received");
            }
            result = tokio::signal::ctrl_c() => {
                if let Err(e) = result {
                    error!("Error waiting for Ctrl+C signal: {}", e);
                }
                info!("Ctrl+C signal received");
            }
        }

        info!("Shutting down...");

        // Cleanup (tasks will be cancelled when dropped)
        drop(screenshot_task);
        drop(render_task);
        drop(display_task);
        drop(keyboard_task);
        drop(click_task);

        Ok(())
    }
}

