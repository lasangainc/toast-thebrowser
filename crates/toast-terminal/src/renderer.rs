use anyhow::Result;
use crossterm::{cursor, execute, queue};
use parking_lot::Mutex;
use std::io::{stdout, Write};
use toast_core::TerminalFrame;

/// Double-buffered terminal renderer with differential updates
pub struct Renderer {
    front_buffer: Mutex<Option<TerminalFrame>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            front_buffer: Mutex::new(None),
        }
    }

    /// Render a frame to the terminal using differential updates
    pub fn render(&self, new_frame: TerminalFrame) -> Result<()> {
        let mut front = self.front_buffer.lock();
        let mut stdout = stdout();

        match front.as_ref() {
            None => {
                // First frame - render everything
                self.render_full(&mut stdout, &new_frame)?;
            }
            Some(old_frame) => {
                // Differential update - only render changed cells
                self.render_diff(&mut stdout, old_frame, &new_frame)?;
            }
        }

        stdout.flush()?;
        *front = Some(new_frame);

        Ok(())
    }

    /// Render entire frame (used for first frame)
    fn render_full<W: Write>(&self, w: &mut W, frame: &TerminalFrame) -> Result<()> {
        // Move to top-left
        queue!(w, cursor::MoveTo(0, 0))?;

        for y in 0..frame.height {
            for x in 0..frame.width {
                if let Some(cell) = frame.get(x, y) {
                    // Write ANSI escape codes for colors and character
                    write!(
                        w,
                        "\x1b[38;5;{}m\x1b[48;5;{}m{}",
                        cell.foreground.as_u8(),
                        cell.background.as_u8(),
                        cell.character
                    )?;
                }
            }
            // Don't add newline on last row to avoid scrolling
            if y < frame.height - 1 {
                queue!(w, cursor::MoveToNextLine(1))?;
            }
        }

        // Reset colors
        write!(w, "\x1b[0m")?;

        Ok(())
    }

    /// Render only changed cells (differential update)
    fn render_diff<W: Write>(
        &self,
        w: &mut W,
        old_frame: &TerminalFrame,
        new_frame: &TerminalFrame,
    ) -> Result<()> {
        // Ensure frames are same size
        if old_frame.width != new_frame.width || old_frame.height != new_frame.height {
            return self.render_full(w, new_frame);
        }

        let mut last_x = None;
        let mut last_y = None;

        for y in 0..new_frame.height {
            for x in 0..new_frame.width {
                let old_cell = old_frame.get(x, y);
                let new_cell = new_frame.get(x, y);

                // Only update if cell changed
                if old_cell != new_cell {
                    if let Some(cell) = new_cell {
                        // Move cursor if necessary
                        if last_x != Some(x) || last_y != Some(y) {
                            queue!(w, cursor::MoveTo(x as u16, y as u16))?;
                        }

                        // Write cell
                        write!(
                            w,
                            "\x1b[38;5;{}m\x1b[48;5;{}m{}",
                            cell.foreground.as_u8(),
                            cell.background.as_u8(),
                            cell.character
                        )?;

                        last_x = Some(x + 1);
                        last_y = Some(y);
                    }
                }
            }
        }

        // Reset colors
        write!(w, "\x1b[0m")?;

        Ok(())
    }

    /// Clear the screen
    pub fn clear(&self) -> Result<()> {
        let mut stdout = stdout();
        execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
