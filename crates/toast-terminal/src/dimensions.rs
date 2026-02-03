use anyhow::Result;
use crossterm::terminal;

/// Get the current terminal dimensions
pub fn get_terminal_size() -> Result<(usize, usize)> {
    let (cols, rows) = terminal::size()?;
    Ok((cols as usize, rows as usize))
}
