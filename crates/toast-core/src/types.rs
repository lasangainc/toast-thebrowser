use bytes::Bytes;

/// Dimensions of a viewport or terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// RGB color with 8-bit channels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// ANSI 256 color index (0-255)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiColor(pub u8);

impl AnsiColor {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

/// A single terminal cell with character and colors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCell {
    pub character: char,
    pub foreground: AnsiColor,
    pub background: AnsiColor,
}

/// Raw screenshot data from browser
#[derive(Debug, Clone)]
pub struct Screenshot {
    pub data: Bytes,
    pub format: ImageFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

/// Decoded RGB image
#[derive(Debug, Clone)]
pub struct RgbImage {
    pub data: Vec<u8>, // RGB triplets, row-major
    pub width: u32,
    pub height: u32,
}

impl RgbImage {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(data.len(), (width * height * 3) as usize);
        Self { data, width, height }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Rgb {
        let offset = ((y * self.width + x) * 3) as usize;
        Rgb::new(
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
        )
    }
}

/// Terminal frame buffer
#[derive(Debug, Clone)]
pub struct TerminalFrame {
    pub cells: Vec<TerminalCell>,
    pub width: usize,
    pub height: usize,
}

impl TerminalFrame {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![
            TerminalCell {
                character: ' ',
                foreground: AnsiColor(0),
                background: AnsiColor(0),
            };
            width * height
        ];
        Self { cells, width, height }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&TerminalCell> {
        if x < self.width && y < self.height {
            self.cells.get(y * self.width + x)
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: TerminalCell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }
}

/// Cursor position for overlay cursor
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
}
