#!/bin/bash
set -e

echo "=== Building TOAST Browser ==="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed"
    echo "Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if Chrome is installed
if ! command -v google-chrome &> /dev/null && ! command -v chromium &> /dev/null && ! command -v chromium-browser &> /dev/null; then
    echo "Warning: Chrome/Chromium not found in PATH"
    echo "Install Chrome: brew install --cask google-chrome (macOS)"
    echo "Or Chromium: sudo apt install chromium-browser (Linux)"
fi

echo "Building workspace..."
cargo build --release

echo ""
echo "=== Build successful! ==="
echo ""
echo "Run with: cargo run --release -- <url>"
echo "Example: cargo run --release -- https://example.com"
echo ""
echo "Or install: cargo install --path crates/toast"
echo "Then run: toast https://example.com"
