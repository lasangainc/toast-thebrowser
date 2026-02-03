# Just commands for TOAST browser development

# List all available commands
default:
    @just --list

# Build the project in release mode
build:
    cargo build --release

# Build in debug mode
build-debug:
    cargo build

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run the browser with a URL
run URL:
    cargo run --release -- {{URL}}

# Run in debug mode
run-debug URL:
    cargo run -- {{URL}}

# Run the color test example
test-colors:
    cargo run --example test_colors

# Run static render example
example-static URL:
    cargo run --example static_render {{URL}}

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Clean build artifacts
clean:
    cargo clean

# Install the binary
install:
    cargo install --path crates/toast

# Run benchmarks (when implemented)
bench:
    cargo bench

# Generate documentation
docs:
    cargo doc --open

# Watch for changes and rebuild
watch:
    cargo watch -x build

# Full CI check (fmt, lint, test, build)
ci: fmt lint test build

# Quick development cycle
dev URL: build-debug
    cargo run -- {{URL}}
