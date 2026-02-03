use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;

/// Terminal controller - manages raw mode and alternate screen
pub struct Terminal {
    _guard: TerminalGuard,
}

impl Terminal {
    /// Initialize the terminal in raw mode with alternate screen
    pub fn new() -> Result<Self> {
        let mut stdout = stdout();

        // Enter alternate screen
        execute!(stdout, EnterAlternateScreen)?;

        // Enable raw mode
        terminal::enable_raw_mode()?;

        // Hide cursor
        execute!(stdout, cursor::Hide)?;

        Ok(Self {
            _guard: TerminalGuard,
        })
    }

    /// Get current terminal dimensions
    pub fn size(&self) -> Result<(usize, usize)> {
        let (cols, rows) = terminal::size()?;
        Ok((cols as usize, rows as usize))
    }
}

/// RAII guard to ensure terminal cleanup on drop
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = execute!(stdout, cursor::Show);
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
