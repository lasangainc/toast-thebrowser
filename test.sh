#!/bin/bash
set -e

echo "=== Testing TOAST Browser ==="

echo "Running unit tests..."
cargo test

echo ""
echo "=== Running color test example ==="
echo "This will display a color gradient in your terminal."
echo "Press Enter to continue..."
read

cargo run --example test_colors

echo ""
echo "=== All tests passed! ==="
