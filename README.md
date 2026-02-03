# 🍞 TOAST (the browser)

> A blazingly fast terminal-based web browser that renders web pages using Unicode half-blocks at 15fps.

TOAST brings the web to your terminal with smooth 15fps rendering, perceptually accurate color quantization, and a memory-safe Rust implementation. Experience websites like never before - right in your command line.

## Features

- Renders any webpage in your terminal using Unicode half-block characters (▀)
- 15fps smooth rendering
- ANSI 256-color support with perceptually accurate color quantization
- Memory-safe Rust implementation
- Zero unsafe code

## Installation

### Prerequisites

- **Rust** (1.70 or later): [Install Rust](https://rustup.rs/)
- **Chrome/Chromium**: Must be installed on your system
- **Terminal**: With ANSI 256-color support (iTerm2, Alacritty, kitty, gnome-terminal, etc.)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/toast-thebrowser.git
cd toast-thebrowser

# Build in release mode
cargo build --release

# The binary will be at ./target/release/toast
```

### Install Locally

```bash
cargo install --path crates/toast
```

This installs the `toast` binary to `~/.cargo/bin/` (ensure it's in your PATH).

## Usage

```bash
toast <url>

# Examples
toast https://example.com
toast wikipedia.org
toast https://github.com
```

Press Ctrl+C to exit.

## Architecture

TOAST uses a multi-stage async pipeline:

1. **Browser** - Headless Chrome via DevTools Protocol
2. **Screenshot** - Capture at 15fps (JPEG format)
3. **Decode** - JPEG → RGB image
4. **Scale** - Resize to terminal dimensions
5. **Quantize** - RGB → ANSI 256 colors (using 32KB LUT for O(1) lookup)
6. **Convert** - Pixels → Unicode half-blocks
7. **Render** - Double-buffered terminal output

## Performance

Target: 15fps (66ms per frame)

- Color quantization: <5ms (O(1) LUT)
- Half-block conversion: <10ms (parallel processing)
- Terminal rendering: <10ms (differential updates)

## Requirements

- Chrome/Chromium installed on the system (or any Chromium-based browser)
- Terminal with ANSI 256-color support (iTerm2, Alacritty, kitty, gnome-terminal)

### Using Custom Chrome Executable

If you're using a Chromium fork (like Helium, Brave, Edge, etc.), set the `CHROME_PATH` environment variable:

```bash
export CHROME_PATH=/path/to/your/chromium-browser
toast https://example.com
```

## Project Structure

```
toast-thebrowser/
├── crates/
│   ├── toast/           # Main binary crate
│   ├── toast-browser/   # Browser control via DevTools Protocol
│   ├── toast-core/      # Core types and utilities
│   ├── toast-render/    # Rendering pipeline
│   └── toast-terminal/  # Terminal output and ANSI handling
├── examples/            # Example programs
└── README.md
```

## Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**
4. **Run tests**: `cargo test`
5. **Format code**: `cargo fmt`
6. **Lint**: `cargo clippy`
7. **Commit**: `git commit -m 'Add amazing feature'`
8. **Push**: `git push origin feature/amazing-feature`
9. **Open a Pull Request**

### Development Commands

This project uses [just](https://github.com/casey/just) for task running:

```bash
just build         # Build in release mode
just test          # Run tests
just run <URL>     # Run the browser
just fmt           # Format code
just lint          # Run clippy
just ci            # Full CI check
```

## Roadmap

- [ ] Mouse support for clickable links
- [ ] Keyboard navigation
- [ ] Page scrolling
- [ ] History and bookmarks
- [ ] Multiple tabs
- [ ] Configuration file support

## Acknowledgments

Built with:
- [Rust](https://rust-lang.org/) - Memory-safe systems programming
- [Tokio](https://tokio.rs/) - Async runtime
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/) - Browser automation

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Made with ❤️ and Rust
